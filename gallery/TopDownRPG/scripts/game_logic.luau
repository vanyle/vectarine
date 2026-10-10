--[[
The rules of the game, updated every frame: moving, attacking, creatures, projectiles, pickups, exits and needs.

`update(game, input, dt)` never reads the keyboard: game_screen.luau reads it and passes an `Input`.
This is what lets tests and debug_tools.luau play the game at full speed.

Things the screen must show (dialogs, title cards, the death screen...) are added to `game.events`.
]]
const blocks = require("./blocks")
const combat = require("./combat")
const dream = require("./dream")
const effects = require("./effects")
const frost = require("./frost")
const gallery = require("./gallery")
const interactions = require("./interactions")
const inventory = require("./inventory")
const items = require("./data/items")
const level = require("./level")
const lighting = require("./lighting")
const loader = require("@vectarine/loader")
const monster_ai = require("./monster_ai")
const monsters = require("./data/monsters")
const notebook = require("./notebook")
const player_rules = require("./player")
const storm = require("./storm")
const swamp = require("./swamp")
const theme = require("./theme")
const tutorials = require("./tutorials")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

export type Input = {
	--- Direction of the movement (not normalized: (1, 1) means up-right... down-right, y points down)
	move: vec.Vec2,
	attack: boolean,
	interact: boolean,
}

module.NO_INPUT = { move = vec.V2(0, 0), attack = false, interact = false } :: Input

local function directionOf(move: vec.Vec2, current: types.Direction): types.Direction
	if math.abs(move.x) > math.abs(move.y) then
		return if move.x > 0 then "right" else "left"
	elseif math.abs(move.y) > 0 then
		return if move.y > 0 then "down" else "up"
	end
	return current
end

