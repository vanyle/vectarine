--[[
Terrains: what each cell of a level is made of.

A cell has a ground terrain (dirt, grass, water...) and optionally a wall terrain on top of it.
Walls are seen from slightly above: the top of a wall ("rock") and its front face ("rock_face") are separate terrains,
placed on separate cells of the map. Nobody can stand on a wall, except on ladders (and vines, with climbing claws).

Fields:
- `walkable`: characters can walk on it. `flyable`: flying creatures can cross it.
- `sprites`: variants of the 16x16 sprite (one is picked per cell), from the "world" sheet
- `autotile`: name of the autotile pieces (see world_view.luau): the edges of the terrain follow its neighbors.
  `joins` lists the other terrains that count as the same for the edges.
- `overlay`: a sprite drawn on top (a ladder on a rock face, for example)
- `water`: the player can fill their bottle here. `ladder`: connects height layers. `climbable`: needs climbing claws.
- `slow`: a speed multiplier for walking on it (deep sand, bogs). The sandstride anklets cancel it on `sand`.
- `frames`: the autotile is animated (water, bogs): its pieces are named "<autotile><frame>".
- `sailable`: the player crosses it with the ferry. `toxic`: it poisons who sails on it (unless they carry the bezoar).
- `bog`: it swallows who stands still on it (see swamp.luau). `miasma`: rotten air, only crossed with the beaked mask.
- `cracked`: a wall that a bomb can open (see combat.luau). `wet`: the ground it becomes once the Great Pump runs
  again (the canal beds of the desert fill with water).
- The frontier (see frost.luau and dream.luau): `cold` is the warmth lost per second standing on it, `slippery` ground
  keeps the player's momentum (ice), `thin` ice cracks under whoever stands on it, `freezing` water chills whoever
  sails on it or falls in, `warm` water warms whoever stands next to it (hot springs). Holes in the ice (cold_water)
  are full of slush: the ferry can't get through, only blocks of ice fill them. Open dark water can be sailed. `icefall` walls are climbed
  with the rime crampons. `veil` (frozen dreams) only lets dreamers through.
]]
const loader = require("@vectarine/loader")

export type Terrain = {
	name: string,
	id: number,
	walkable: boolean,
	flyable: boolean,
	sprites: { string }?,
	autotile: string?,
	joins: { string }?,
	--- Drawn below an autotiled terrain whose edges are transparent (grass over dirt)
	under: string?,
	overlay: string?,
	water: boolean?,
	ladder: boolean?,
	climbable: boolean?,
	--- True for the front face of walls: the floor below gets a shadow
	face: boolean?,
	slow: number?,
	sand: boolean?,
	cracked: boolean?,
	wet: string?,
	frames: number?,
	sailable: boolean?,
	toxic: boolean?,
	bog: boolean?,
	miasma: boolean?,
	cold: number?,
	slippery: boolean?,
	thin: boolean?,
	freezing: boolean?,
	warm: boolean?,
	icefall: boolean?,
	veil: boolean?,
}

const module = loader.init() or {}

local function variants(prefix: string, count: number): { string }
	local list = {}
	for i = 0, count - 1 do
		table.insert(list, `world/{prefix}_{i}`)
	end
	return list
end

