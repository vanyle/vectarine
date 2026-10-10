--[[
Levels: turns the text maps of data/maps/ into a level that can be played and drawn.

A level is stored in 3 arrays of cells:
- `ground`: the terrain id of the ground (data/terrain.luau)
- `wall`: the terrain id of the wall on top of the ground, or 0
- `heights`: the height layer of the cell (0 by default, see "Heights" below)

The flags of the game change how a level is built: cracked walls blown up by a bomb are gone, canal beds are full of
water once the Great Pump runs again, gates opened stay open, broken crates stay broken.

Heights: the explorium has several levels of ground, like cliffs. A character can only walk on cells of its own height,
and goes from one height to another with ladders. The `heights` of a level map gives the height of each cell
(a digit, or "." for 0). Walls between heights are drawn with their front face, thanks to the tilted camera.

Levels are listed in data/levels.luau.
]]
const decor = require("./data/decor")
const legend = require("./data/legend")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const props = require("./data/props")
const terrain = require("./data/terrain")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

export type Level = types.LevelRuntime & {
	ground: { number },
	wall: { number },
	heights: { number },
	--- Props by cell index, to find solid props quickly
	propAt: { [number]: types.Prop },
	--- The ground of "." cells, also left by walls blown up by a bomb
	floorGround: number,
	--- Decorations drawn with the terrain, by cell index (see data/decor.luau)
	decor: { [number]: decor.Decoration },
	--- The flags of the game the level was built with (the same table): props that only show up with a flag
	--- (`data.showIf`, `data.hideIf`) are only in the way while they are there
	flags: { [string]: any },
}

--- The flag set when the cracked wall at a cell is blown up
function module.crackedKey(levelId: string, x: number, y: number): string
	return `cracked:{levelId}:{x},{y}`
end

local function cellIndex(level: Level, x: number, y: number): number
	return 1 + x + y * level.width
end

