--[[
The state of the game: creating a new game, entering and leaving levels, undo and restart.

The rules of each turn (moving, monsters, bombs, ...) are in rules.luau.
This module does not draw anything and does not read the keyboard.

Levels can be saved to a compact string with `saveLevel` and restored with `loadLevel`.
This is used for undo and for restarting a level.
]]
const dialog = require("./dialog")
const effects = require("./effects")
const items = require("./data/items")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const npcs = require("./data/npcs")
const rooms = require("./data/rooms")
const types = require("./types")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")
const world = require("./world")

const module = loader.init() or {}

const PICKUP_SPRITES: { [string]: string } = {
	key = "town/key",
	arrows = "town/arrow",
	bombs = "town/bomb",
	gear = "extra/gear",
}
const POTION_SPRITES: { [string]: string } = {
	leap = "dungeon/potion_red",
	slant = "dungeon/potion_blue",
	clear = "dungeon/potion_clear",
}

-- MARK: Entities

local function newEntity(kind: types.EntityKind, roomId: string, pos: vec.Vec2, sprite: string): types.Entity
	return {
		kind = kind,
		sprite = sprite,
		room = roomId,
		pos = pos,
		removed = false,
		drawPos = pos,
		bump = vec.V2(0, 0),
		facingLeft = false,
	}
end

local function createEntity(roomId: string, spawn: world.Spawn): types.Entity?
	local entry = spawn.entry
	local pos = spawn.pos
	if entry.monster then
		local e = newEntity("monster", roomId, pos, monsters.monsters[entry.monster].sprite)
		e.monster = entry.monster
		e.dir = entry.dir
		return e
	elseif entry.pickup then
		local sprite = if entry.pickup == "potion"
			then POTION_SPRITES[entry.potion or "clear"]
			else PICKUP_SPRITES[entry.pickup]
		local e = newEntity("pickup", roomId, pos, sprite)
		e.pickup = entry.pickup
		e.amount = entry.amount or 1
		e.potion = entry.potion
		return e
	elseif entry.gate then
		local sprite = if entry.spikes
			then "extra/spikes"
			elseif entry.gate == "monsters" then "dungeon/skull_wall"
			else "dungeon/gate"
		local e = newEntity("gate", roomId, pos, sprite)
		e.condition = entry.gate
		e.count = entry.count
		e.spikes = entry.spikes
		return e
	elseif entry.door then
		return newEntity("door", roomId, pos, "dungeon/locked_door")
	elseif entry.target then
		local e = newEntity("target", roomId, pos, "town/target")
		e.hit = false
		return e
	elseif entry.barrel then
		return newEntity("barrel", roomId, pos, "dungeon/barrel")
	elseif entry.npc then
		local npc = npcs.npcs[entry.npc]
		assert(npc, `Unknown npc '{entry.npc}'`)
		local e = newEntity("npc", roomId, pos, npc.sprite)
		e.npc = entry.npc
		return e
	elseif entry.sign then
		local e = newEntity("sign", roomId, pos, "town/sign")
		e.lines = entry.sign
		return e
	elseif entry.ladder then
		local e = newEntity("ladder", roomId, pos, "town/ladder_down")
		e.level = entry.ladder
		return e
	end
	return nil
end

--- Create the entities of a room from its map. Returns where the player starts in this room, if anywhere.
local function spawnRoom(game: types.Game, roomId: string): vec.Vec2?
	local list: { types.Entity } = {}
	local start: vec.Vec2? = nil
	for _, spawn in game.spawns[roomId] :: { world.Spawn } do
		if spawn.entry.player then
			start = spawn.pos
		else
			local entity = createEntity(roomId, spawn)
			if entity then
				table.insert(list, entity)
			end
		end
	end
	game.entities[roomId] = list
	return start
end

--- Placed bombs are the only entities created during a level.
function module.createBomb(game: types.Game, pos: vec.Vec2, fuse: number): types.Entity
	local bomb = newEntity("bomb", game.currentRoom, pos, "town/bomb")
	bomb.fuse = fuse
	table.insert(game.entities[game.currentRoom], bomb)
	return bomb
end

--- Methods of the Game object (see types.Game for their documentation).
--- They are stored in the module so that hot-reloading this file updates the methods of the running game.
module.methods = module.methods or {}
module.methods.__index = module.methods
local Game = module.methods

