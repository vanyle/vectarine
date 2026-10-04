--[[
The lodge of Elder Hask, in Rimehold: a hearth to rest by, the quantum chest of the village, the beds of the family.
]]
const legend = require("../legend")

return {
	name = "The Elder's Lodge",
	subtitle = "The Frosted Frontier",
	depth = 4,
	style = "frost",
	floor = "planks",
	darkness = 0.3,
	legend = {
		h = { prop = "hearth" },
		k = { npc = "hask" },
	} :: { [string]: legend.LegendEntry },
	links = {
		["3"] = { level = "rimehold", entry = "3" },
	},
	map = {
		"[[[[[[[[[[[[[[[[[[",
		"[]]]]]]]]]]]]]]]][",
		"[]]]]]]]]]]]]]]]][",
		"[.......h........[",
		"[.E..k.......Q...[",
		"[................[",
		"[................[",
		"[.E..............[",
		"[..............X.[",
		"[...........U....[",
		"[..U...........X.[",
		"[................[",
		"[[[[[[[[3[[[[[[[[[",
	},
	decor = {
		"..................",
		"......I......I.I..",
		"..................",
		"..k...............",
		".b............k...",
		"..................",
		".....k....k.......",
		"..................",
		"..................",
		"...k..............",
		"..................",
		"............k.....",
		"..................",
	},

}
