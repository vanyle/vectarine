--[[
The barracks of the Old Watch, where the Watch sleeps in the ice, in rows, as ordered. Corporal Vey still stands at his
post, in the dream (a dream character: only dreamers see him and talk to him): he lost the names of his four men, carved
in the dream all over the frontier. Across a chasm, the armory of the Watch: the bridge is only there for those who
dream of it (a dream bridge, and the dream rune that tells of it).
]]
const legend = require("../legend")

return {
	name = "The Sleepers' Barracks",
	subtitle = "The Frosted Frontier",
	depth = 4,
	style = "frost",
	floor = "rime_floor",
	darkness = 0.65,
	legend = {
		f = { prop = "frozen_soldier" },
		v = { npc = "vey", data = { dream = true } },
		b = { ground = "chasm", prop = "dream_bridge" },
		o = { prop = "dream_rune", data = { lore = "dream_armory" } },
		k = { prop = "weapon_rack" },
		c = { prop = "chest", data = { items = { { "watch_halberd", 1 }, { "bomb", 4 }, { "nettle_tonic", 2 } } } },
		h = { monster = "shade" },
		m = { monster = "dream_moth" },
	} :: { [string]: legend.LegendEntry },
	links = {
		["7"] = { level = "old_watch", entry = "7" },
	},
	map = {
		"RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
		"RBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBR",
		"RBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBR",
		"R............h........____R......R",
		"R.....................____R.k.k..R",
		"R..f.f.f.f.f..........____R......R",
		"R................v....____R......R",
		"R.....................____R.....kR",
		"R....................o____R......R",
		"R..f.f.f.f.f..........____R......R",
		"R.....................bbbb...h...R",
		"R..............h......____R......R",
		"R.....................____R......R",
		"R..f.f.f.f.f..........____R......R",
		"R.................m...____R......R",
		"R.....................____R...c..R",
		"R.....................____R......R",
		"R........m....G....C..____R.!....R",
		"R.....................____R......R",
		"R.....................____R......R",
		"RRRRRRRRRRRRRRRR7RRRRRRRRRRRRRRRRR",
		"RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
	},
	decor = {
		"..................................",
		"......I......I.I...I.I.....II.....",
		"..................................",
		"..k...............................",
		".b............k...................",
		"...............................b..",
		".....k....k.......................",
		"..................................",
		"..................................",
		"..................................",
		"..................................",
		"............k.....................",
		"...................b..............",
		"..................................",
		"............k.b.k...k.............",
		"..................................",
		"...............k..................",
		"..........b...................k.b.",
		"..............k....b.........k....",
		".................k..k......k......",
		"..................................",
		"..................................",
	},

}
