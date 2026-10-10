--[[
Draws the terrain of a level: grounds, walls, and their shadows.

The terrain never changes while playing, so the sprites of each chunk of 16x16 cells are computed once, when the
level is first drawn, and kept in fastlists: drawing a chunk is then a single call per pass, done in native code.

Passes, drawn in this order for all the visible chunks (so that consecutive draws use the same sheet and are batched):
for the ground then the walls: full sprites (and the sprite under transparent edges), autotiled quarters (water
has 2 frames), overlays (a ladder on a rock face), decorations (data/decor.luau: flowers on the ground, moss and
waterfalls on the walls). Then the shadows below wall faces and on the edges between heights.

Autotiled terrains are made of 4 quarters of 8x8 pixels, picked from the neighbors of each cell
(see tools/art/terrain.py for the 5 cases of a quarter).
]]
const fastlist = require("@vectarine/fastlist")
const image = require("@vectarine/image")
const level = require("./level")
const loader = require("@vectarine/loader")
const sprites = require("./sprites")
const terrain = require("./data/terrain")
const theme = require("./theme")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

const CHUNK = 16
const FACE_SHADOW = theme.withAlpha(theme.colors.darkness, 0.3)
const HEIGHT_EDGE = theme.withAlpha(theme.colors.darkness, 0.55)

--- A pass draws the quads of one sheet. `frame` is set for animated passes (water): only the current frame is drawn.
type Pass = { name: string, frame: number? }
const PASSES: { Pass } = {
	{ name = "groundBase" },
	{ name = "groundAuto" },
	{ name = "groundAuto0", frame = 0 },
	{ name = "groundAuto1", frame = 1 },
	{ name = "groundOver" },
	{ name = "groundDecor" },
	{ name = "wallBase" },
	{ name = "wallAuto" },
	{ name = "wallAuto0", frame = 0 },
	{ name = "wallAuto1", frame = 1 },
	{ name = "wallOver" },
	{ name = "wallDecor" },
	{ name = "wallDecor0", frame = 0 },
	{ name = "wallDecor1", frame = 1 },
}

--- The quads of a pass in a chunk: groups of 5 Vec2 for fastlist.drawTileQuads ((tile id, 0), then 4 corners)
type PassList = { sheet: image.ImageWithTileset, quads: fastlist.FastList }
type Chunk = { passes: { [string]: PassList }, shadows: fastlist.FastList }

--- The chunks of the level drawn last, by chunk index. Reset on reload, so that changes to the drawing show up,
--- and when the sprite sheets are reloaded (their tile ids may have changed).
module.cache = { level = nil :: level.Level?, version = -1, tilesetVersion = -1, chunks = {} :: { [number]: Chunk } }

-- MARK: Building

--- A stable pseudo-random number for a cell, to pick sprite variants
local function cellHash(x: number, y: number): number
	local h = (x * 374761393 + y * 668265263) % 2147483647
	h = (h * 1274126177) % 2147483647
	return h
end

