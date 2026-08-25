--[[
The world: all the rooms (overworld and levels), stored inside a tilemap.

The rooms are written as text in data/rooms.luau and data/levels.luau. At startup, `build` converts them into
2 layers of numbers (terrain ids, see data/terrain.luau) and creates a generated tilemap that reads from them.
Everything else (drawing, collisions) goes through the tilemap.

The overworld rooms are placed next to each other following `rooms.layout`. The levels are placed below,
far from each other, so that you can never walk from one level into another.

When the level changes (a rock is broken), `setTerrain` updates the layer and invalidates the tilemap
so that the change is picked up.

World coordinates are in tiles, with (0, 0) the top-left tile of the top-left room and y pointing down.

Hot reload: when a data file (rooms, levels, legend, terrain) is saved, it runs again and replaces its tables.
`isOutdated` notices it, and game_screen.luau rebuilds the world.
]]
const legend = require("./data/legend")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const rooms = require("./data/rooms")
const terrain = require("./data/terrain")
const tile = require("@vectarine/tile")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

module.GROUND = 0
module.OBJECTS = 1

const ROOM_WIDTH = rooms.ROOM_WIDTH
const ROOM_HEIGHT = rooms.ROOM_HEIGHT
--- Generated tilemaps are made of chunks of 16x16 tiles
const CHUNK_SIZE = 16
--- Levels are placed on a grid with this many columns, leaving an empty room between each level
const LEVEL_COLUMNS = 5

--- Something to create in a room (monster, npc, gate, ...)
export type Spawn = {
	pos: vec.Vec2,
	entry: legend.LegendEntry,
}

--- The id of the room of a level
function module.levelRoomId(index: number): string
	return `level{index}`
end

--- The room definition of a room id, for both overworld rooms and levels
function module.roomData(roomId: string): rooms.Room
	local levelIndex = tonumber(string.match(roomId, "^level(%d+)$"))
	if levelIndex then
		return levels.levels[levelIndex]
	end
	return rooms.rooms[roomId]
end

local function terrainId(name: string, roomId: string): number
	local t = terrain.byName[name]
	assert(t, `Room '{roomId}' uses unknown terrain '{name}'. Add it to data/terrain.luau`)
	return t.id
end

local function hasEntity(entry: legend.LegendEntry): boolean
	return entry.monster ~= nil
		or entry.pickup ~= nil
		or entry.gate ~= nil
		or entry.door == true
		or entry.target == true
		or entry.barrel == true
		or entry.npc ~= nil
		or entry.sign ~= nil
		or entry.ladder ~= nil
		or entry.player == true
end

--- The data tables the world is built from. Saving a data file replaces them.
local function sources(): { any }
	return { rooms.rooms, rooms.layout, levels.levels, legend.characters, terrain.byId }
end

--- Was a data file edited since the world was built?
function module.isOutdated(world: types.World): boolean
	for index, source in sources() do
		if world.sources[index] ~= source then
			return true
		end
	end
	return false
end

local function gridKey(gridX: number, gridY: number): string
	return `{gridX},{gridY}`
end