local function message(game: types.Game, text: string)
	game.events[#game.events + 1] = { kind = "message", text = text }
end

-- MARK: Player

local function movePlayer(game: types.Game, input: Input, dt: number)
	local player = game.player
	local lvl = game.level :: any
	local move = input.move
	if move:length() > 0.01 then
		move = move:normalized()
		-- Keep facing the same way when moving diagonally, unless the main direction changes
		local direction = directionOf(move, player.facing)
		local forward = combat.DIRECTIONS[player.facing]
		if move:dot(forward) < 0.3 then
			player.facing = direction
		end
	end
	-- Deep sand slows the player down (unless they wear the sandstride anklets), the wind of storms pushes them
	local ground = level.groundAt(lvl, level.cellOf(player.pos).x, level.cellOf(player.pos).y)
	local slow = if ground and ground.slow and not (ground.sand and player.artefacts.sandstride_anklets) then ground.slow else 1
	-- On ice the player keeps their momentum (see frost.luau); freezing slows them down
	local wanted = move:scale(player_rules.speed(player) * slow * frost.speedFactor(player))
	local velocity = frost.steer(game, wanted, dt) + player.knockback + storm.wind(game)
	local onIce = ground ~= nil and ground.slippery == true
	player.knockback = player.knockback:scale(math.exp(-dt * (if onIce then 4 else 12)))
	local cell = level.cellOf(player.pos)
	local climber = player.artefacts.climbing_claws == true
	local crampons = player.artefacts.rime_crampons == true
	local wallHere = level.wallAt(lvl, cell.x, cell.y)
	local climbing = wallHere ~= nil and ((climber and wallHere.climbable == true) or (crampons and wallHere.icefall == true))
	local mover: level.Mover = {
		height = player.height,
		climber = climber,
		crampons = crampons,
		-- On a ladder (or on vines with the claws, on an icefall with the crampons), the player can change height
		onLadder = level.isLadder(lvl, cell.x, cell.y) or climbing,
		boat = player.artefacts.ferry_skiff == true,
		masked = player.artefacts.beaked_mask == true,
		dreaming = dream.isDreaming(game),
	}
	local before = player.pos
	player.pos = level.move(lvl, player.pos, velocity:scale(dt), player_rules.RX, player_rules.RY, mover)
	frost.afterMove(game, before, dt)
	player.moving = (player.pos - before):length() > 0.001 and move:length() > 0.01
	if player.moving then
		player.walkTime += dt
	else
		player.walkTime = 0
	end
	swamp.update(game, move:length() > 0.01, dt)
	frost.update(game, dt)
	blocks.update(game, move, dt)
	-- The height of the player follows the ground, except on ladders
	cell = level.cellOf(player.pos)
	if not level.isLadder(lvl, cell.x, cell.y) then
		local wall = level.wallAt(lvl, cell.x, cell.y)
		if not (wall and (wall.climbable or wall.icefall)) then
			player.height = level.heightAt(lvl, cell.x, cell.y)
		end
	end
end

local function collectPickups(game: types.Game, dt: number)
	local player = game.player
	for _, pickup in game.pickups do
		pickup.age += dt
		local distance = (pickup.pos - player.pos):length()
		-- Dropped items wait a bit, so that they don't go straight back into a full bag
		if not pickup.dead and pickup.age > 0.6 and distance < 1.6 then
			if distance > 0.45 then
				pickup.pos = pickup.pos:lerp(player.pos, math.min(1, dt * 8))
			else
				local left = inventory.give(player, pickup.item, pickup.count)
				local item = items.items[pickup.item]
				if left < pickup.count then
					local taken = pickup.count - left
					effects.floatText(player.pos + vec.V2(0, -1.2), if taken > 1 then `+{taken} {item.name}` else `+ {item.name}`, theme.colors.text)
					effects.burst(pickup.pos, "sparkle", 6)
					tutorials.onPickup(game, pickup.item)
					if item.kind == "artefact" then
						game.events[#game.events + 1] = { kind = "artefact", item = pickup.item }
					end
				end
				if left == 0 then
					pickup.dead = true
					if pickup.key then
						game.flags[pickup.key] = true
					end
				else
					pickup.count = left
					pickup.age = -2
					message(game, "Your bag is full.")
				end
			end
		end
	end
	for i = #game.pickups, 1, -1 do
		if game.pickups[i].dead then
			table.remove(game.pickups, i)
		end
	end
end

--- Walking on an exit (a digit of the map linked to another level) changes the level. Players arrive in a level one
--- step away from its exit (see game_state.luau), so turning back right away works.
local function checkExits(game: types.Game)
	local player = game.player
	local cell = level.cellOf(player.pos)
	local onEntry: string? = nil
	local index = 1
	for name, cells in game.level.entries do
		for i, entryCell in cells do
			if entryCell.x == cell.x and entryCell.y == cell.y then
				onEntry = name
				index = i
			end
		end
	end
	-- The level changes during a transition: the player still stands on the exit meanwhile, don't leave twice
	if onEntry ~= game.flags.leavingBy then
		game.flags.leavingBy = nil
	end
	if onEntry and onEntry ~= game.flags.leavingBy then
		local link = game.level.links[onEntry]
		if link then
			game.flags.leavingBy = onEntry
			game.events[#game.events + 1] = { kind = "travel", level = link.level, entry = link.entry, index = index }
		end
	end
end

-- MARK: Update

function module.update(game: types.Game, input: Input, dt: number)
	local player = game.player
	if player.hp <= 0 then
		return
	end
	game.time += dt
	player.attackTimer = math.max(0, player.attackTimer - dt)
	player.swingTime += dt
	player.invincible = math.max(0, player.invincible - dt)

	movePlayer(game, input, dt)
	if input.attack then
		local problem = combat.attack(game)
		if problem then
			message(game, problem)
		end
	end
	if input.interact then
		local interaction = interactions.find(game)
		if interaction then
			interaction.run(game)
		end
	end

	-- Creatures (phantoms only exist for a dreaming player: awake, they wait where they are)
	dream.update(game)
	gallery.update(game, dt)
	for _, monster in game.monsters do
		if not monster.dead and dream.isPresent(game, monster) then
			monster_ai.update(game, monster, dt)
			local species = monsters.species[monster.kind]
			local touching = (monster.pos - player.pos):length() < species.radius + 0.3
			local sameHeight = species.flying or monster.height == player.height
			if touching and sameHeight and monster_ai.isDangerous(monster) then
				combat.creatureHurts(game, monster, species.damage, species.damageType, monster.pos, species.poison)
			end
			-- Creatures seen are added to the bestiary
			if (monster.pos - player.pos):length() < 5 and game.notebook.creatures[monster.kind] == nil and not species.hazard and not species.prize then
				game.notebook.creatures[monster.kind] = 0
			end
		end
	end
	for i = #game.monsters, 1, -1 do
		if game.monsters[i].dead then
			table.remove(game.monsters, i)
		end
	end

	tutorials.checkCreatures(game)
	interactions.checkOfferings(game)
	combat.updateProjectiles(game, dt)
	combat.updateBombs(game, dt)
	collectPickups(game, dt)
	checkExits(game)
	lighting.update(game, dt)
	local sources = lighting.sources(game)
	player_rules.updateNeeds(game, dt, lighting.isSafe(game, sources))
	notebook.explore(game)
end

return module
