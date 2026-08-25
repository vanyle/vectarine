--[[
The rules of a turn. Everything that happens when the player acts is in this file.

A turn goes like this:
1. The player acts: move, use an item or wait (see `act`)
2. Every monster of the room moves once (see monster_ai.luau for where they want to go)
3. Lit bombs tick down and explode
4. Gates open or close depending on their condition. A gate closing on someone crushes them.

Touching a monster is deadly, and so are spikes, chasms and explosions.
When the player dies, game_screen.luau restarts the level (or the player can undo).

Talking, reading and bumping into walls are free: no turn passes.
]]
const effects = require("./effects")
const game_state = require("./game_state")
const items = require("./data/items")
const loader = require("@vectarine/loader")
const monster_ai = require("./monster_ai")
const monsters = require("./data/monsters")
const npcs = require("./data/npcs")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")
const world = require("./world")

const module = loader.init() or {}

--- Turns before a dropped bomb explodes
module.BOMB_FUSE = 3

const DIAGONALS: { [types.Direction]: vec.Vec2 } = {
	up = vec.V2(1, -1),
	right = vec.V2(1, 1),
	down = vec.V2(-1, 1),
	left = vec.V2(-1, -1),
}

local function samePos(a: vec.Vec2, b: vec.Vec2): boolean
	return a.x == b.x and a.y == b.y
end

--- What moving in a direction means for someone with the given potion effect.
--- Returns the offset of the move, and the offset of the tile jumped over for leaps.
function module.moveFor(effect: types.Effect?, dir: types.Direction): (vec.Vec2, vec.Vec2?)
	local step = types.directions[dir]
	if effect == "leap" then
		return step:scale(2), step
	elseif effect == "slant" then
		-- Like a bishop: each arrow key moves diagonally (up = up-right, right = down-right, ...)
		return DIAGONALS[dir], nil
	end
	return step, nil
end

-- MARK: Tiles

local function isFlying(entity: types.Entity): boolean
	return monsters.monsters[entity.monster or ""].flying == true
end

--- Entities that nobody can walk through, whatever the situation.
local function isObstacle(game: types.Game, entity: types.Entity): boolean
	local kind = entity.kind
	if kind == "gate" then
		return not entity.spikes and not game_state.isGateOpen(game, entity)
	end
	return kind == "npc" or kind == "sign" or kind == "barrel" or kind == "target" or kind == "bomb" or kind == "door"
end

--- Walls, trees, rocks and closed gates: you can't jump over them and arrows stop on them.
local function isTall(game: types.Game, pos: vec.Vec2): boolean
	local object = game:objectAt(pos)
	if object and object.solid then
		return true
	end
	for _, entity in game_state.entitiesAt(game, pos) do
		if isObstacle(game, entity) and entity.kind ~= "bomb" then
			return true
		end
	end
	return false
end

local function isSolidTerrain(game: types.Game, pos: vec.Vec2, flying: boolean): boolean
	local object = game:objectAt(pos)
	if object and object.solid then
		return true
	end
	local ground = game:groundAt(pos)
	return ground ~= nil and ground.solid == true and not (flying and ground.flyable)
end

local function raisedSpikeGateAt(game: types.Game, pos: vec.Vec2): boolean
	local gate = game_state.entityOfKindAt(game, pos, "gate")
	return gate ~= nil and gate.spikes == true and not game_state.isGateOpen(game, gate)
end

-- MARK: Deaths

function module.killPlayer(game: types.Game, cause: string)
	local player = game.player
	if player.dead then
		return
	end
	player.dead = true
	player.deathCause = cause
	effects.shake(0.35)
	effects.flash(theme.colors.danger, 0.5)
	effects.burst(player.pos, "blood", 14)
end

local function killMonster(game: types.Game, monster: types.Entity, cause: string)
	if monster.removed then
		return
	end
	monster.removed = true
	effects.dying(monster, cause)
	effects.burst(monster.pos, "poof", 10)
	effects.shake(0.12)
	game_state.refreshConditions(game)
end

-- MARK: Explosions