--- Build the world from the room definitions. Also returns what to create in each room.
function module.build(): (types.World, { [string]: { Spawn } })
	-- 1. Decide where each room goes. Grid positions start at 0.
	type Placement = { id: string, kind: "overworld" | "level", gridX: number, gridY: number }
	local placements: { Placement } = {}
	for gridY, row in rooms.layout do
		for gridX, roomId in row do
			if roomId ~= "" then
				local placement: Placement = { id = roomId, kind = "overworld", gridX = gridX - 1, gridY = gridY - 1 }
				table.insert(placements, placement)
			end
		end
	end
	for index in levels.levels do
		local placement: Placement = {
			id = module.levelRoomId(index),
			kind = "level",
			gridX = ((index - 1) % LEVEL_COLUMNS) * 2,
			gridY = #rooms.layout + 1 + math.floor((index - 1) / LEVEL_COLUMNS) * 2,
		}
		table.insert(placements, placement)
	end

	local gridWidth, gridHeight = 0, 0
	for _, placement in placements do
		gridWidth = math.max(gridWidth, placement.gridX + 1)
		gridHeight = math.max(gridHeight, placement.gridY + 1)
	end
	local width = gridWidth * ROOM_WIDTH
	local height = gridHeight * ROOM_HEIGHT

	-- 2. Read the maps
	local layers: { [number]: { number } } = {
		[module.GROUND] = table.create(width * height, 0),
		[module.OBJECTS] = table.create(width * height, 0),
	}
	local roomInfos: { [string]: types.RoomInfo } = {}
	local spawns: { [string]: { Spawn } } = {}
	local grid: { [string]: string } = {}

	for _, placement in placements do
		local roomId = placement.id
		local room = module.roomData(roomId)
		assert(room, `The room '{roomId}' does not exist`)
		assert(#room.map == ROOM_HEIGHT, `Room '{roomId}' should have {ROOM_HEIGHT} rows`)

		local origin = vec.V2(placement.gridX * ROOM_WIDTH, placement.gridY * ROOM_HEIGHT)
		local info: types.RoomInfo = { id = roomId, kind = placement.kind, origin = origin, plates = {} }
		roomInfos[roomId] = info
		spawns[roomId] = {}
		grid[gridKey(placement.gridX, placement.gridY)] = roomId

		for y, line in room.map do
			assert(#line == ROOM_WIDTH, `Room '{roomId}': row {y} has {#line} characters instead of {ROOM_WIDTH}`)
			for x = 1, ROOM_WIDTH do
				local char = string.sub(line, x, x)
				local entry = (room.legend and room.legend[char]) or legend.characters[char]
				assert(entry, `Room '{roomId}': unknown character '{char}' at row {y}, column {x}`)

				local pos = origin + vec.V2(x - 1, y - 1)
				local index = 1 + pos.x + pos.y * width

				local groundName = entry.ground or "floor"
				if groundName == "floor" then
					groundName = room.floor
				end
				layers[module.GROUND][index] = terrainId(groundName, roomId)
				if groundName == "plate" then
					table.insert(info.plates, pos)
				end

				local objectName = entry.object
				if objectName == "border" then
					objectName = room.border
				end
				if objectName then
					layers[module.OBJECTS][index] = terrainId(objectName, roomId)
				end

				if hasEntity(entry) then
					table.insert(spawns[roomId], { pos = pos, entry = entry })
				end
			end
		end
	end

	-- 3. Create the tilemap. The generator is called when the tilemap needs a chunk of 16x16 tiles.
	-- It must return the tiles as a flat list: result[1 + dx + dy * 16].
	local tilemap = tile.createGeneratedTilemap(function(layer: number, chunkX: number, chunkY: number)
		local chunk = table.create(CHUNK_SIZE * CHUNK_SIZE, 0)
		local source = layers[layer]
		if not source then
			return chunk
		end
		for dy = 0, CHUNK_SIZE - 1 do
			for dx = 0, CHUNK_SIZE - 1 do
				local x = chunkX + dx
				local y = chunkY + dy
				if x >= 0 and y >= 0 and x < width and y < height then
					chunk[1 + dx + dy * CHUNK_SIZE] = source[1 + x + y * width]
				end
			end
		end
		return chunk
	end)

	-- Generate every chunk now. Chunks can't be generated while the tilemap is being read by getTilemapPart,
	-- and the renderer looks at neighboring tiles during getTilemapPart to pick the right sprites.
	for chunkY = 0, height - 1, CHUNK_SIZE do
		for chunkX = 0, width - 1, CHUNK_SIZE do
			tilemap:getTile(module.GROUND, chunkX, chunkY)
			tilemap:getTile(module.OBJECTS, chunkX, chunkY)
		end
	end

	local world: types.World = {
		width = width,
		height = height,
		tilemap = tilemap,
		layers = layers,
		initialLayers = {
			[module.GROUND] = table.clone(layers[module.GROUND]),
			[module.OBJECTS] = table.clone(layers[module.OBJECTS]),
		},
		rooms = roomInfos,
		grid = grid,
		sources = sources(),
	}
	return world, spawns
end

--- Returns the terrain at the given position of a layer, or nil if there is nothing.
function module.getTerrain(world: types.World, layer: number, pos: vec.Vec2): types.Terrain?
	return terrain.byId[world.tilemap:getTile(layer, pos.x, pos.y)]
end

--- Tell the tilemap that the tiles of a room changed, and regenerate them right away.
local function refreshRoom(world: types.World, layer: number, origin: vec.Vec2)
	-- A room is at most 16x16, so its 4 corners touch every chunk it overlaps.
	local corners = {
		origin,
		origin + vec.V2(ROOM_WIDTH - 1, 0),
		origin + vec.V2(0, ROOM_HEIGHT - 1),
		origin + vec.V2(ROOM_WIDTH - 1, ROOM_HEIGHT - 1),
	}
	for _, corner in corners do
		world.tilemap:invalidate(layer, corner.x, corner.y)
	end
	for _, corner in corners do
		world.tilemap:getTile(layer, corner.x, corner.y)
	end
end

--- Change the terrain at the given position of a layer. Use nil to remove it.
function module.setTerrain(world: types.World, layer: number, pos: vec.Vec2, name: string?)
	if pos.x < 0 or pos.y < 0 or pos.x >= world.width or pos.y >= world.height then
		return
	end
	local id = if name then terrain.byName[name].id else 0
	world.layers[layer][1 + pos.x + pos.y * world.width] = id
	world.tilemap:invalidate(layer, pos.x, pos.y)
	-- Regenerate the chunk right away so it is ready when it is drawn.
	world.tilemap:getTile(layer, pos.x, pos.y)
end

--- The objects of a room as a compact string (one character per tile). Used to save and restore levels.
function module.saveRoomObjects(world: types.World, roomId: string): string
	local origin = world.rooms[roomId].origin
	local source = world.layers[module.OBJECTS]
	local chars = table.create(ROOM_WIDTH * ROOM_HEIGHT)
	for y = 0, ROOM_HEIGHT - 1 do
		local rowStart = 1 + origin.x + (origin.y + y) * world.width
		for x = 0, ROOM_WIDTH - 1 do
			table.insert(chars, string.char(source[rowStart + x]))
		end
	end
	return table.concat(chars)
end

--- Restore the objects of a room saved with `saveRoomObjects`.
function module.loadRoomObjects(world: types.World, roomId: string, saved: string)
	local origin = world.rooms[roomId].origin
	local target = world.layers[module.OBJECTS]
	local changed = false
	for y = 0, ROOM_HEIGHT - 1 do
		local rowStart = 1 + origin.x + (origin.y + y) * world.width
		for x = 0, ROOM_WIDTH - 1 do
			local id = string.byte(saved, 1 + x + y * ROOM_WIDTH)
			if target[rowStart + x] ~= id then
				target[rowStart + x] = id
				changed = true
			end
		end
	end
	if changed then
		refreshRoom(world, module.OBJECTS, origin)
	end
end

--- Put the objects of a room back to how they were in the room map.
function module.resetRoomObjects(world: types.World, roomId: string)
	local origin = world.rooms[roomId].origin
	for y = 0, ROOM_HEIGHT - 1 do
		local rowStart = 1 + origin.x + (origin.y + y) * world.width
		for x = 0, ROOM_WIDTH - 1 do
			world.layers[module.OBJECTS][rowStart + x] = world.initialLayers[module.OBJECTS][rowStart + x]
		end
	end
	refreshRoom(world, module.OBJECTS, origin)
end

--- Returns the room containing the given position, or nil if the position is outside of all rooms.
function module.roomAt(world: types.World, pos: vec.Vec2): types.RoomInfo?
	local roomId = world.grid[gridKey(math.floor(pos.x / ROOM_WIDTH), math.floor(pos.y / ROOM_HEIGHT))]
	return if roomId then world.rooms[roomId] else nil
end

--- Returns true if the position is inside the room.
function module.isInRoom(room: types.RoomInfo, pos: vec.Vec2): boolean
	local offset = pos - room.origin
	return offset.x >= 0 and offset.y >= 0 and offset.x < ROOM_WIDTH and offset.y < ROOM_HEIGHT
end

return module
