--[[
Nettle's Hut: inside, Nettle the witch ant brews her remedies. Her cauldron, her shelves of jars, a quantum chest.
]]
const legend = require("../legend")

return {
	name = "Nettle's Hut",
	subtitle = "The Wetlands",
	depth = 3,
	style = "swamp",
	floor = "planks",
	darkness = 0.3,
	legend = {
		n = { npc = "nettle" },
		c = { prop = "cauldron" },
		s = { prop = "shelf" },
	} :: { [string]: legend.LegendEntry },
	links = {
		["3"] = { level = "nettle_clearing", entry = "3" },
	},
	map = {
		"[[[[[[[[[[[[[[[[",
		"[[]]]]]]]]]]]][[",
		"[[]]]]]]]]]]]][[",
		"[[.ss.......s.[[",
		"[[.....n......[[",
		"[[........c...[[",
		"[[U..!........[[",
		"[[............[[",
		"[[.........U..[[",
		"[[.E........Q.[[",
		"[[............[[",
		"[[[[[[[33[[[[[[[",
	},
	decor = {
		"................",
		"......M......R..",
		"...M............",
		"..k.............",
		"................",
		"................",
		".........b...b..",
		"...........b....",
		"................",
		"................",
		"................",
		"................",
	},

}