--- Explode a bomb or a barrel. Everything in the 3x3 area around it is hit, which can set off other bombs.
local function explode(game: types.Game, center: vec.Vec2)
	local queue = { center }
	while #queue > 0 do
		local c = table.remove(queue, 1) :: vec.Vec2
		for _, entity in game_state.entitiesAt(game, c) do
			if entity.kind == "bomb" or entity.kind == "barrel" then
				entity.removed = true
			end
		end
		effects.explosion(c)
		for dy = -1, 1 do
			for dx = -1, 1 do
				local pos = c + vec.V2(dx, dy)
				local object = game:objectAt(pos)
				if object and object.breaksWith and object.breaksWith.bomb then
					world.setTerrain(game.world, world.OBJECTS, pos, nil)
					effects.burst(pos, "debris", 8)
				end
				for _, entity in game_state.entitiesAt(game, pos) do
					if entity.kind == "monster" then
						killMonster(game, entity, "explosion")
					elseif entity.kind == "target" then
						entity.hit = true
					elseif entity.kind == "bomb" or entity.kind == "barrel" then
						-- Chain reaction
						entity.removed = true
						table.insert(queue, pos)
					end
				end
				if samePos(pos, game.player.pos) then
					module.killPlayer(game, "explosion")
				end
			end
		end
	end
	game_state.refreshConditions(game)
end

-- MARK: Items

--- What each item does when used. `target` is the tile in front of the player.
module.itemActions = {
	mine = function(game: types.Game, target: vec.Vec2)
		local object = game:objectAt(target)
		if object and object.breaksWith and object.breaksWith.mine then
			world.setTerrain(game.world, world.OBJECTS, target, nil)
			effects.burst(target, "debris", 10)
			effects.shake(0.08)
		end
	end,

	--- The arrow flies until it hits something: monsters die, targets are hit, explosives explode.
	shoot = function(game: types.Game, target: vec.Vec2)
		if game.inventory.arrows <= 0 then
			return
		end
		game.inventory.arrows -= 1
		local step = target - game.player.pos
		local room = game.world.rooms[game.currentRoom]
		local pos = target
		while world.isInRoom(room, pos) do
			local hit = game_state.entityOfKindAt(game, pos, "monster")
				or game_state.entityOfKindAt(game, pos, "target")
				or game_state.entityOfKindAt(game, pos, "bomb")
				or game_state.entityOfKindAt(game, pos, "barrel")
			if hit or isTall(game, pos) then
				effects.arrow(game.player.pos, pos, game.player.facing)
				if hit and hit.kind == "monster" then
					killMonster(game, hit, "arrow")
				elseif hit and hit.kind == "target" then
					hit.hit = true
					effects.burst(pos, "sparkle", 10)
					game_state.refreshConditions(game)
				elseif hit then
					explode(game, pos)
				end
				return
			end
			pos += step
		end
		effects.arrow(game.player.pos, pos - step, game.player.facing)
	end,

	bomb = function(game: types.Game, target: vec.Vec2)
		local inventory = game.inventory
		if inventory.bombs <= 0 then
			return
		end
		local ground = game:groundAt(target)
		local blocked = isSolidTerrain(game, target, false)
			or (ground ~= nil and ground.deadly ~= nil)
			or #game_state.entitiesAt(game, target) > 0
		if blocked then
			return
		end
		inventory.bombs -= 1
		local bomb = game_state.createBomb(game, target, module.BOMB_FUSE);
		-- A bomb doesn't tick during the turn it was dropped
		(bomb :: any).fresh = true
		game_state.refreshConditions(game)
	end,
}

-- MARK: Player

local function collect(game: types.Game, pickup: types.Entity)
	pickup.removed = true
	local inventory = game.inventory
	local kind = pickup.pickup
	if kind == "gear" then
		if game.level then
			game.level.won = true
		end
		effects.burst(pickup.pos, "sparkle", 30)
		return
	elseif kind == "key" then
		inventory.keys += 1
	elseif kind == "arrows" then
		inventory.arrows += pickup.amount or 1
		inventory.selected = "bow"
	elseif kind == "bombs" then
		inventory.bombs += pickup.amount or 1
		inventory.selected = "bomb"
	elseif kind == "potion" then
		game.player.effect = if pickup.potion == "clear" then nil else pickup.potion :: types.Effect
	end
	effects.burst(pickup.pos, "sparkle", 8)
	effects.pop(pickup)
