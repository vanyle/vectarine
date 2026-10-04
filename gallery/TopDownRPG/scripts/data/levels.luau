--[[
The levels of the Explorium, by id. Each level is a module of data/maps/ (see level.luau for the format).

The first floor, the Oasis:
- sinkhole: where the explorer falls
- camp: Wren's camp, the safe place of the oasis
- fern_hollow: a lush cave with water, snakes and Click the ant
- dark_hollow: a cave without light, where the eternal lantern waits
- colonnade: the first ruins of the sea people, with Dr. Ilse and Unit-7
- mole_den: the den of the Great Mole, and the stairs down

The second floor, the Desert:
- sunken_steps: the bottom of the stairs, a dead camp
- anthill: the colony of the ants, the safe place of the desert
- dry_canals: the dry irrigation of the sea people, and the gate of the Great Pump
- bazaar: a ruined market, Mara, and the council chamber
- pumping_station: inside the Great Pump, its three valves
- storm_dunes: dunes swept by sandstorms, and a hidden spring
- warden_hall: the hall of the Sun Warden, boss of the desert
- howling_gorge: an endless storm, and the stairs down

The third floor, the Wetlands:
- mire_edge: the bottom of the stairs: lily pads, a bog, a green pool
- nettle_clearing: the safe place of the wetlands, and Nettle's hut (nettle_hut, inside)
- hospital_grounds, house_of_healing: the gardens and the rooms of the hospital of the sea people
- drowned_docks: the harbor, the bell of the ferry, the storeroom of the smugglers
- great_mire: a lake of islands; frog_isle: the island of the Frog King, boss of the wetlands
- toxic_fen: the green lake, and the stairs down

The fourth floor, the Frosted Frontier:
- frozen_stair: the bottom of the stairs, the first braziers, the first ice
- rimehold: the village of the Frostborn, the safe place of the frontier, and the elder's lodge (elder_lodge, inside)
- the_glare: a frozen lake, and the icefall up to the fort
- ice_caves: the puzzles of the ice blocks, Tamsin, the hot spring
- old_watch, sleepers_barracks: the fort of the Watch, and its barracks
- watch_hall: the hall of the Last Watchman, boss of the frontier
- whiteout: a field of killing frost, the Cold Lock, and the stairs down
]]
const loader = require("@vectarine/loader")

export type LevelData = {
	name: string,
	subtitle: string?,
	--- The ground of "." cells (a terrain of data/terrain.luau), dirt by default
	floor: string?,
	darkness: number?,
	lights: { { number } }?,
	--- A beam of daylight falling from a hole in the ceiling: x, y of the cell it lights
	sunbeam: { number }?,
	legend: { [string]: any }?,
	links: { [string]: { level: string, entry: string } }?,
	map: { string },
	heights: { string }?,
	--- The floor of the Explorium the level is on (1 when not set)
	depth: number?,
	--- A style of data/legend.luau, that changes the meaning of some symbols ("desert")
	style: string?,
	--- How fast the player gets thirsty, compared to the oasis (1 when not set)
	heat: number?,
	--- Sandstorms: `calm` and `duration` in seconds (or `always`), `wind` the direction, `strength` its push
	storm: { calm: number?, duration: number?, always: boolean?, wind: { number }, strength: number }?,
	--- Snow falling over the level, from 0 (none) to 1 (a blizzard): only drawn, the cold comes from the ground
	snowfall: number?,
	--- Decorations, rows of the same size as `map` (see data/decor.luau)
	decor: { string }?,
	--- Parts of the level without light: x1, y1, x2, y2 (cells, included) and their darkness (see lighting.luau)
	darkAreas: { { number } }?,
}

const module = loader.init() or {}

module.levels = {
	sinkhole = require("./maps/sinkhole"),
	camp = require("./maps/camp"),
	fern_hollow = require("./maps/fern_hollow"),
	dark_hollow = require("./maps/dark_hollow"),
	colonnade = require("./maps/colonnade"),
	mole_den = require("./maps/mole_den"),
	sunken_steps = require("./maps/sunken_steps"),
	anthill = require("./maps/anthill"),
	dry_canals = require("./maps/dry_canals"),
	bazaar = require("./maps/bazaar"),
	pumping_station = require("./maps/pumping_station"),
	storm_dunes = require("./maps/storm_dunes"),
	warden_hall = require("./maps/warden_hall"),
	howling_gorge = require("./maps/howling_gorge"),
	mire_edge = require("./maps/mire_edge"),
	nettle_clearing = require("./maps/nettle_clearing"),
	nettle_hut = require("./maps/nettle_hut"),
	hospital_grounds = require("./maps/hospital_grounds"),
	house_of_healing = require("./maps/house_of_healing"),
	drowned_docks = require("./maps/drowned_docks"),
	great_mire = require("./maps/great_mire"),
	frog_isle = require("./maps/frog_isle"),
	toxic_fen = require("./maps/toxic_fen"),
	frozen_stair = require("./maps/frozen_stair"),
	rimehold = require("./maps/rimehold"),
	elder_lodge = require("./maps/elder_lodge"),
	the_glare = require("./maps/the_glare"),
	ice_caves = require("./maps/ice_caves"),
	old_watch = require("./maps/old_watch"),
	sleepers_barracks = require("./maps/sleepers_barracks"),
	watch_hall = require("./maps/watch_hall"),
	whiteout = require("./maps/whiteout"),
} :: { [string]: LevelData }

--- The names of the floors of the Explorium, from the top
module.floorNames = { "The Oasis", "The Desert", "The Wetlands", "The Frosted Frontier", "The Factory-City", "The Depth" }

--- The levels in the order of the journey, for the maps of the notebook
module.order = {
	"sinkhole",
	"camp",
	"fern_hollow",
	"dark_hollow",
	"colonnade",
	"mole_den",
	"sunken_steps",
	"anthill",
	"dry_canals",
	"bazaar",
	"pumping_station",
	"storm_dunes",
	"warden_hall",
	"howling_gorge",
	"mire_edge",
	"nettle_clearing",
	"nettle_hut",
	"hospital_grounds",
	"house_of_healing",
	"drowned_docks",
	"great_mire",
	"frog_isle",
	"toxic_fen",
	"frozen_stair",
	"rimehold",
	"elder_lodge",
	"the_glare",
	"ice_caves",
	"old_watch",
	"sleepers_barracks",
	"watch_hall",
	"whiteout",
}

return module