function module.new(): types.Game
	local level, spawns = world.build()
	local game = {
		world = level,
		spawns = spawns,
		player = {
			pos = vec.V2(0, 0),
			drawPos = vec.V2(0, 0),
			bump = vec.V2(0, 0),
			facing = "right",
			dead = false,
		},
		entities = {},
		inventory = { hasPickaxe = false, arrows = 0, bombs = 0, keys = 0 },
		currentRoom = rooms.startRoom,
		finished = {},
		flags = {},
		conditions = { plates = false, targets = false, monsters = false },
		turn = 0,
		status = "playing",
	}
	setmetatable(game, Game)
	local typed = (game :: any) :: types.Game
	for roomId in spawns do
		local start = spawnRoom(typed, roomId)
		if roomId == rooms.startRoom and start then
			typed.player.pos = start
			typed.player.drawPos = start
		end
	end
	module.refreshConditions(typed)
	return typed
end

-- MARK: Queries

--- Is the entity part of the game right now? Some characters only appear at some points of the story.
function module.isPresent(game: types.Game, entity: types.Entity): boolean
	if entity.removed then
		return false
	end
	if entity.npc then
		local visible = npcs.npcs[entity.npc].visible
		return visible == nil or visible(game)
	end
	return true
end

local function entitiesOfRoomAt(game: types.Game, pos: vec.Vec2): { types.Entity }
	local room = game.world.rooms[game.currentRoom]
	if world.isInRoom(room, pos) then
		return game.entities[game.currentRoom]
	end
	local other = world.roomAt(game.world, pos)
	return if other then game.entities[other.id] else {}
end

--- All the entities on a tile.
function module.entitiesAt(game: types.Game, pos: vec.Vec2): { types.Entity }
	local result = {}
	for _, entity in entitiesOfRoomAt(game, pos) do
		if entity.pos.x == pos.x and entity.pos.y == pos.y and module.isPresent(game, entity) then
			table.insert(result, entity)
		end
	end
	return result
end

--- The first entity of the given kind on a tile.
function module.entityOfKindAt(game: types.Game, pos: vec.Vec2, kind: types.EntityKind): types.Entity?
	for _, entity in entitiesOfRoomAt(game, pos) do
		if
			entity.kind == kind
			and entity.pos.x == pos.x
			and entity.pos.y == pos.y
			and module.isPresent(game, entity)
		then
			return entity
		end
	end
	return nil
end

function Game.entityAt(self: types.Game, pos: vec.Vec2): types.Entity?
	return module.entitiesAt(self, pos)[1]
end

function Game.objectAt(self: types.Game, pos: vec.Vec2): types.Terrain?
	return world.getTerrain(self.world, world.OBJECTS, pos)
end

function Game.groundAt(self: types.Game, pos: vec.Vec2): types.Terrain?
	return world.getTerrain(self.world, world.GROUND, pos)
end

function Game.finishedCount(self: types.Game): number
	local count = 0
	for _ in self.finished do
		count += 1
	end
	return count
end

function Game.say(_self: types.Game, pos: vec.Vec2, text: string, color: vec4.Vec4?)
	effects.floatText(pos, text, color)
end

function Game.startDialog(self: types.Game, speaker: types.Speaker?, steps: { types.DialogStep })
	dialog.open(self, speaker, steps)
end

-- MARK: Gates

--- Is something heavy standing on this tile? Flying monsters are too light to press plates.
function module.isPressed(game: types.Game, pos: vec.Vec2): boolean
	local player = game.player
	if not player.dead and player.pos.x == pos.x and player.pos.y == pos.y then
		return true
	end
	for _, entity in module.entitiesAt(game, pos) do
		if entity.kind == "bomb" or entity.kind == "barrel" then
			return true
		elseif entity.kind == "monster" then
			local monsterId: string = entity.monster or ""
			if not monsters.monsters[monsterId].flying then
				return true
			end
		end
	end
	return false
end

--- Recompute the conditions of the gates of the current room. rules.luau calls it whenever something changes.
function module.refreshConditions(game: types.Game)
	local room = game.world.rooms[game.currentRoom]
	local allPlatesPressed = #room.plates > 0
	for _, pos in room.plates do
		if not module.isPressed(game, pos) then
			allPlatesPressed = false
			break
		end
	end
	local anyTarget, allTargetsHit, anyMonster = false, true, false
	for _, entity in game.entities[game.currentRoom] do
		if module.isPresent(game, entity) then
			if entity.kind == "target" then
				anyTarget = true
				allTargetsHit = allTargetsHit and entity.hit == true
			elseif entity.kind == "monster" then
				anyMonster = true
			end
		end
	end
	game.conditions.plates = allPlatesPressed
	game.conditions.targets = anyTarget and allTargetsHit
	game.conditions.monsters = not anyMonster
end