end

--- What happens when the player arrives on a tile.
local function arrive(game: types.Game)
	local player = game.player
	local pos = player.pos
	local ground = game:groundAt(pos)
	local object = game:objectAt(pos)
	if ground and ground.deadly == "fall" then
		module.killPlayer(game, "fall")
		return
	elseif (object and object.deadly == "spikes") or raisedSpikeGateAt(game, pos) then
		module.killPlayer(game, "spikes")
		return
	end
	for _, entity in game_state.entitiesAt(game, pos) do
		if entity.kind == "pickup" then
			collect(game, entity)
		elseif entity.kind == "ladder" then
			game.enteringLevel = entity.level
		end
	end
end

--- Move the player. Returns true if a turn passes.
local function movePlayer(game: types.Game, dir: types.Direction): boolean
	local player = game.player
	player.facing = dir
	local offset, jumped = module.moveFor(player.effect, dir)
	local target = player.pos + offset
	local function bump()
		player.bump = types.directions[dir]:scale(0.15)
		return false
	end

	if jumped and isTall(game, player.pos + jumped) then
		return bump()
	end
	local room = world.roomAt(game.world, target)
	if not room then
		return bump()
	end
	local changesRoom = room.id ~= game.currentRoom
	if changesRoom and (room.kind ~= "overworld" or game.level ~= nil) then
		return bump()
	end
	if isSolidTerrain(game, target, false) then
		return bump()
	end
	for _, entity in game_state.entitiesAt(game, target) do
		if entity.kind == "monster" then
			-- Walking into a monster is a bad idea
			player.pos = target
			module.killPlayer(game, "monster")
			return true
		elseif entity.kind == "door" and game.inventory.keys > 0 then
			game.inventory.keys -= 1
			entity.removed = true
			effects.burst(target, "sparkle", 12)
			effects.shake(0.1)
		elseif isObstacle(game, entity) then
			return bump()
		end
	end

	player.pos = target
	player.lastIntent = dir
	effects.burst(target, "dust", 3)
	if changesRoom then
		game.currentRoom = room.id
		game_state.refreshConditions(game)
	end
	arrive(game)
	game_state.refreshConditions(game)
	return true
end

--- Talk to the character or read the sign in front of the player. Returns true if there was someone.
local function talk(game: types.Game, target: vec.Vec2): boolean
	local entity = game:entityAt(target)
	if not entity then
		return false
	elseif entity.kind == "npc" then
		local npc = npcs.npcs[entity.npc or ""]
		game:startDialog({ name = npc.name, sprite = npc.sprite }, npc.talk(game))
		return true
	elseif entity.kind == "sign" then
		game:startDialog({ name = "Sign", sprite = entity.sprite }, (entity.lines or {}) :: { types.DialogStep })
		return true
	end
	return false
end

-- MARK: Monsters

--- Try to move a monster in a direction. Returns false if it is blocked.
local function moveMonster(game: types.Game, monster: types.Entity, dir: types.Direction): boolean
	local flying = isFlying(monster)
	local offset, jumped = module.moveFor(monster.effect, dir)
	local target = monster.pos + offset
	local player = game.player

	if jumped then
		if isTall(game, monster.pos + jumped) then
			return false
		end
		if samePos(monster.pos + jumped, player.pos) then
			monster.pos = target
			module.killPlayer(game, "monster")
			return true
		end
	end
	if not world.isInRoom(game.world.rooms[monster.room], target) then
		return false
	end
	if samePos(target, player.pos) then
		monster.bump = offset:scale(0.3)
		monster.pos = target
		module.killPlayer(game, "monster")
		return true
	end
	if isSolidTerrain(game, target, flying) then
		return false
	end
	local ground = game:groundAt(target)
	if ground and ground.deadly == "fall" and not flying then
		return false -- Monsters are scared of heights
	end
	local potion: types.Entity? = nil
	for _, entity in game_state.entitiesAt(game, target) do
		if entity.kind == "monster" or isObstacle(game, entity) or entity.kind == "ladder" then
			return false
		elseif entity.kind == "pickup" then
			if entity.pickup ~= "potion" then
				return false
			end
			potion = entity
		end
	end

	if offset.x ~= 0 then
		monster.facingLeft = offset.x < 0
	end
	monster.pos = target
	if potion then
		potion.removed = true
		monster.effect = if potion.potion == "clear" then nil else potion.potion :: types.Effect
		effects.burst(target, "sparkle", 8)
	end
	local object = game:objectAt(target)
	if not flying and ((object and object.deadly == "spikes") or raisedSpikeGateAt(game, target)) then
		killMonster(game, monster, "spikes")
	end
	game_state.refreshConditions(game)
	return true
