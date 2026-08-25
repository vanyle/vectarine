--[[
The overworld: the rooms you walk through between the puzzles.
The camera shows one room at a time and scrolls when the player walks through an exit.

Each room is a map of 11 rows of 16 characters. See legend.luau for the meaning of each character.
To connect two rooms, leave the same tiles walkable on both sides of their shared border.

The puzzles are in levels.luau. They are entered through the ladders of the "mine" room.
]]
const legend = require("./legend")
const loader = require("@vectarine/loader")

export type Room = {
	--- The two names shown when entering the room: "Plouneuc" / "Village"
	name: string,
	subtitle: string,
	--- Terrain used for the "." character
	floor: string,
	--- Terrain used for the "#" character
	border: string,
	map: { string },
	legend: { [string]: legend.LegendEntry }?,
}

const module = loader.init() or {}

module.ROOM_WIDTH = 16
module.ROOM_HEIGHT = 11

--- Position of the rooms in the overworld. "" means that there is no room there.
module.layout = {
	{ "glade", "town", "mine" },
}

--- Where a new game starts
module.startRoom = "glade"

module.rooms = {
	glade = {
		name = "Morgane's Glade",
		subtitle = "Midnight",
		floor = "grass",
		border = "tree",
		map = {
			"################",
			"#..............#",
			"#..*........*..#",
			"#..............#",
			"#..............#",
			"#......@1....,,,",
			"#............,,,",
			"#..*........*..#",
			"#..............#",
			"#..............#",
			"################",
		},
		legend = {
			["1"] = { npc = "fairy" },
		},
	},

	town = {
		name = "Plouneuc",
		subtitle = "Village",
		floor = "grass",
		border = "tree",
		map = {
			"################",
			"#UUU...VVV..UUU#",
			"#UUU...VVV..UUU#",
			"#HDH...SKS..HOH#",
			"#..1...SES.....#",
			",,,,,,,,,,,,,,,,",
			",,,,,,,,,,,,,,,,",
			"#.VVV....2...*.#",
			"#.VVV..........#",
			"#.IES.3..*.....#",
			"################",
		},
		legend = {
			["1"] = { npc = "gwen" },
			["2"] = { npc = "lomig" },
			["3"] = {
				sign = {
					"PLOUNEUC. Population: 212. Time: midnight.",
					"Someone scratched a word below: STILL.",
				},
			},
		},
	},

	mine = {
		name = "The Old Mine",
		subtitle = "Choose a ladder",
		floor = "cave_floor",
		border = "wall",
		map = {
			"################",
			"#1#2#3#4#5#6#7##",
			"#.#a#b#c#d#e#f##",
			"#.............##",
			"#..........m.o*#",
			"..............##",
			"........y......#",
			"#..............#",
			"#g#h#i#j#k#l#n##",
			"#8#9#0#&#%#$#?##",
			"################",
		},
		legend = {
			-- Ladders to the levels (see levels.luau)
			["1"] = { ladder = 1 },
			["2"] = { ladder = 2 },
			["3"] = { ladder = 3 },
			["4"] = { ladder = 4 },
			["5"] = { ladder = 5 },
			["6"] = { ladder = 6 },
			["7"] = { ladder = 7 },
			["8"] = { ladder = 8 },
			["9"] = { ladder = 9 },
			["0"] = { ladder = 10 },
			["&"] = { ladder = 11 },
			["%"] = { ladder = 12 },
			["$"] = { ladder = 13 },
			["?"] = { ladder = 14 },
			["*"] = { ladder = 15 },
			-- Gates that open once enough levels are finished
			["a"] = { gate = "levels", count = 1 },
			["b"] = { gate = "levels", count = 2 },
			["c"] = { gate = "levels", count = 3 },
			["d"] = { gate = "levels", count = 4 },
			["e"] = { gate = "levels", count = 5 },
			["f"] = { gate = "levels", count = 6 },
			["g"] = { gate = "levels", count = 7 },
			["h"] = { gate = "levels", count = 8 },
			["i"] = { gate = "levels", count = 9 },
			["j"] = { gate = "levels", count = 10 },
			["k"] = { gate = "levels", count = 11 },
			["l"] = { gate = "levels", count = 12 },
			["n"] = { gate = "levels", count = 13 },
			["o"] = { gate = "levels", count = 14 },
			["y"] = { npc = "yann" },
			["m"] = { npc = "fairy_mine" },
		},
	},
} :: { [string]: Room }

return module