--- Is the gate open? Gates of the "levels" kind open when enough levels are finished.
function module.isGateOpen(game: types.Game, gate: types.Entity): boolean
	if gate.condition == "levels" then
		return game:finishedCount() >= (gate.count or 0)
	end
	local conditions = game.conditions
	if gate.condition == "targets" then
		return conditions.targets
	elseif gate.condition == "monsters" then
		return conditions.monsters
	end
	return conditions.plates
end

-- MARK: Inventory

--- The items the player can currently use, in the order of items.order
function module.ownedItems(game: types.Game): { string }
	local inventory = game.inventory
	local result = {}
	for _, itemId in items.order do
		local item = items.items[itemId]
		local owned = if item.ammo then (inventory :: any)[item.ammo] > 0 else inventory.hasPickaxe
		if owned then
			table.insert(result, itemId)
		end
	end
	return result
end

--- The item used by [Space]. If the selected item ran out, the first available item is selected.
function module.selectedItem(game: types.Game): string?
	local owned = module.ownedItems(game)
	if not table.find(owned, game.inventory.selected or "") then
		game.inventory.selected = owned[1]
	end
	return game.inventory.selected
end

--- The item used by [Space]: the pickaxe when facing a boulder, otherwise the selected item.
function module.itemForSpace(game: types.Game): string?
	local object = game:objectAt(game.player.pos + types.directions[game.player.facing])
	if game.inventory.hasPickaxe and object and object.breaksWith and object.breaksWith.mine then
		return "pickaxe"
	end
	return module.selectedItem(game)
end

-- MARK: Saving levels

const DIRECTION_CODES: { [string]: number } = { up = 1, right = 2, down = 3, left = 4 }
const EFFECT_CODES: { [string]: number } = { leap = 1, slant = 2 }
const EFFECTS: { types.Effect } = { "leap", "slant" }
const KINDS: { types.EntityKind } =
	{ "monster", "npc", "sign", "pickup", "bomb", "barrel", "target", "gate", "door", "ladder" }
const KIND_CODES: { [string]: number } = {}
for index, kind in KINDS do
	KIND_CODES[kind] = index
end
--- Binary layout of the player part and of each entity in a saved level (see string.pack)
const PLAYER_FORMAT = "<bbBBBBBB"
const ENTITY_FORMAT = "<BBbbBBBB"
const PLAYER_SIZE = 8
const ENTITY_SIZE = 8
const OBJECTS_SIZE = rooms.ROOM_WIDTH * rooms.ROOM_HEIGHT

--- string.unpack for formats that only contain numbers (it is typed as returning unknown values)
local function unpackNumbers(format: string, data: string, offset: number?): ...number
	return (string.unpack :: any)(format, data, offset or 1)
end

--- Save everything that can change in the current level to a string: the player, the objects and the entities.
function module.saveLevel(game: types.Game): string
	local roomId = game.currentRoom
	local origin = game.world.rooms[roomId].origin
	local player = game.player
	local inventory = game.inventory
	local parts = {
		string.pack(
			PLAYER_FORMAT,
			player.pos.x - origin.x,
			player.pos.y - origin.y,
			DIRECTION_CODES[player.facing],
			if player.effect then EFFECT_CODES[player.effect] else 0,
			if player.dead then 1 else 0,
			inventory.arrows,
			inventory.bombs,
			inventory.keys
		),
		world.saveRoomObjects(game.world, roomId),
	}
	for _, entity in game.entities[roomId] do
		local flags = (if entity.hit then 1 else 0) + (if (entity :: any).fresh then 2 else 0)
		table.insert(
			parts,
			string.pack(
				ENTITY_FORMAT,
				KIND_CODES[entity.kind],
				if entity.removed then 1 else 0,
				entity.pos.x - origin.x,
				entity.pos.y - origin.y,
				if entity.dir then DIRECTION_CODES[entity.dir] else 0,
				if entity.effect then EFFECT_CODES[entity.effect] else 0,
				entity.fuse or 0,
				flags
			)
		)
	end
	return table.concat(parts)
end

