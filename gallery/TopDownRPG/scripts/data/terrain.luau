--[[
Terrain types: everything that is part of the level and does not move.

The level is stored in a tilemap with 2 layers:
- the GROUND layer (grass, paths, chasms, pressure plates, ...) is always present
- the OBJECTS layer (trees, walls, rocks, spikes, houses, ...) sits on top of the ground and is often empty

Each terrain type describes how it looks and how it behaves:

- `sprite`, or `variants`: a list of sprites picked pseudo-randomly per tile (see sprites.luau)
- `autotile`: pick the sprite from the neighboring tiles (see below)
- `tint`: color multiplied with the sprite (the water is a tinted stone floor!)
- `solid`: nobody can walk on it, arrows stop on it and you can't leap over it
- `flyable`: flying monsters can go over it
- `deadly`: walking on it kills you. "spikes" also kill monsters, "fall" (chasms) is avoided by monsters.
- `breaksWith`: the actions that destroy this object ("mine" for the pickaxe, "bomb" for explosions)
- `plate`: a pressure plate (see the gates in rules.luau)

Autotiling looks at the 4 neighbors of a tile that have the same terrain (or one listed in `connects`):
- `nine = { tl, t, tr, l, c, r, bl, b, br }`: like a 9-slice, for shapes at least 2 tiles wide.
  `inner = { tl, tr, br, bl }` are used for the inside corners of L shapes.
- `row = { single, left, middle, right }`: for things that are one tile high, like walls.
- `mask = { [n] = sprite }` with n = up(1) + right(2) + down(4) + left(8), for any shape.
  `fallback` is used when the mask is missing from the table.
]]
const loader = require("@vectarine/loader")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

export type Autotile = {
	nine: { string }?,
	inner: { string }?,
	row: { string }?,
	mask: { [number]: string }?,
	fallback: string?,
	connects: { string }?,
}

export type TerrainDefinition = {
	name: string,
	sprite: string?,
	variants: { string }?,
	autotile: Autotile?,
	tint: vec4.Vec4?,
	solid: boolean?,
	flyable: boolean?,
	deadly: ("spikes" | "fall")?,
	breaksWith: { [string]: boolean }?,
	plate: boolean?,
}

--- A terrain definition with its id, which is the number stored inside the tilemap.
export type Terrain = TerrainDefinition & { id: number }

local function nineSlice(prefix: string): { string }
	local result = {}
	for _, part in { "tl", "t", "tr", "l", "", "r", "bl", "b", "br" } do
		table.insert(result, if part == "" then prefix else `{prefix}_{part}`)
	end
	return result
end

--- Roofs are 2 tiles high: the middle row uses the bottom sprites.
local function roof(color: string): { string }
	local p = `town/roof_{color}`
	return { `{p}_tl`, `{p}_t`, `{p}_tr`, `{p}_bl`, `{p}_b`, `{p}_br`, `{p}_bl`, `{p}_b`, `{p}_br` }
end

local chasmSprites: { [number]: string } = {}
for mask = 0, 15 do
	chasmSprites[mask] = `extra/chasm_{mask}`
end

local definitions: { TerrainDefinition } = {
	-- MARK: Ground
	{ name = "grass", variants = { "town/grass", "town/grass", "town/grass", "town/grass_tufts" } },
	{ name = "flowers", sprite = "town/grass_flowers" },
	{
		name = "path",
		autotile = {
			nine = nineSlice("town/path"),
			inner = { "town/path_inner_tl", "town/path_inner_tr", "town/path_inner_br", "town/path_inner_bl" },
		},
	},
	{
		name = "water",
		solid = true,
		flyable = true,
		tint = vec4.createColor(0.35, 0.6, 1.0, 1),
		autotile = { nine = nineSlice("town/slab") },
	},
	{ name = "chasm", deadly = "fall", flyable = true, autotile = { mask = chasmSprites } },
	{ name = "soil", autotile = { row = { "farm/soil_single", "farm/soil_left", "farm/soil", "farm/soil_right" } } },
	{ name = "cave_floor", sprite = "dungeon/cave_floor" },
	{ name = "plate", sprite = "extra/plate", plate = true },

	-- MARK: Objects
	{ name = "tree", sprite = "town/tree_green", solid = true },
	{ name = "autumn_tree", sprite = "town/tree_orange", solid = true },
	{ name = "wall", sprite = "dungeon/cave_wall", solid = true },
	{ name = "rock", sprite = "farm/boulder", solid = true, breaksWith = { mine = true, bomb = true } },
	{ name = "cracked_wall", sprite = "extra/cracked_wall", solid = true, breaksWith = { bomb = true } },
	{ name = "spikes", sprite = "extra/spikes", deadly = "spikes" },
	{ name = "roof_red", solid = true, autotile = { nine = roof("red") } },
	{ name = "roof_blue", solid = true, autotile = { nine = roof("blue") } },
	-- Houses with red roofs have wooden walls, houses with blue roofs have stone walls.
	{
		name = "wood_wall",
		solid = true,
		autotile = {
			row = { "town/wall_wood", "town/wall_wood_l", "town/wall_wood", "town/wall_wood_r" },
			connects = { "wood_door", "wood_window" },
		},
	},
	{ name = "wood_door", sprite = "town/door_wood", solid = true },
	{ name = "wood_window", sprite = "town/window_wood", solid = true },
	{
		name = "stone_wall",
		solid = true,
		autotile = {
			row = { "town/wall_stone", "town/wall_stone_l", "town/wall_stone", "town/wall_stone_r" },
			connects = { "stone_door", "stone_window", "clock" },
		},
	},
	{ name = "stone_door", sprite = "town/door_stone", solid = true },
	{ name = "stone_window", sprite = "town/window_stone", solid = true },
	{ name = "clock", sprite = "extra/clock_wall", solid = true },
	{ name = "hay", sprite = "farm/hay", solid = true },
	{ name = "crop", variants = { "farm/corn", "farm/cabbage", "farm/tomato" }, solid = true },
	{ name = "mushrooms", sprite = "town/mushrooms" },
	{
		name = "fence",
		solid = true,
		autotile = {
			mask = {
				[2] = "town/fence_h_left",
				[8] = "town/fence_h_right",
				[10] = "town/fence_h",
				[4] = "town/fence_v_top",
				[1] = "town/fence_v_bottom",
				[5] = "town/fence_v",
				[6] = "town/fence_tl",
				[12] = "town/fence_tr",
				[3] = "town/fence_bl",
				[9] = "town/fence_br",
				[14] = "town/fence_t",
			},
			fallback = "town/fence_h",
		},
	},
}

--- Terrain types by id (the number stored inside the tilemap, 0 means "nothing") and by name.
module.byId = {} :: { [number]: Terrain }
module.byName = {} :: { [string]: Terrain }

for index, definition in definitions do
	local terrain = table.clone(definition) :: Terrain
	terrain.id = index
	module.byId[index] = terrain
	module.byName[terrain.name] = terrain
end

return module