--- Build the level `id` from its definition in data/levels.luau, as the `flags` of the game left it.
function module.load(id: string, flags: { [string]: any }?): Level
	local data = levels.levels[id]
	assert(data, `There is no level '{id}'`)
	local state = flags or {}
	local map = data.map
	local height = #map
	local width = #map[1]
	local floor = terrain.byName[data.floor or "dirt"].id
	local style = if data.style then legend.styles[data.style] else nil
	local storm: types.Storm? = nil
	if data.storm then
		local wind = data.storm.wind
		storm = {
			calm = data.storm.calm,
			duration = data.storm.duration,
			always = data.storm.always,
			wind = vec.V2(wind[1], wind[2]):normalized(),
			strength = data.storm.strength,
		}
	end

	local level: Level = {
		id = id,
		name = data.name,
		subtitle = data.subtitle or "",
		width = width,
		height = height,
		darkness = data.darkness or 0.5,
		depth = data.depth or 1,
		heat = data.heat or 1,
		storm = storm,
		snowfall = data.snowfall or 0,
		darkAreas = data.darkAreas or {},
		version = 0,
		floorGround = floor,
		props = {},
		entries = {},
		links = data.links or {},
		start = nil,
		spawns = {},
		pickups = {},
		ground = table.create(width * height, floor),
		wall = table.create(width * height, 0),
		heights = table.create(width * height, 0),
		propAt = {},
		decor = {},
		flags = state,
	}

	--- Cells of things placed on the map (props, creatures, items...) without a ground: their ground is guessed
	--- from their neighbors, so that a bush placed in the grass stands on grass.
	local guessGround: { number } = {}

	for y, row in map do
		assert(#row == width, `Level '{id}': row {y} has {#row} characters instead of {width}`)
		for x = 1, width do
			local char = string.sub(row, x, x)
			local cell = vec.V2(x - 1, y - 1)
			local index = cellIndex(level, x - 1, y - 1)
			if string.match(char, "%d") and char ~= "0" then
				-- Entry points and exits
				level.entries[char] = level.entries[char] or {}
				table.insert(level.entries[char], cell)
				table.insert(guessGround, index)
				continue
			end
			local entry = (data.legend and data.legend[char]) or (style and style[char]) or legend.characters[char]
			assert(entry, `Level '{id}': unknown character '{char}' at row {y}, column {x}`)
			if entry.ground then
				local ground = terrain.byName[entry.ground]
				-- Canal beds are full of water once the Great Pump runs again
				if ground.wet and state.waterRestored then
					ground = terrain.byName[ground.wet]
				end
				level.ground[index] = ground.id
			elseif char ~= "." and not entry.wall then
				table.insert(guessGround, index)
			end
			if entry.wall then
				local wall = terrain.byName[entry.wall]
				if not (wall.cracked and state[module.crackedKey(id, x - 1, y - 1)]) then
					level.wall[index] = wall.id
				end
			end
			local key = `{id}:{x - 1},{y - 1}`
			if entry.prop then
				local kind = props.kinds[entry.prop]
				assert(kind, `Level '{id}': unknown prop '{entry.prop}'`)
				local propData: { [string]: any } = entry.data or {}
				local prop: types.Prop = { kind = entry.prop, cell = cell, key = key, data = propData }
				-- Gates, valves and sun eyes stay open once opened (see switches.luau)
				local switch = propData.id or propData.opens
				if kind.openKind and switch and state[`open:{switch}`] then
					prop.kind = kind.openKind
				end
				-- Braziers stay lit once lit, frozen chests stay thawed
				if kind.litKind and state[`lit:{key}`] then
					prop.kind = kind.litKind
				end
				if entry.prop == "frozen_chest" and state[`thawed:{key}`] then
					prop.kind = "chest"
				end
				table.insert(level.props, prop)
				-- Broken props (crates, pots) don't block the way any more
				if not state[`broken:{key}`] then
					level.propAt[index] = prop
				end
			end
			if entry.npc then
				-- The data of the entry too: `showIf`, `hideIf` (a character who moves once a quest is done)
				local npcData: { [string]: any } = table.clone(entry.data or {})
				npcData.npc = entry.npc
				local prop: types.Prop = { kind = "npc", cell = cell, key = key, data = npcData }
				table.insert(level.props, prop)
				level.propAt[index] = prop
			end
			if entry.monster then
				table.insert(level.spawns, {
					kind = entry.monster,
					cell = cell,
					uniqueKey = if entry.unique then `killed:{entry.unique}` else nil,
					group = entry.group,
				})
			end
			if entry.item then
				table.insert(level.pickups, { item = entry.item, count = entry.count or 1, cell = cell, key = `picked:{key}` })
			end
			if entry.start then
				level.start = cell
			end
			if entry.entry then
				level.entries[entry.entry] = level.entries[entry.entry] or {}
				table.insert(level.entries[entry.entry], cell)
			end
		end
	end

	-- The most common walkable ground around each guessed cell
	local guessed: { [number]: boolean } = {}
	for _, index in guessGround do
		guessed[index] = true
	end
	for _, index in guessGround do
		local x, y = (index - 1) % width, (index - 1) // width
		local counts: { [number]: number } = {}
		local best, bestCount = level.ground[index], 0
		for _, offset in { { 1, 0 }, { -1, 0 }, { 0, 1 }, { 0, -1 } } do
			local nx, ny = x + offset[1], y + offset[2]
			local neighbor = 1 + nx + ny * width
			if nx >= 0 and ny >= 0 and nx < width and ny < height and not guessed[neighbor] and level.wall[neighbor] == 0 then
				local ground = level.ground[neighbor]
				local t = terrain.get(ground)
				if t and t.walkable then
					counts[ground] = (counts[ground] or 0) + 1
					if counts[ground] > bestCount then
						best, bestCount = ground, counts[ground]
					end
				end
			end
		end
		level.ground[index] = best
	end

	-- Decorations: drawn with the terrain, except the big ones, which are props (they block the way)
	for y, row in data.decor or {} do
		for x = 1, math.min(#row, width) do
			local char = string.sub(row, x, x)
			local entry = if char ~= "." then decor.characters[char] else nil
			local index = cellIndex(level, x - 1, y - 1)
			local ground = terrain.get(level.ground[index])
			local onWall = level.wall[index] ~= 0
			-- Only where it makes sense now: ground things on walkable ground (lilies on water), wall things on walls
			-- (the canals of the desert fill with water, cracked walls are blown up)
			if entry then
				local fits = if entry.layer == "wall" then onWall
					elseif entry.water then ground ~= nil and not ground.walkable
					else not onWall and ground ~= nil and ground.walkable
				if not fits then
					entry = nil
				end
			end
			if entry then
				local cell = vec.V2(x - 1, y - 1)
				if entry.prop then
					local kinds: { string } = { entry.prop :: string }
					if entry.partner then
						table.insert(kinds, entry.partner :: string)
					end
					for i, kind in kinds do
						local at = cell + vec.V2(i - 1, 0)
						local prop: types.Prop = { kind = kind, cell = at, key = `{id}:{at.x},{at.y}`, data = {} }
						table.insert(level.props, prop)
						level.propAt[cellIndex(level, at.x, at.y)] = prop
					end
				else
					level.decor[cellIndex(level, x - 1, y - 1)] = entry
				end
			end
		end
	end

	if data.heights then
		for y, row in data.heights do
			for x = 1, math.min(#row, width) do
				local digit = tonumber(string.sub(row, x, x))
				if digit then
					level.heights[cellIndex(level, x - 1, y - 1)] = digit
				end
			end
		end
	end

	return level
end

--- A prop that was broken doesn't block the way any more
function module.clearProp(level: Level, cell: vec.Vec2)
	level.propAt[cellIndex(level, cell.x, cell.y)] = nil
end

--- Move a prop to another cell (an ice block sliding)
function module.moveProp(level: Level, prop: types.Prop, cell: vec.Vec2)
	if level.propAt[cellIndex(level, prop.cell.x, prop.cell.y)] == prop then
		level.propAt[cellIndex(level, prop.cell.x, prop.cell.y)] = nil
	end
	prop.cell = cell
	level.propAt[cellIndex(level, cell.x, cell.y)] = prop
end

--- Change the ground of a cell (thin ice breaking, an ice block filling a hole of water)
function module.setGround(level: Level, x: number, y: number, terrainName: string)
	level.ground[cellIndex(level, x, y)] = terrain.byName[terrainName].id
	level.version += 1
end

--- Blow up the cracked wall at a cell: it becomes floor. Returns true if there was a cracked wall.
function module.breakWall(level: Level, x: number, y: number): boolean
	local wall = module.wallAt(level, x, y)
	if not (wall and wall.cracked) then
		return false
	end
	local index = cellIndex(level, x, y)
	level.wall[index] = 0
	level.ground[index] = level.floorGround
	level.decor[index] = nil
	level.version += 1
	return true
end

-- MARK: Queries

function module.inside(level: Level, x: number, y: number): boolean
	return x >= 0 and y >= 0 and x < level.width and y < level.height
end

function module.groundAt(level: Level, x: number, y: number): terrain.Terrain?
	if not module.inside(level, x, y) then
		return nil
	end
	return terrain.get(level.ground[cellIndex(level, x, y)])
end

function module.wallAt(level: Level, x: number, y: number): terrain.Terrain?
	if not module.inside(level, x, y) then
		return nil
	end
	return terrain.get(level.wall[cellIndex(level, x, y)])
end

function module.heightAt(level: Level, x: number, y: number): number
	if not module.inside(level, x, y) then
		return 0
	end
	return level.heights[cellIndex(level, x, y)]
end

function module.propAt(level: Level, x: number, y: number): types.Prop?
	if not module.inside(level, x, y) then
		return nil
	end
	return level.propAt[cellIndex(level, x, y)]
end

function module.isLadder(level: Level, x: number, y: number): boolean
	local wall = module.wallAt(level, x, y)
	return wall ~= nil and wall.ladder == true
end

--- Who is moving, for collisions
export type Mover = {
	height: number,
	flying: boolean?,
	--- Can climb walls covered with vines (the player with the climbing claws)
	climber: boolean?,
	--- Currently on a ladder: can step on cells of any height
	onLadder: boolean?,
	--- Sails on water (the player with the ferry)
	boat: boolean?,
	--- Breathes in miasma (the player with the beaked mask)
	masked: boolean?,
	--- Climbs icefalls (the player with the rime crampons)
	crampons: boolean?,
	--- Dreams (the player with a low sanity, phantoms): crosses the veils, walks on dream bridges
	dreaming: boolean?,
	--- Slides into freezing water (a creature knocked across the ice: it falls in, see monster_ai.luau)
	falls: boolean?,
}

--- Can `mover` stand on the cell (x, y)?
function module.isFree(level: Level, x: number, y: number, mover: Mover): boolean
	if not module.inside(level, x, y) then
		return false
	end
	local index = cellIndex(level, x, y)
	local wall = terrain.get(level.wall[index])
	local climbing = false
	if wall then
		if (wall.climbable and mover.climber) or (wall.icefall and mover.crampons) then
			climbing = true
		elseif not wall.walkable then
			return false
		end
		climbing = climbing or wall.ladder == true
	end
	local ground = terrain.get(level.ground[index])
	local prop = level.propAt[index]
	local kind = if prop then props.kinds[prop.kind] else nil
	-- A lily pad carries whoever walks on it over the water (a dream bridge, only dreamers)
	local onFloor = kind ~= nil and kind.floor == true and (not kind.dream or mover.dreaming == true)
	local falling = mover.falls and ground ~= nil and ground.freezing
	if ground and not ground.walkable and not (mover.flying and ground.flyable) and not (mover.boat and ground.sailable) and not onFloor and not falling then
		return false
	end
	if ground and ground.miasma and not mover.masked and not mover.flying then
		return false
	end
	if ground and ground.veil and not mover.dreaming then
		return false
	end
	-- A phantom (`data.dream`, Corporal Vey) is only in the way of dreamers, and what is not there yet (or any more)
	-- is not in the way: a character who arrives later, the ferry before the bell is rung
	local phantom = prop ~= nil and prop.data.dream == true and not mover.dreaming
	local absent = prop ~= nil and ((prop.data.showIf ~= nil and not level.flags[prop.data.showIf])
		or (prop.data.hideIf ~= nil and level.flags[prop.data.hideIf] == true))
	if kind and kind.solid and not mover.flying and not phantom and not absent then
		return false
	end
	if mover.flying then
		return true
	end
	return climbing or mover.onLadder == true or level.heights[index] == mover.height
end

--- The cell containing a world position
function module.cellOf(pos: vec.Vec2): vec.Vec2
	return vec.V2(math.floor(pos.x), math.floor(pos.y))
end

--- Can a body of half-size (rx, ry) centered on `pos` be there?
function module.fits(level: Level, pos: vec.Vec2, rx: number, ry: number, mover: Mover): boolean
	local x1, x2 = math.floor(pos.x - rx), math.floor(pos.x + rx)
	local y1, y2 = math.floor(pos.y - ry), math.floor(pos.y + ry)
	for y = y1, y2 do
		for x = x1, x2 do
			if not module.isFree(level, x, y, mover) then
				return false
			end
		end
	end
	return true
end

--- Move a body by `delta`, sliding along walls. Returns the new position.
--- The move is done in small steps, one axis at a time, so that fast bodies don't go through walls.
--- A body that is already somewhere it can't be (the ice broke under it) can always step towards a free cell:
--- nothing can get stuck.
function module.move(level: Level, pos: vec.Vec2, delta: vec.Vec2, rx: number, ry: number, mover: Mover): vec.Vec2
	local steps = math.max(1, math.ceil(math.max(math.abs(delta.x), math.abs(delta.y)) / 0.1))
	local step = delta:scale(1 / steps)
	local x, y = pos.x, pos.y
	local function canGo(nx: number, ny: number): boolean
		if module.fits(level, vec.V2(nx, ny), rx, ry, mover) then
			return true
		end
		local stuck = not module.fits(level, vec.V2(x, y), rx, ry, mover)
		return stuck and module.isFree(level, math.floor(nx), math.floor(ny), mover)
	end
	for _ = 1, steps do
		if step.x ~= 0 and canGo(x + step.x, y) then
			x += step.x
		end
		if step.y ~= 0 and canGo(x, y + step.y) then
			y += step.y
		end
	end
	return vec.V2(x, y)
end

--- Where to put a body back on its feet (after a fall in the water or a bog): the center of the cell of `wanted` if
--- the body fits there, or else the closest cell around `from` where it fits (and `accept` agrees, if given).
function module.landingSpot(
	level: Level,
	wanted: vec.Vec2?,
	from: vec.Vec2,
	rx: number,
	ry: number,
	mover: Mover,
	accept: ((x: number, y: number) -> boolean)?
): vec.Vec2?
	local function spot(x: number, y: number): vec.Vec2?
		local center = vec.V2(x + 0.5, y + 0.6)
		if module.fits(level, center, rx, ry, mover) and (accept == nil or accept(x, y)) then
			return center
		end
		return nil
	end
	if wanted then
		local cell = module.cellOf(wanted)
		local found = spot(cell.x, cell.y)
		if found then
			return found
		end
	end
	local origin = module.cellOf(from)
	for radius = 1, 10 do
		local best: vec.Vec2? = nil
		local bestDistance = math.huge
		for dy = -radius, radius do
			for dx = -radius, radius do
				if math.max(math.abs(dx), math.abs(dy)) == radius then
					local found = spot(origin.x + dx, origin.y + dy)
					if found and (found - from):length() < bestDistance then
						best, bestDistance = found, (found - from):length()
					end
				end
			end
		end
		if best then
			return best
		end
	end
	return nil
end

--- Is there a clear line between two points (for projectiles and for monsters to see the player)?
function module.lineIsClear(level: Level, from: vec.Vec2, to: vec.Vec2): boolean
	local distance = (to - from):length()
	local steps = math.max(1, math.ceil(distance / 0.25))
	for i = 1, steps - 1 do
		local p = from:lerp(to, i / steps)
		local x, y = math.floor(p.x), math.floor(p.y)
		local wall = module.wallAt(level, x, y)
		if wall and not wall.walkable then
			return false
		end
		local prop = module.propAt(level, x, y)
		if prop and props.kinds[prop.kind].tall then
			return false
		end
	end
	return true
end

return module