--- Restore the current level to a state saved with `saveLevel`.
--- Entities are updated in place, so that their smooth drawn positions animate towards the restored ones.
function module.loadLevel(game: types.Game, state: string)
	local roomId = game.currentRoom
	local origin = game.world.rooms[roomId].origin
	local player = game.player
	local inventory = game.inventory

	local px, py, facing, effect, dead, arrows, bombs, keys = unpackNumbers(PLAYER_FORMAT, state)
	player.pos = origin + vec.V2(px, py)
	player.facing = types.directionList[facing]
	player.effect = if effect > 0 then EFFECTS[effect] else nil
	player.dead = dead == 1
	player.lastIntent = nil
	inventory.arrows = arrows
	inventory.bombs = bombs
	inventory.keys = keys

	world.loadRoomObjects(game.world, roomId, string.sub(state, PLAYER_SIZE + 1, PLAYER_SIZE + OBJECTS_SIZE))

	local list = game.entities[roomId]
	local count = (#state - PLAYER_SIZE - OBJECTS_SIZE) // ENTITY_SIZE
	for index = 1, count do
		local offset = PLAYER_SIZE + OBJECTS_SIZE + (index - 1) * ENTITY_SIZE + 1
		local kind, removed, x, y, dir, entityEffect, fuse, flags = unpackNumbers(ENTITY_FORMAT, state, offset)
		local pos = origin + vec.V2(x, y)
		local entity = list[index]
		if not entity then
			-- Only bombs are created during a level
			entity = newEntity(KINDS[kind], roomId, pos, "town/bomb")
			list[index] = entity
		end
		entity.removed = removed == 1
		entity.pos = pos
		entity.dir = if dir > 0 then types.directionList[dir] else nil
		entity.effect = if entityEffect > 0 then EFFECTS[entityEffect] else nil
		entity.fuse = if entity.kind == "bomb" then fuse else nil
		entity.hit = if entity.kind == "target" then bit32.band(flags, 1) ~= 0 else nil
		local dynamic = entity :: any
		dynamic.fresh = bit32.band(flags, 2) ~= 0
	end
	for index = #list, count + 1, -1 do
		list[index] = nil
	end
	if game.level then
		game.level.won = false
	end
	module.refreshConditions(game)
end

-- MARK: Levels

--- Enter a level from its ladder. The level always starts fresh.
function module.enterLevel(game: types.Game, index: number)
	local roomId = world.levelRoomId(index)
	assert(game.world.rooms[roomId], `There is no level {index}`)
	game.overworldReturn = game.player.pos
	world.resetRoomObjects(game.world, roomId)
	local start = spawnRoom(game, roomId)
	assert(start, `Level {index} has no starting position (@)`)

	local player = game.player
	player.pos = start
	player.drawPos = start
	player.effect = nil
	player.dead = false
	player.facing = "down"
	game.inventory.arrows = 0
	game.inventory.bombs = 0
	game.inventory.keys = 0
	game.currentRoom = roomId
	module.refreshConditions(game)
	effects.clear()
	game.level = {
		index = index,
		roomId = roomId,
		startPos = start,
		initialState = module.saveLevel(game),
		undoStack = {},
		won = false,
		moves = 0,
	}
end

--- Build the world again after a data file was edited (hot reload). The current level restarts.
function module.rebuildWorld(game: types.Game)
	game.world, game.spawns = world.build()
	for roomId, room in game.world.rooms do
		if room.kind == "overworld" then
			spawnRoom(game, roomId)
		end
	end
	local level = game.level
	if level and game.world.rooms[level.roomId] then
		local returnPos = game.overworldReturn
		module.enterLevel(game, level.index)
		game.overworldReturn = returnPos
	elseif level then
		module.leaveLevel(game)
	else
		module.refreshConditions(game)
	end
end

--- Go back to the overworld, on the ladder of the level.
function module.leaveLevel(game: types.Game)
	if not game.level then
		return
	end
	local returnPos = game.overworldReturn or game.player.pos
	local room = world.roomAt(game.world, returnPos)
	game.level = nil
	game.currentRoom = if room then room.id else "mine"
	local player = game.player
	player.pos = returnPos
	player.drawPos = returnPos
	player.effect = nil
	player.dead = false
	player.facing = "down"
	game.inventory.arrows = 0
	game.inventory.bombs = 0
	game.inventory.keys = 0
	effects.clear()
	module.refreshConditions(game)
end

--- Put the level back to how it started. Restarting can be undone.
function module.restartLevel(game: types.Game)
	local level = game.level
	if not level then
		return
	end
	local current: string = module.saveLevel(game)
	table.insert(level.undoStack, current)
	module.loadLevel(game, level.initialState)
end

--- Go back one turn. Returns false if there is nothing to undo.
function module.undo(game: types.Game): boolean
	local level = game.level
	if not level or #level.undoStack == 0 then
		return false
	end
	module.loadLevel(game, table.remove(level.undoStack) :: string)
	return true
end

--- Mark the current level as finished.
function module.finishLevel(game: types.Game)
	if game.level then
		game.finished[game.level.index] = true
	end
end

return module
