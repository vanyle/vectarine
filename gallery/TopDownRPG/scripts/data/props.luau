--[[
Props: things placed on the cells of a level. Plants, rocks, ruins, the camp, chests, characters...

Fields:
- `sprite`: from the "world" sheet. `frames` instead of `sprite` for animated props (frames per second: `fps`).
- `solid`: blocks characters. `flat`: drawn on the ground, below characters (a fern, a lily pad).
- `tall`: a sprite drawn on the cell above (the top of a column). `wide`: a sprite drawn on the cell to the right.
- `light`: radius of the light it gives, in tiles (see lighting in world_view.luau).
- `interact`: what [E] does near it (see interactions.luau), and `label` the word shown in the key prompts.
- `breakable`: hitting it breaks it (and it may drop something, see `drops`).
- `fruit`: it can be picked (`item`, `count`), and grows back after `regrow` seconds. `empty` is its sprite meanwhile.
- `openKind`: what it becomes once opened (a gate, a valve...). It opens when the flag "open:<id>" is set, where
  <id> is its `data.id`, or the `data.opens` of a sun eye (see switches.luau).
- `target`: an arrow that hits it opens what it guards (`data.opens`).
- `floor`: it carries the player over water (a lily pad). `sinks`: it sinks a moment after someone steps on it, and
  comes back up later (see swamp.luau).
- A prop with `data.showIf` only appears once that flag is set (the ferry, called by the bell).
- The frontier: `litKind` is what a brazier becomes once lit (the flag "lit:<key>" remembers it). `pushable` props
  (ice blocks) slide when the player walks into them (see blocks.luau). `plate`: a rune plate, pressed by an ice block
  resting on it. `dream`: only there for dreamers (dream bridges, dream writing, see dream.luau). `warmth`: radius of
  the heat it gives (fires, braziers, see frost.luau). `hides`: drawn above the player (a snowdrift hiding a passage).
]]
const loader = require("@vectarine/loader")

export type PropKind = {
	sprite: string?,
	frames: { string }?,
	fps: number?,
	solid: boolean,
	flat: boolean?,
	tall: string?,
	wide: string?,
	light: number?,
	interact: string?,
	label: string?,
	breakable: boolean?,
	drops: { { item: string, chance: number, count: number? } }?,
	fruit: { item: string, count: number, regrow: number, empty: string }?,
	openKind: string?,
	target: boolean?,
	floor: boolean?,
	sinks: boolean?,
	litKind: string?,
	pushable: boolean?,
	plate: boolean?,
	dream: boolean?,
	warmth: number?,
	hides: boolean?,
}

const module = loader.init() or {}

