--[[
The legend maps each character of a room map (see rooms.luau and levels.luau) to what it creates.

Conventions: symbols are ground, UPPERCASE letters are objects, lowercase letters are things that
live on a tile (monsters, items, targets, ...) and digits are defined by each room.

An entry can define:
- `ground`: a terrain name for the ground layer. "floor" means the floor of the room (grass, cave, ...)
- `object`: a terrain name for the object layer. "border" means the border of the room (trees, cave walls, ...)
- `monster` (+ `dir` for monsters that fly in a straight line): a monster from monsters.luau
- `pickup` (+ `amount`, `potion`): something collected by walking on it
- `gate`: a barrier that opens when its condition is true (see rules.luau). `spikes = true` makes it
  a spike trap instead: the spikes go down when the condition is true.
- `door`: a locked door, opened with a key
- `target`, `barrel`: things to shoot at
- `npc`, `sign`: characters to talk to and signs to read
- `ladder`: the entrance of a level (its number)
- `player`: where the player starts

Rooms can override any character with their own `legend` table.
]]
const loader = require("@vectarine/loader")

export type Direction = "up" | "down" | "left" | "right"
export type GateCondition = "plates" | "targets" | "monsters" | "levels"
export type PickupKind = "arrows" | "bombs" | "key" | "gear" | "potion"
export type Potion = "leap" | "slant" | "clear"

export type LegendEntry = {
	ground: string?,
	object: string?,
	monster: string?,
	dir: Direction?,
	pickup: PickupKind?,
	amount: number?,
	potion: Potion?,
	gate: GateCondition?,
	--- For "levels" gates: how many levels must be finished
	count: number?,
	spikes: boolean?,
	door: boolean?,
	target: boolean?,
	barrel: boolean?,
	npc: string?,
	sign: { string }?,
	ladder: number?,
	player: boolean?,
}

const module = loader.init() or {}

module.characters = {
	-- Ground
	["."] = { ground = "floor" },
	["*"] = { ground = "flowers" },
	[","] = { ground = "path" },
	["~"] = { ground = "water" },
	["_"] = { ground = "chasm" },
	["="] = { ground = "soil" },
	["+"] = { ground = "plate" },

	-- Objects
	["#"] = { object = "border" },
	["T"] = { object = "tree" },
	["A"] = { object = "autumn_tree" },
	["W"] = { object = "wall" },
	["^"] = { object = "spikes" },
	["R"] = { object = "rock" },
	["C"] = { object = "cracked_wall" },
	["U"] = { object = "roof_red" },
	["V"] = { object = "roof_blue" },
	["H"] = { object = "wood_wall" },
	["D"] = { object = "wood_door" },
	["O"] = { object = "wood_window" },
	["S"] = { object = "stone_wall" },
	["E"] = { object = "stone_door" },
	["I"] = { object = "stone_window" },
	["K"] = { object = "clock" },
	["F"] = { object = "fence" },
	["X"] = { object = "hay" },
	["Y"] = { object = "crop" },
	["M"] = { object = "mushrooms" },

	-- Gates and doors
	["G"] = { gate = "plates" },
	["Z"] = { gate = "plates", spikes = true },
	["Q"] = { gate = "targets" },
	["N"] = { gate = "monsters" },
	["L"] = { door = true },

	-- Creatures
	["@"] = { player = true },
	["r"] = { monster = "rat" },
	["b"] = { monster = "bat", dir = "right" },
	["v"] = { monster = "bat", dir = "down" },
	["s"] = { monster = "spider" },

	-- Things
	["k"] = { pickup = "key" },
	["a"] = { pickup = "arrows", amount = 1 },
	["q"] = { pickup = "bombs", amount = 1 },
	["g"] = { pickup = "gear" },
	["h"] = { pickup = "potion", potion = "leap" },
	["j"] = { pickup = "potion", potion = "slant" },
	["c"] = { pickup = "potion", potion = "clear" },
	["t"] = { target = true },
	["x"] = { barrel = true },
} :: { [string]: LegendEntry }

return module