module.list = {
	-- Grounds
	{ name = "dirt", walkable = true, flyable = true, sprites = variants("dirt", 4) },
	{ name = "grass", walkable = true, flyable = true, sprites = variants("grass", 4), autotile = "grass", under = "dirt" },
	{ name = "ruin_floor", walkable = true, flyable = true, sprites = variants("ruin_floor", 4) },
	{ name = "water", walkable = false, flyable = true, autotile = "water", frames = 2, water = true, sailable = true },
	{ name = "chasm", walkable = false, flyable = true, autotile = "chasm" },
	-- Walls
	{ name = "rock", walkable = false, flyable = false, autotile = "rock", joins = { "rock_face", "ladder", "vines", "roots", "cave_opening" } },
	{ name = "rock_face", walkable = false, flyable = false, sprites = variants("rock_face", 3), face = true },
	{ name = "ruin", walkable = false, flyable = false, autotile = "ruin", joins = { "ruin_face", "ruin_door", "ruin_cracked", "ruin_face_cracked" } },
	{ name = "ruin_face", walkable = false, flyable = false, sprites = variants("ruin_face", 3), face = true },
	{ name = "ladder", walkable = true, flyable = false, sprites = variants("rock_face", 3), overlay = "world/ladder", ladder = true, face = true },
	{ name = "vines", walkable = false, flyable = false, sprites = variants("rock_face", 3), overlay = "world/vines", climbable = true, face = true },
	{ name = "roots", walkable = false, flyable = false, sprites = variants("rock_face", 3), overlay = "world/roots", face = true },
	{ name = "cave_opening", walkable = true, flyable = true, sprites = variants("rock_face", 3), overlay = "world/cave_opening", face = true },
	{ name = "ruin_door", walkable = true, flyable = true, sprites = variants("ruin_face", 3), overlay = "world/ruin_door", face = true },
	-- The desert (new terrains go at the end of the list: their position is their id in the maps)
	{ name = "sand", walkable = true, flyable = true, sprites = variants("sand", 4) },
	{ name = "dune", walkable = true, flyable = true, autotile = "dune", under = "sand", slow = 0.6, sand = true },
	{ name = "canal_bed", walkable = true, flyable = true, autotile = "canal", wet = "water" },
	{
		name = "sandstone",
		walkable = false,
		flyable = false,
		autotile = "sandstone",
		joins = { "sandstone_face", "sand_ladder", "sandstone_cracked", "sandstone_face_cracked" },
	},
	{ name = "sandstone_face", walkable = false, flyable = false, sprites = variants("sandstone_face", 3), face = true },
	{ name = "sand_ladder", walkable = true, flyable = false, sprites = variants("sandstone_face", 3), overlay = "world/ladder", ladder = true, face = true },
	{ name = "sandstone_cracked", walkable = false, flyable = false, sprites = { "world/sandstone_top_cracked" }, cracked = true },
	{ name = "sandstone_face_cracked", walkable = false, flyable = false, sprites = { "world/sandstone_face_cracked" }, face = true, cracked = true },
	{ name = "ruin_cracked", walkable = false, flyable = false, sprites = { "world/ruin_top_cracked" }, cracked = true },
	{ name = "ruin_face_cracked", walkable = false, flyable = false, sprites = { "world/ruin_face_cracked" }, face = true, cracked = true },
	-- The wetlands
	{ name = "mud", walkable = true, flyable = true, sprites = variants("mud", 4) },
	{ name = "bog", walkable = true, flyable = true, autotile = "bog", frames = 2, under = "mud", slow = 0.45, bog = true },
	{ name = "toxic", walkable = false, flyable = true, autotile = "toxic", frames = 2, sailable = true, toxic = true },
	{ name = "miasma", walkable = true, flyable = true, sprites = variants("mud", 4), miasma = true },
	{ name = "planks", walkable = true, flyable = true, sprites = variants("planks", 2) },
	{ name = "clinic_floor", walkable = true, flyable = true, sprites = variants("clinic_floor", 2) },
	{ name = "mossrock", walkable = false, flyable = false, autotile = "mossrock", joins = { "mossrock_face", "moss_ladder" } },
	{ name = "mossrock_face", walkable = false, flyable = false, sprites = variants("mossrock_face", 3), face = true },
	{ name = "moss_ladder", walkable = true, flyable = false, sprites = variants("mossrock_face", 3), overlay = "world/ladder", ladder = true, face = true },
	{ name = "thatch", walkable = false, flyable = false, autotile = "thatch", joins = { "log_wall", "hut_door" } },
	{ name = "log_wall", walkable = false, flyable = false, sprites = variants("log_wall", 2), face = true },
	{ name = "hut_door", walkable = true, flyable = true, sprites = variants("log_wall", 2), overlay = "world/hut_door", face = true },
	-- The frontier
	{ name = "snow", walkable = true, flyable = true, autotile = "snow", under = "rime_floor", slow = 0.85, cold = 3.5 },
	{ name = "ice", walkable = true, flyable = true, sprites = variants("ice", 4), slippery = true, cold = 2.5 },
	{ name = "thin_ice", walkable = true, flyable = true, sprites = variants("thin_ice", 2), slippery = true, thin = true, cold = 2.5 },
	{ name = "cold_water", walkable = false, flyable = true, autotile = "coldwater", frames = 2, water = true, freezing = true },
	{ name = "frost", walkable = true, flyable = true, sprites = variants("frost", 4), slow = 0.75, cold = 14 },
	{ name = "rime_floor", walkable = true, flyable = true, sprites = variants("rime_floor", 4) },
	{ name = "hot_spring", walkable = false, flyable = true, autotile = "spring", frames = 2, water = true, warm = true },
	{ name = "veil", walkable = true, flyable = true, sprites = variants("rime_floor", 4), veil = true },
	{
		name = "glacier",
		walkable = false,
		flyable = false,
		autotile = "glacier",
		joins = { "glacier_face", "glacier_ladder", "icefall", "glacier_cracked", "glacier_face_cracked" },
	},
	{ name = "glacier_face", walkable = false, flyable = false, sprites = variants("glacier_face", 3), face = true },
	{ name = "glacier_ladder", walkable = true, flyable = false, sprites = variants("glacier_face", 3), overlay = "world/ladder", ladder = true, face = true },
	{ name = "icefall", walkable = false, flyable = false, sprites = { "world/icefall" }, icefall = true, face = true },
	{ name = "glacier_cracked", walkable = false, flyable = false, sprites = { "world/glacier_top_cracked" }, cracked = true },
	{ name = "glacier_face_cracked", walkable = false, flyable = false, sprites = { "world/glacier_face_cracked" }, face = true, cracked = true },
	{ name = "dark_water", walkable = false, flyable = true, autotile = "water", frames = 2, water = true, sailable = true, freezing = true },
	{ name = "snowbrick", walkable = false, flyable = false, autotile = "snowbrick", joins = { "snowbrick_face", "snow_door" } },
	{ name = "snowbrick_face", walkable = false, flyable = false, sprites = variants("snowbrick_face", 2), face = true },
	{ name = "snow_door", walkable = true, flyable = true, sprites = variants("snowbrick_face", 2), overlay = "world/snow_door", face = true },
	-- The roofs of the buildings of the sea people: red clay tiles, over walls of marble
	{ name = "roof_tiles", walkable = false, flyable = false, autotile = "roof_tiles", joins = { "ruin_face", "ruin_door" } },
} :: { Terrain }

module.byName = {} :: { [string]: Terrain }
for index, terrain in module.list do
	terrain.id = index
	module.byName[terrain.name] = terrain
end

--- The terrain of a tilemap id (0 = nothing)
function module.get(id: number): Terrain?
	return module.list[id]
end

return module