module.kinds = {
	-- Plants
	bush = { sprite = "world/bush", solid = true },
	fruit_bush = {
		sprite = "world/fruit_bush",
		solid = true,
		interact = "fruit",
		label = "Pick",
		fruit = { item = "fig", count = 2, regrow = 90, empty = "world/bush" },
	},
	fern = { sprite = "world/fern", solid = false, flat = true },
	tall_grass = { sprite = "world/tall_grass", solid = false, flat = true },
	glowcap = { sprite = "world/glowcap", solid = false, flat = true, light = 2.5 },
	mushrooms = { sprite = "world/mushrooms", solid = false, flat = true },
	lily_pad = { sprite = "world/lily_pad", solid = false, flat = true },
	reeds = { sprite = "world/reeds", solid = false, flat = true },
	-- Rocks and remains
	rock = { sprite = "world/rock", solid = true },
	boulder = { sprite = "world/boulder", solid = true },
	bones = { sprite = "world/bones", solid = false, flat = true },
	skull = { sprite = "world/skull", solid = false, flat = true },
	-- Ruins of the sea people
	column = { sprite = "world/column_base", tall = "world/column_top", solid = true },
	column_broken = { sprite = "world/column_broken", solid = true },
	statue_head = { sprite = "world/statue_head", solid = true },
	tablet = { sprite = "world/tablet", solid = true, interact = "read", label = "Read" },
	pot = {
		sprite = "world/pot",
		solid = true,
		breakable = true,
		drops = {
			{ item = "fig", chance = 0.3 },
			{ item = "arrows", chance = 0.2, count = 3 },
			{ item = "ancient_fragment", chance = 0.1 },
		},
	},
	-- The explorers' camp
	campfire = {
		frames = { "world/campfire_0", "world/campfire_1" },
		fps = 6,
		solid = true,
		light = 4.5,
		warmth = 4,
		interact = "campfire",
		label = "Rest",
	},
	tent = { sprite = "world/tent_l", wide = "world/tent_r", solid = true },
	bedroll = { sprite = "world/bedroll", solid = false, flat = true },
	crate = { sprite = "world/crate", solid = true, breakable = true, drops = { { item = "fig", chance = 0.5 } } },
	sign = { sprite = "world/sign", solid = true, interact = "read", label = "Read" },
	note = { sprite = "world/note", solid = false, flat = true, interact = "read", label = "Read" },
	-- Containers and machines
	chest = { sprite = "world/chest", solid = true, interact = "chest", label = "Open" },
	quantum_chest = { sprite = "world/quantum_chest", solid = true, light = 1.5, interact = "quantum", label = "Open" },
	teleporter = { sprite = "world/teleporter_off", solid = false, flat = true, interact = "teleporter", label = "Use" },
	stairs = { sprite = "world/stairs_down", solid = false, flat = true, interact = "stairs", label = "Go down" },
	-- The desert
	cactus = { sprite = "world/cactus", solid = true },
	cactus_fruit = {
		sprite = "world/cactus_fruit",
		solid = true,
		interact = "fruit",
		label = "Pick",
		fruit = { item = "prickly_pear", count = 1, regrow = 120, empty = "world/cactus" },
	},
	dry_bush = { sprite = "world/dry_bush", solid = false, flat = true },
	palm = { sprite = "world/palm", tall = "world/palm_top", solid = true },
	palm_dead = { sprite = "world/palm_dead", tall = "world/palm_dead_top", solid = true },
	sun_crystal = { sprite = "world/sun_crystal", solid = true, light = 3.2 },
	cistern = { sprite = "world/cistern", solid = true, interact = "cistern", label = "Look" },
	valve = { sprite = "world/valve", solid = true, interact = "valve", label = "Turn", openKind = "valve_open" },
	valve_open = { sprite = "world/valve_open", solid = true },
	sun_eye = { sprite = "world/sun_eye", solid = true, target = true, openKind = "sun_eye_open" },
	sun_eye_open = { sprite = "world/sun_eye_open", solid = true },
	gate = { sprite = "world/gate", solid = true, interact = "gate", label = "Look", openKind = "gate_open" },
	gate_open = { sprite = "world/gate_open", solid = false, flat = true },
	council_door = { sprite = "world/council_door", solid = true, interact = "council", label = "Open", openKind = "council_door_open" },
	council_door_open = { sprite = "world/council_door_open", solid = false, flat = true },
	stairs_up = { sprite = "world/stairs_up", solid = false, flat = true, interact = "stairs", label = "Go up" },
	-- The wetlands
	lily_big = { sprite = "world/lily_big", solid = false, flat = true, floor = true },
	lily_sink = { sprite = "world/lily_sink", solid = false, flat = true, floor = true, sinks = true },
	lily_sunk = { sprite = "world/lily_sunk", solid = false, flat = true },
	willow = { sprite = "world/willow", tall = "world/willow_top", solid = true },
	glowmoss = { sprite = "world/glowmoss", solid = false, flat = true, light = 2.6 },
	bell = { sprite = "world/bell", solid = true, interact = "bell", label = "Ring" },
	skiff = { sprite = "world/skiff", solid = false, flat = true, interact = "skiff", label = "Board" },
	cauldron = { frames = { "world/cauldron_0", "world/cauldron_1" }, fps = 3, solid = true, light = 2 },
	shelf = { sprite = "world/shelf", solid = true },
	bed = { sprite = "world/bed", solid = true },
	surgery_table = { sprite = "world/surgery_table", solid = true },
	cabinet = { sprite = "world/cabinet", solid = true },
	shrine = { sprite = "world/shrine", solid = true, interact = "shrine", label = "Look", openKind = "shrine_lit" },
	shrine_lit = { sprite = "world/shrine_lit", solid = true, light = 2 },
	sick_door = { sprite = "world/sick_door", solid = true, interact = "sick_door", label = "Open", openKind = "sick_door_open" },
	sick_door_open = { sprite = "world/sick_door_open", solid = false, flat = true },
	office_door = { sprite = "world/office_door", solid = true, interact = "office_door", label = "Open", openKind = "office_door_open" },
	office_door_open = { sprite = "world/office_door_open", solid = false, flat = true },
	--- A way down to a place below (the isle of the Frog King): it shows where an exit is, it does nothing itself
	passage = { sprite = "world/stairs_down", solid = false, flat = true },
	-- The frontier
	pine = { sprite = "world/pine", tall = "world/pine_top", solid = true },
	snow_bush = { sprite = "world/snow_bush", solid = true },
	frozen_grass = { sprite = "world/frozen_grass", solid = false, flat = true },
	frost_crystal = { sprite = "world/frost_crystal", solid = true, light = 3 },
	brazier = { sprite = "world/brazier", solid = true, interact = "brazier", label = "Light", litKind = "brazier_lit" },
	brazier_lit = { frames = { "world/brazier_lit_0", "world/brazier_lit_1" }, fps = 6, solid = true, light = 4, warmth = 3.5 },
	--- A hearth inside a house: a campfire against a wall
	hearth = {
		frames = { "world/hearth_0", "world/hearth_1" },
		fps = 5,
		solid = true,
		light = 4.5,
		warmth = 5,
		interact = "campfire",
		label = "Rest",
	},
	ice_block = { sprite = "world/ice_block", solid = true, pushable = true },
	--- A chest frozen in a block of ice: it slides like a block, and thaws next to a lit brazier
	frozen_chest = { sprite = "world/frozen_chest", solid = true, pushable = true },
	rune_plate = { sprite = "world/rune_plate", solid = false, flat = true, plate = true },
	watch_gate = { sprite = "world/watch_gate", solid = true, interact = "look", label = "Look", openKind = "watch_gate_open" },
	watch_gate_open = { sprite = "world/watch_gate_open", solid = false, flat = true },
	--- The Cold Lock, the gate to the city: it only melts before the ember of the Watch
	cold_lock = { sprite = "world/cold_lock", solid = true, interact = "cold_lock", label = "Look", openKind = "cold_lock_open" },
	cold_lock_open = { sprite = "world/watch_gate_open", solid = false, flat = true },
	dream_mirror = { sprite = "world/dream_mirror", solid = true, interact = "mirror", label = "Gaze" },
	dream_bridge = { sprite = "world/dream_bridge", solid = false, flat = true, floor = true, dream = true },
	dream_rune = { sprite = "world/dream_rune", solid = false, flat = true, dream = true, interact = "read", label = "Read" },
	frozen_soldier = { sprite = "world/frozen_soldier", tall = "world/frozen_soldier_top", solid = true },
	steam = { frames = { "world/steam_0", "world/steam_1" }, fps = 3, solid = false, flat = true },
	footprints = { sprite = "world/footprints", solid = false, flat = true },
	snowdrift = { sprite = "world/snowdrift", solid = false, hides = true },
	snowman = { sprite = "world/snowman", solid = true },
	weapon_rack = { sprite = "world/weapon_rack", solid = true },
	-- Big decorations (see data/decor.luau): they block the way like rocks
	log_l = { sprite = "world/log_l", solid = true },
	log_r = { sprite = "world/log_r", solid = true },
	frozen_log_l = { sprite = "world/frozen_log_l", solid = true },
	frozen_log_r = { sprite = "world/frozen_log_r", solid = true },
	mossy_log_l = { sprite = "world/mossy_log_l", solid = true },
	mossy_log_r = { sprite = "world/mossy_log_r", solid = true },
	stump = { sprite = "world/stump", solid = true },
	dead_tree = { sprite = "world/dead_tree", solid = true },
	snowy_rock = { sprite = "world/snowy_rock", solid = true },
	--- Where the snowball gallery puts its targets (see gallery.luau): invisible
	gallery_spot = { solid = false, flat = true },
	-- Characters (their sprite comes from data/npcs.luau)
	npc = { solid = true, interact = "talk", label = "Talk" },
} :: { [string]: PropKind }

return module
