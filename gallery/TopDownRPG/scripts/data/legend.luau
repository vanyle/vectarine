--[[
The legend maps each character of a level map (see data/levels/) to what it creates.

Conventions:
- symbols and uppercase letters are terrains and props, shared by every level
- lowercase letters are defined by each level, in its own `legend` (monsters, characters, chests and items)
- digits are entry points and exits: they are linked to other levels by the `links` of the level
- "@" is where a new game starts
- a level can have a `style` (`styles` below) that gives other meanings to some symbols: in the desert,
  "#" and "W" are sandstone, "," is deep sand, "*" and "&" are cacti...

An entry can define:
- `ground`: a terrain of data/terrain.luau. Defaults to the floor of the level.
- `wall`: a wall terrain on top of the ground
- `prop`: a prop of data/props.luau, with optional `data` (the text of a sign, the content of a chest...)
- `monster`: a creature of data/monsters.luau. `unique` names bosses, which never come back once killed
  (the flag "killed:<unique>" is set when they die). `group` names the guards of a chest: a chest with
  `data.guard = "<group>"` stays sealed while one of them is alive
- `npc`: a character of data/npcs.luau
- `item` (+ `count`): an item lying on the ground, picked up by walking on it
- `entry`: the cell is also an entry point and exit with this name, like a digit (the door of a hut)
]]
const loader = require("@vectarine/loader")

export type LegendEntry = {
	ground: string?,
	wall: string?,
	prop: string?,
	data: { [string]: any }?,
	monster: string?,
	unique: string?,
	group: string?,
	npc: string?,
	item: string?,
	count: number?,
	start: boolean?,
	entry: string?,
}

const module = loader.init() or {}

module.characters = {
	-- Grounds ("." is the floor of the level, usually dirt)
	["."] = {},
	[","] = { ground = "grass" },
	[":"] = { ground = "ruin_floor" },
	["~"] = { ground = "water" },
	["_"] = { ground = "chasm" },

	-- Walls
	["#"] = { wall = "rock" },
	["W"] = { wall = "rock_face" },
	["N"] = { wall = "roots" },
	["H"] = { wall = "ladder" },
	["V"] = { wall = "vines" },
	["Z"] = { wall = "cave_opening" },
	["R"] = { wall = "ruin" },
	["B"] = { wall = "ruin_face" },
	["A"] = { wall = "ruin_door" },

	-- Plants
	["*"] = { prop = "bush" },
	["&"] = { prop = "fruit_bush" },
	["%"] = { prop = "fern" },
	[";"] = { ground = "grass", prop = "tall_grass" },
	["!"] = { prop = "glowcap" },
	["^"] = { prop = "mushrooms" },
	["L"] = { ground = "water", prop = "lily_pad" },
	["M"] = { prop = "reeds" },

	-- Rocks and remains
	["O"] = { prop = "rock" },
	["0"] = { prop = "boulder" },
	["+"] = { prop = "bones" },
	["K"] = { prop = "skull" },

	-- Ruins
	["I"] = { ground = "ruin_floor", prop = "column" },
	["J"] = { prop = "column_broken" },
	["S"] = { prop = "statue_head" },
	["U"] = { prop = "pot" },

	-- Camp and machines
	["F"] = { prop = "campfire" },
	["Y"] = { prop = "tent" },
	["E"] = { prop = "bedroll" },
	["X"] = { prop = "crate" },
	["Q"] = { prop = "quantum_chest" },
	["P"] = { prop = "teleporter" },
	["D"] = { prop = "stairs" },

	-- The desert
	["-"] = { ground = "canal_bed" },
	["G"] = { prop = "cistern" },
	["T"] = { prop = "palm" },
	["$"] = { wall = "ruin_cracked" },
	["?"] = { wall = "ruin_face_cracked" },

	["@"] = { start = true },
} :: { [string]: LegendEntry }

module.styles = {
	desert = {
		["#"] = { wall = "sandstone" },
		["W"] = { wall = "sandstone_face" },
		["H"] = { wall = "sand_ladder" },
		["$"] = { wall = "sandstone_cracked" },
		["?"] = { wall = "sandstone_face_cracked" },
		[","] = { ground = "dune" },
		["*"] = { prop = "cactus" },
		["&"] = { prop = "cactus_fruit" },
		["%"] = { prop = "dry_bush" },
		["!"] = { prop = "sun_crystal" },
		["T"] = { prop = "palm_dead" },
	},
	swamp = {
		["#"] = { wall = "mossrock" },
		["W"] = { wall = "mossrock_face" },
		["H"] = { wall = "moss_ladder" },
		["_"] = { ground = "bog" },
		["-"] = { ground = "toxic" },
		["/"] = { ground = "miasma" },
		["="] = { ground = "planks" },
		["L"] = { ground = "water", prop = "lily_big" },
		["C"] = { ground = "water", prop = "lily_sink" },
		["!"] = { prop = "glowmoss" },
		["T"] = { prop = "willow" },
		["["] = { wall = "thatch" },
		["]"] = { wall = "log_wall" },
	},
	frost = {
		["."] = { ground = "rime_floor" },
		["#"] = { wall = "glacier" },
		["W"] = { wall = "glacier_face" },
		["H"] = { wall = "glacier_ladder" },
		["$"] = { wall = "glacier_cracked" },
		["?"] = { wall = "glacier_face_cracked" },
		["V"] = { wall = "icefall" },
		[","] = { ground = "snow" },
		["-"] = { ground = "ice" },
		["="] = { ground = "thin_ice" },
		["~"] = { ground = "cold_water" },
		["&"] = { ground = "dark_water" },
		["/"] = { ground = "veil" },
		[";"] = { ground = "frost" },
		["`"] = { ground = "hot_spring" },
		["["] = { wall = "snowbrick" },
		["]"] = { wall = "snowbrick_face" },
		["!"] = { prop = "frost_crystal" },
		["T"] = { prop = "pine" },
		["*"] = { prop = "snow_bush" },
		["%"] = { prop = "frozen_grass" },
		["L"] = { prop = "ice_block" },
		["C"] = { prop = "brazier" },
		["G"] = { prop = "dream_mirror" },
		["M"] = { prop = "snowdrift" },
	},
} :: { [string]: { [string]: LegendEntry } }

return module