end

local function monstersTurn(game: types.Game)
	for _, monster in game.entities[game.currentRoom] do
		if game.player.dead then
			return
		end
		if monster.kind == "monster" and not monster.removed then
			local moved = false
			for _, dir in monster_ai.intents(game, monster) do
				if moveMonster(game, monster, dir) then
					moved = true
					break
				end
			end
			-- Monsters flying in a straight line turn around when they are blocked
			if not moved and monster.dir then
				monster.dir = types.opposite[monster.dir]
			end
		end
	end
end

-- MARK: Turn

local function bombsTurn(game: types.Game)
	for _, bomb in game.entities[game.currentRoom] do
		if bomb.kind == "bomb" and not bomb.removed then
			local dynamic = bomb :: any
			if dynamic.fresh then
				dynamic.fresh = false
			else
				bomb.fuse = (bomb.fuse or 1) - 1
				if (bomb.fuse or 0) <= 0 then
					explode(game, bomb.pos)
				end
			end
		end
	end
end

--- Gates closing on someone crush them. Spike traps rising under someone kill them.
local function gatesTurn(game: types.Game)
	for _, gate in game.entities[game.currentRoom] do
		if gate.kind == "gate" and not gate.removed and not game_state.isGateOpen(game, gate) then
			local cause = if gate.spikes then "spikes" else "crushed"
			if samePos(gate.pos, game.player.pos) then
				module.killPlayer(game, cause)
			end
			local monster = game_state.entityOfKindAt(game, gate.pos, "monster")
			if monster and not (gate.spikes and isFlying(monster)) then
				killMonster(game, monster, cause)
			end
		end
	end
end

local function endTurn(game: types.Game)
	game_state.refreshConditions(game)
	if not game.player.dead then
		monstersTurn(game)
	end
	bombsTurn(game)
	game_state.refreshConditions(game)
	gatesTurn(game)
	game.turn += 1
	if game.level then
		game.level.moves += 1
	end
	effects.tick()
end

export type Action = "up" | "down" | "left" | "right" | "use" | "wait" | "pickaxe" | "bow" | "bomb"

--- Perform an action of the player: a direction, "use" (talk or use the selected item), "wait",
--- or the name of an item to use it directly. Returns true if a turn passed.
--- In levels, the state before the action is saved so that it can be undone.
function module.act(game: types.Game, action: Action): boolean
	local level = game.level
	if game.player.dead or (level and level.won) then
		return false
	end
	local before = if level then game_state.saveLevel(game) else nil
	game.player.lastIntent = nil

	local tookTurn = false
	if types.directions[action :: any] then
		tookTurn = movePlayer(game, action :: types.Direction)
	elseif action == "wait" then
		tookTurn = true
	else
		local target = game.player.pos + types.directions[game.player.facing]
		if action == "use" and talk(game, target) then
			return false
		end
		local itemId = if action == "use" then game_state.itemForSpace(game) else action
		local item = if itemId then items.items[itemId] else nil
		if item then
			effects.swing(target, item.sprite, game.player.facing)
			module.itemActions[item.action](game, target)
		end
		tookTurn = true
	end

	if tookTurn then
		if not (level and level.won) then
			endTurn(game)
		end
		if level and before then
			table.insert(level.undoStack, before)
		end
	end
	return tookTurn
end

return module