local function pickVariant(list: { string }, x: number, y: number): string
	return list[(cellHash(x, y) % #list) + 1]
end

const QUARTERS = {
	{ name = "tl", dx = -1, dy = -1, offset = vec.V2(0, 0) },
	{ name = "tr", dx = 1, dy = -1, offset = vec.V2(0.5, 0) },
	{ name = "bl", dx = -1, dy = 1, offset = vec.V2(0, 0.5) },
	{ name = "br", dx = 1, dy = 1, offset = vec.V2(0.5, 0.5) },
}

--- Which terrain ids count as the same terrain for the edges of `t`
local joinCache: { [terrain.Terrain]: { [number]: boolean } } = {}
local function joinsOf(t: terrain.Terrain): { [number]: boolean }
	local cached = joinCache[t]
	if cached then
		return cached
	end
	local ids = { [t.id] = true }
	for _, name in t.joins or {} do
		ids[terrain.byName[name].id] = true
	end
	joinCache[t] = ids
	return ids
end

--- Collects the quads of a chunk, pass by pass, as plain tables before they become fastlists
type Builder = { [string]: { sheet: image.ImageWithTileset, points: { vec.Vec2 } } }

--- Add a sprite stretched over the quad of corners top-left `pos` and size `size` (in tiles, y down)
local function addQuad(builder: Builder, pass: string, sprite: string, pos: vec.Vec2, size: number)
	local sheet, id = sprites.tileOf(sprite)
	local entry = builder[pass]
	if not entry then
		entry = { sheet = sheet, points = {} }
		builder[pass] = entry
	end
	local points = entry.points
	-- Corners in the order of drawTileQuad: bottom-left, bottom-right, top-right, top-left (y points down)
	table.insert(points, vec.V2(id, 0))
	table.insert(points, pos + vec.V2(0, size))
	table.insert(points, pos + vec.V2(size, size))
	table.insert(points, pos + vec.V2(size, 0))
	table.insert(points, pos)
end

--- The autotiled quarters of a cell. Returns false if the cell is surrounded by the same terrain and has detailed
--- 16x16 sprites: one of them is used instead of quarters.
local function addAutotile(builder: Builder, prefix: string, lvl: level.Level, layer: { number }, t: terrain.Terrain, x: number, y: number): boolean
	local same = joinsOf(t)
	local function isSame(cx: number, cy: number): boolean
		if not level.inside(lvl, cx, cy) then
			return true
		end
		return same[layer[1 + cx + cy * lvl.width]] == true
	end
	local cases = {}
	local allFull = true
	for i, q in QUARTERS do
		local outV = not isSame(x, y + q.dy)
		local outH = not isSame(x + q.dx, y)
		local outD = not isSame(x + q.dx, y + q.dy)
		local case = if outV and outH then "outer" elseif outV then "vert" elseif outH then "horiz" elseif outD then "inner" else "full"
		cases[i] = case
		allFull = allFull and case == "full"
	end
	if allFull and t.sprites then
		return false
	end
	local name = t.autotile :: string
	-- Animated terrains (water, bogs): their 2 frames go in separate passes
	local frames = if t.frames then { { pass = `{prefix}Auto0`, name = `{name}0` }, { pass = `{prefix}Auto1`, name = `{name}1` } }
		else { { pass = `{prefix}Auto`, name = name } }
	for _, frame in frames do
		for i, q in QUARTERS do
			addQuad(builder, frame.pass, `autotile/{frame.name}_{cases[i]}_{q.name}`, vec.V2(x, y) + q.offset, 0.5)
		end
	end
	return true
end

local function addCell(builder: Builder, prefix: string, lvl: level.Level, layer: { number }, t: terrain.Terrain, x: number, y: number)
	local pos = vec.V2(x, y)
	if t.under then
		local under = terrain.byName[t.under]
		addQuad(builder, `{prefix}Base`, pickVariant(under.sprites :: { string }, x, y), pos, 1)
	end
	local drawn = t.autotile ~= nil and addAutotile(builder, prefix, lvl, layer, t, x, y)
	if not drawn and t.sprites then
		addQuad(builder, `{prefix}Base`, pickVariant(t.sprites, x, y), pos, 1)
	end
	if t.overlay then
		addQuad(builder, `{prefix}Over`, t.overlay, pos, 1)
	end
end

--- Shadows below the front of walls, and on the edges between heights: groups of 4 Vec2 for fastlist.drawRects
local function addShadows(points: { vec.Vec2 }, lvl: level.Level, x: number, y: number)
	local function rect(pos: vec.Vec2, size: vec.Vec2, color: vec4.Vec4)
		table.insert(points, pos)
		table.insert(points, size)
		table.insert(points, vec.V2(color.x, color.y))
		table.insert(points, vec.V2(color.z, color.w))
	end
	local wall = level.wallAt(lvl, x, y)
	if wall and wall.face and not level.wallAt(lvl, x, y + 1) and level.inside(lvl, x, y + 1) then
		rect(vec.V2(x, y + 1), vec.V2(1, 0.25), FACE_SHADOW)
	end
	if not wall then
		local h = level.heightAt(lvl, x, y)
		for _, side in { -1, 1 } do
			local nx = x + side
			if level.inside(lvl, nx, y) and not level.wallAt(lvl, nx, y) and level.heightAt(lvl, nx, y) < h then
				local edgeX = if side < 0 then x else x + 1 - 0.0625
				rect(vec.V2(edgeX, y), vec.V2(0.0625, 1), HEIGHT_EDGE)
			end
		end
	end
end

local function buildChunk(lvl: level.Level, cx: number, cy: number): Chunk
	local builder: Builder = {}
	local shadows: { vec.Vec2 } = {}
	for y = cy * CHUNK, math.min(lvl.height, (cy + 1) * CHUNK) - 1 do
		for x = cx * CHUNK, math.min(lvl.width, (cx + 1) * CHUNK) - 1 do
			local index = 1 + x + y * lvl.width
			local ground = terrain.get(lvl.ground[index])
			if ground then
				addCell(builder, "ground", lvl, lvl.ground, ground, x, y)
			end
			local wall = terrain.get(lvl.wall[index])
			if wall then
				addCell(builder, "wall", lvl, lvl.wall, wall, x, y)
			end
			local decoration = lvl.decor[index]
			if decoration then
				local prefix = if decoration.layer == "wall" then "wall" else "ground"
				if decoration.frames then
					for frame, sprite in decoration.frames do
						addQuad(builder, `{prefix}Decor{frame - 1}`, sprite, vec.V2(x, y), 1)
					end
				elseif decoration.sprites then
					addQuad(builder, `{prefix}Decor`, pickVariant(decoration.sprites, x, y), vec.V2(x, y), 1)
				end
			end
			addShadows(shadows, lvl, x, y)
		end
	end
	local chunk: Chunk = { passes = {}, shadows = fastlist.fromTable(shadows) }
	for name, entry in builder do
		chunk.passes[name] = { sheet = entry.sheet, quads = fastlist.fromTable(entry.points) }
	end
	return chunk
end

-- MARK: Drawing

--- Draw the terrain of the cells from `low` to `high` (included), in world coordinates
function module.draw(lvl: level.Level, low: vec.Vec2, high: vec.Vec2, time: number)
	local cache = module.cache
	-- Built again for a new level, when a bomb opened a wall (the version of the level changes), or new sprites
	if cache.level ~= lvl or cache.version ~= lvl.version or cache.tilesetVersion ~= sprites.tilesetVersion then
		cache.level = lvl
		cache.version = lvl.version
		cache.tilesetVersion = sprites.tilesetVersion
		cache.chunks = {}
	end
	local chunksX = math.ceil(lvl.width / CHUNK)
	local visible: { Chunk } = {}
	for cy = math.max(0, low.y // CHUNK), math.min(math.ceil(lvl.height / CHUNK) - 1, high.y // CHUNK) do
		for cx = math.max(0, low.x // CHUNK), math.min(chunksX - 1, high.x // CHUNK) do
			local index = cx + cy * chunksX
			local chunk = cache.chunks[index]
			if not chunk then
				chunk = buildChunk(lvl, cx, cy)
				cache.chunks[index] = chunk
			end
			table.insert(visible, chunk)
		end
	end
	local frame = math.floor(time * 2) % 2
	for _, pass in PASSES do
		if pass.frame == nil or pass.frame == frame then
			for _, chunk in visible do
				local list = chunk.passes[pass.name]
				if list then
					list.quads:drawTileQuads(list.sheet)
				end
			end
		end
	end
	for _, chunk in visible do
		chunk.shadows:drawRects()
	end
end

return module
