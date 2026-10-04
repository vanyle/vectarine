--[[
Decorations: the second layer of a level map (`decor` in data/maps/, rows of the same size as `map`). They make places
feel alive and have no gameplay: nothing to pick up, nothing to use, no [E] prompt.

- `layer = "ground"`: drawn flat on the ground, under everything (flowers, tufts, pebbles, rubble...)
- `layer = "wall"`: drawn on the front of a wall (moss, roots, icicles; waterfalls are animated: `frames`)
- `prop`: a decoration that becomes a prop of the level: big ones block the way like rocks (a fallen trunk, a stump),
  glowing ones light the dark (glowing mushrooms). `partner` is the prop of the next cell to the right (the two
  halves of a fallen trunk).

"." is nothing. One sprite of `sprites` is picked for each cell, so that repeated decorations vary.
Decorations only show where they make sense when the level is built: things on the ground on walkable ground
(`water` ones on water instead: lilies), things on walls on walls. So when the canals of the desert fill with water,
or a bomb opens a cracked wall, what was on them goes away.
The decorations are drawn with the terrain (see terrain_view.luau): they cost nothing while playing.
]]
const loader = require("@vectarine/loader")

export type Decoration = {
	sprites: { string }?,
	frames: { string }?,
	layer: ("ground" | "wall")?,
	--- Lies on water (a water lily), instead of the ground
	water: boolean?,
	prop: string?,
	partner: string?,
}

const module = loader.init() or {}

local function ground(...: string): Decoration
	return { sprites = { ... }, layer = "ground" }
end

local function wall(...: string): Decoration
	return { sprites = { ... }, layer = "wall" }
end

module.characters = {
	-- On the ground
	r = ground("world/flowers_red"),
	y = ground("world/flowers_yellow"),
	w = ground("world/flowers_white"),
	t = ground("world/tuft_0", "world/tuft_1"),
	p = ground("world/pebbles_0", "world/pebbles_1"),
	l = ground("world/leaves"),
	c = ground("world/clover"),
	m = ground("world/mushrooms_small"),
	i = ground("world/twigs"),
	s = ground("world/ripples"),
	d = ground("world/desert_flower"),
	h = ground("world/shards"),
	j = ground("world/dry_twigs"),
	a = ground("world/cattails"),
	o = ground("world/moss"),
	u = { sprites = { "world/water_lily" }, layer = "ground", water = true },
	n = ground("world/snow_tuft"),
	e = ground("world/ice_shards"),
	f = ground("world/frost_flower"),
	b = ground("world/rubble"),
	k = ground("world/floor_crack"),
	g = ground("world/fern"),
	q = ground("world/tall_grass"),
	x = ground("world/mushrooms"),
	z = ground("world/reeds"),
	v = ground("world/dry_bush"),
	Y = ground("world/bones"),
	Z = ground("world/frozen_grass"),
	-- On the front of walls
	M = wall("world/moss_face"),
	R = wall("world/roots_face"),
	C = wall("world/crack_face"),
	I = wall("world/icicles_face"),
	V = wall("world/ivy_face"),
	F = { frames = { "world/waterfall_0", "world/waterfall_1" }, layer = "wall" },
	-- Big decorations, solid
	L = { prop = "log_l", partner = "log_r" },
	G = { prop = "frozen_log_l", partner = "frozen_log_r" },
	H = { prop = "mossy_log_l", partner = "mossy_log_r" },
	S = { prop = "stump" },
	T = { prop = "dead_tree" },
	O = { prop = "snowy_rock" },
	-- Plants and rocks at the foot of the walls, solid
	U = { prop = "bush" },
	K = { prop = "rock" },
	Q = { prop = "boulder" },
	P = { prop = "cactus" },
	X = { prop = "snow_bush" },
	N = { prop = "willow" },
	E = { prop = "pine" },
	-- Things that glow: they light the dark caves (glowing mushrooms and moss are flat, crystals are solid)
	A = { prop = "glowcap" },
	B = { prop = "glowmoss" },
	W = { prop = "frost_crystal" },
	D = { prop = "sun_crystal" },
} :: { [string]: Decoration }

return module
