--[[
Sprites are drawn from spritesheets (an image + a tileset).

A sprite is named "sheet/tile_type", for example "town/tree_green" or "dungeon/hero".
The tile types are defined inside the .tsx files of the textures/ folder. You can edit them with Tiled
(select a tile and set its "Class") or with a text editor: `<tile id="16" type="tree_green"/>`.

To add a new sprite, add a `<tile id="..." type="my_name"/>` line in the corresponding .tsx file
and use "sheet/my_name" in your code or data files.
]]
const image = require("@vectarine/image")
const loader = require("@vectarine/loader")
const resource = require("@vectarine/resource")
const tile = require("@vectarine/tile")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

export type SpriteName = string

type Sheet = {
	tileset: tile.TilesetResource,
	image: image.ImageResource,
	tiles: image.ImageWithTileset,
}

local function loadSheet(name: string, antialiasing: boolean): Sheet
	const tileset = loader.loadTileset(`textures/{name}.tsx`)
	-- Pixel art needs antialiasing = false to stay crisp when scaled up.
	const img = loader.loadImage(`textures/{name}.png`, antialiasing)
	return { tileset = tileset, image = img, tiles = img:withTileset(tileset) }
end

local sheets: { [string]: Sheet } = {}
sheets.town = loadSheet("town-tileset", false)
sheets.dungeon = loadSheet("dungeon-tileset", false)
sheets.farm = loadSheet("farm-tileset", false)
-- Sprites drawn for this game in the same style (spikes, plates, gears, ...)
sheets.extra = loadSheet("extra-tileset", false)
sheets.icons = loadSheet("game-icons", true)
module.sheets = sheets

--- Resources to wait for before drawing anything (used by the splash screen).
function module.resources(): { resource.Resource }
	local list: { resource.Resource } = {}
	for _, sheet in module.sheets do
		table.insert(list, sheet.tileset :: any)
		table.insert(list, sheet.image :: any)
	end
	return list
end

type Resolved = { tiles: image.ImageWithTileset, tile: string }
local resolvedCache: { [string]: Resolved } = {}

--- Turn "sheet/tile" into the objects needed to draw it. Results are cached as this is called a lot.
local function resolve(name: SpriteName): Resolved
	local cached = resolvedCache[name]
	if cached then
		return cached
	end
	local separator = string.find(name, "/", 1, true)
	local sheetName = if separator then string.sub(name, 1, separator - 1) else ""
	local tileName = if separator then string.sub(name, separator + 1) else ""
	local sheet = module.sheets[sheetName]
	assert(sheet and tileName ~= "", `Invalid sprite name '{name}', expected "sheet/tile_type"`)
	local result: Resolved = { tiles = sheet.tiles, tile = tileName :: string }
	resolvedCache[name] = result
	return result
end

--- Draw the sprite into a quad. `corners` are where the bottom-left, bottom-right, top-right and top-left
--- corners of the image go (the order expected by drawTileQuad).
--- Rotating the sprite is just shifting which corner of the image goes to which corner of the quad.
local function drawInQuad(r: Resolved, corners: { vec.Vec2 }, quarterTurns: number?, tint: vec4.Vec4?)
	local turns = (quarterTurns or 0) % 4
	local function corner(i: number): vec.Vec2
		return corners[((i - 1 - turns) % 4) + 1]
	end
	r.tiles:drawTileQuad(r.tile, corner(1), corner(2), corner(3), corner(4), tint)
end

--- Draw a sprite in screen space (OpenGL coordinates), with `pos` as the bottom-left corner.
--- `quarterTurns` rotates the sprite clockwise.
function module.draw(name: SpriteName, pos: vec.Vec2, size: vec.Vec2, tint: vec4.Vec4?, quarterTurns: number?)
	local r = resolve(name)
	if not quarterTurns or quarterTurns % 4 == 0 then
		r.tiles:drawTile(r.tile, pos, size, tint)
		return
	end
	drawInQuad(r, { pos, pos + vec.V2(size.x, 0), pos + size, pos + vec.V2(0, size.y) }, quarterTurns, tint)
end

export type WorldDrawOptions = {
	tint: vec4.Vec4?,
	flipX: boolean?,
	--- Number of clockwise quarter turns (0 to 3).
	quarterTurns: number?,
	--- Scale around the center of the tile. Defaults to 1.
	scale: number?,
}

--- Draw a sprite in world space, where 1 unit is 1 tile and the y axis points down (like the rows of a level).
--- `pos` is the top-left corner of the tile (it can be fractional for smooth movement).
function module.drawInWorld(name: SpriteName, pos: vec.Vec2, options: WorldDrawOptions?)
	local r = resolve(name)
	local opts: WorldDrawOptions = options or {}
	local half = (opts.scale or 1) * 0.5
	local center = pos + vec.V2(0.5, 0.5)
	-- drawTileQuad takes where the bottom-left, bottom-right, top-right and top-left corners of the image go.
	-- In world space, y points down, so "bottom" is +y.
	local corners = {
		center + vec.V2(-half, half),
		center + vec.V2(half, half),
		center + vec.V2(half, -half),
		center + vec.V2(-half, -half),
	}
	if opts.flipX then
		corners = { corners[2], corners[1], corners[4], corners[3] }
	end
	drawInQuad(r, corners, opts.quarterTurns, opts.tint)
end

return module
