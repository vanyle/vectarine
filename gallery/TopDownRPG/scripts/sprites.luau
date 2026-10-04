--[[
Sprites are drawn from spritesheets (an image + a tileset).

A sprite is named "sheet/tile_type", for example "world/bush" or "characters/player_down_0".
The tile types are defined inside the .tsx files of the textures/ folder. They are generated, with the images,
by tools/art/make_art.py: change the drawings there and run the script again.

Sheets:
- world: terrain and props (16x16)
- autotile: quarters of the autotiled terrains (8x8), see world_view.luau
- characters: the player, creatures, friendly characters and combat effects (16x16)
- items: item icons (16x16)
- bosses: big creatures (32x32)
- icons: menu icons
]]
const event = require("@vectarine/event")
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
sheets.world = loadSheet("world", false)
sheets.autotile = loadSheet("autotile", false)
sheets.characters = loadSheet("characters", false)
sheets.items = loadSheet("items", false)
sheets.bosses = loadSheet("bosses", false)
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

--- Tile ids of each sheet by tile type, read from the tilesets once they are loaded
local tileIds: { [string]: { [string]: number } } = {}

--- Increases every time a tileset is (re)loaded: tile ids may have changed, so anything that kept them
--- (the terrain of terrain_view.luau) must be computed again.
module.tilesetVersion = module.tilesetVersion or 0

event.getResourceLoadedEvent():on("spritesTilesets", function(id: number)
	for name, sheet in module.sheets do
		if sheet.tileset:getId() == id then
			tileIds[name] = nil
			module.tilesetVersion += 1
		end
	end
end)

--- The sheet of a sprite and the id of its tile, to draw many sprites at once with fastlists.
function module.tileOf(name: SpriteName): (image.ImageWithTileset, number)
	local separator = string.find(name, "/", 1, true) :: number
	local sheetName = string.sub(name, 1, separator - 1)
	local sheet = module.sheets[sheetName]
	local ids = tileIds[sheetName]
	if not ids then
		ids = {}
		for _, id in sheet.tileset:getTiles() do
			local info = sheet.tileset:getTile(id)
			if info then
				ids[info.type] = id
			end
		end
		tileIds[sheetName] = ids
	end
	local id = ids[string.sub(name, separator + 1)]
	assert(id, `Unknown sprite '{name}'`)
	return sheet.tiles, id
end

--- Draw a sprite in screen space (OpenGL coordinates), with `pos` as the bottom-left corner.
function module.draw(name: SpriteName, pos: vec.Vec2, size: vec.Vec2, tint: vec4.Vec4?)
	local r = resolve(name)
	r.tiles:drawTile(r.tile, pos, size, tint)
end

export type WorldDrawOptions = {
	tint: vec4.Vec4?,
	flipX: boolean?,
	--- Size in tiles (1 for 16x16 sprites, 2 for 32x32 sprites). Defaults to 1.
	size: number?,
	--- Rotation in radians, around the center of the sprite
	angle: number?,
	--- The part of the sprite drawn, from its top: 1 draws all of it (the default), 0.5 its upper half.
	--- Something sinking in mud is cut at the mud line. Not combined with `angle`.
	visible: number?,
}

--- Draw a sprite in world space, where 1 unit is 1 tile and the y axis points down.
--- `pos` is the top-left corner of the sprite (it can be fractional for smooth movement).
function module.drawInWorld(name: SpriteName, pos: vec.Vec2, options: WorldDrawOptions?)
	local r = resolve(name)
	local opts: WorldDrawOptions = options or {}
	local size = opts.size or 1
	local half = size / 2
	local center = pos + vec.V2(half, half)
	-- Corners of the image, before rotation: bottom-left, bottom-right, top-right, top-left.
	-- In world space y points down, so "bottom" is +y.
	local offsets = { vec.V2(-half, half), vec.V2(half, half), vec.V2(half, -half), vec.V2(-half, -half) }
	local visible = opts.visible
	if visible and visible < 1 then
		-- Only the upper part of the tile, on the upper part of the destination
		local bottom = -half + size * math.max(0, visible)
		offsets[1] = vec.V2(-half, bottom)
		offsets[2] = vec.V2(half, bottom)
		if opts.flipX then
			offsets = { offsets[2], offsets[1], offsets[4], offsets[3] }
		end
		local _, id = module.tileOf(name)
		local separator = string.find(name, "/", 1, true) :: number
		local sheet = module.sheets[string.sub(name, 1, separator - 1)]
		local info = sheet.tileset:getTile(id)
		if not info then
			return
		end
		-- The size of a tile as a ratio of the image (getTileRatio gives the number of tiles per row and column)
		local tiles = sheet.tileset:getTileRatio()
		local ratio = vec.V2(1 / tiles.x, 1 / tiles.y)
		sheet.image:drawPart(
			center + offsets[1],
			center + offsets[2],
			center + offsets[3],
			center + offsets[4],
			info.pos,
			vec.V2(ratio.x, ratio.y * math.max(0, visible)),
			opts.tint
		)
		return
	end
	if opts.flipX then
		offsets = { offsets[2], offsets[1], offsets[4], offsets[3] }
	end
	local angle = opts.angle
	if angle and angle ~= 0 then
		local c, s = math.cos(angle), math.sin(angle)
		for i, o in offsets do
			offsets[i] = vec.V2(o.x * c - o.y * s, o.x * s + o.y * c)
		end
	end
	r.tiles:drawTileQuad(
		r.tile,
		center + offsets[1],
		center + offsets[2],
		center + offsets[3],
		center + offsets[4],
		opts.tint
	)
end

--- Draw a quarter of a tile (an 8x8 sprite of the autotile sheet) in world space.
function module.drawQuarter(name: SpriteName, pos: vec.Vec2)
	local r = resolve(name)
	r.tiles:drawTileQuad(
		r.tile,
		pos + vec.V2(0, 0.5),
		pos + vec.V2(0.5, 0.5),
		pos + vec.V2(0.5, 0),
		pos,
		nil
	)
end

return module
