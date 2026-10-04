--[[
Types shared by the whole game.

The state of the game (`Game`) is split in two:
- what is saved: the player, the flags of the story, the content of chests, the notebook...
- what is rebuilt when entering a level (`Game.level` and the `LevelRuntime`): monsters, projectiles, pickups.

World coordinates are in tiles, with y pointing down. Positions of characters are the center of their feet.
]]
const vec = require("@vectarine/vec")

export type Vec2 = vec.Vec2

export type Direction = "up" | "down" | "left" | "right"
--- `cold` freezes (it also chills: see frost.luau), `dream` comes from the dream (phantoms, dream weapons)
export type DamageType = "blunt" | "slash" | "pierce" | "fire" | "electric" | "acid" | "cold" | "dream"
export type StatName = "health" | "sanity" | "strength" | "agility" | "intelligence"
export type Slot = "weapon" | "armor" | "charm"

export type Stats = {
	health: number,
	sanity: number,
	strength: number,
	agility: number,
	intelligence: number,
}

export type ItemStack = { id: string, count: number }

export type Player = {
	pos: Vec2,
	--- The height layer the player stands on (see level.luau)
	height: number,
	facing: Direction,
	hp: number,
	sanity: number,
	--- 0 = dying of thirst, 100 = not thirsty at all
	water: number,
	--- 0 = freezing, 100 = warm (see frost.luau)
	warmth: number,
	--- Sips left in the water bottle
	sips: number,
	stats: Stats,
	level: number,
	xp: number,
	--- Points to spend on stats, earned when leveling up
	statPoints: number,
	inventory: { ItemStack },
	--- Two charms can be worn at the same time (`charm` and `charm2`)
	equipment: { weapon: string?, armor: string?, charm: string?, charm2: string? },
	--- Artefacts found, by item id. They are permanent abilities and don't take space in the bag.
	artefacts: { [string]: boolean },
	--- Seconds left before the next attack is possible
	attackTimer: number,
	--- Time since the last attack started, to draw the swing
	swingTime: number,
	--- Seconds of invincibility left after being hit
	invincible: number,
	--- Seconds of poison left
	poison: number,
	--- How many sips the water bottle holds (nil: the default, see player.luau)
	bottle: number?,
	knockback: Vec2,
	--- Speed kept from the last frame: on ice, the player slides (see frost.luau)
	velocity: Vec2,
	--- Time spent walking, to animate the steps
	walkTime: number,
	moving: boolean,
}

export type Monster = {
	--- Unique in the level, to find a monster again
	id: number,
	--- Key of data/monsters.luau
	kind: string,
	pos: Vec2,
	height: number,
	spawn: Vec2,
	hp: number,
	--- What the monster is doing, see monster_ai.luau
	state: string,
	stateTime: number,
	facing: Direction,
	--- A point the monster is moving towards
	target: Vec2?,
	cooldown: number,
	--- Time since the monster was last hit (to flash it)
	hitTime: number,
	knockback: Vec2,
	--- Key of the flag set when a unique monster (a boss) is killed
	uniqueKey: string?,
	--- The guards of a chest have a group: the chest is sealed while they are alive
	group: string?,
	--- Bosses have phases, see monster_ai.luau
	phase: number,
	--- Seconds left of being slowed down by the cold (cold weapons)
	chill: number?,
	dead: boolean,
}

export type Projectile = {
	pos: Vec2,
	velocity: Vec2,
	damage: number,
	damageType: DamageType,
	fromPlayer: boolean,
	sprite: string,
	life: number,
	poison: number?,
	--- Shot by a boss: it hits softer when the player is low (see combat.bossFactor)
	boss: boolean?,
	dead: boolean,
}

--- A lit bomb, about to blow up
export type Bomb = { pos: Vec2, fuse: number, damage: number }

export type Pickup = {
	pos: Vec2,
	item: string,
	count: number,
	--- Flag set when picked up, for items placed in the level (dropped items have none)
	key: string?,
	--- Time since it appeared, to animate it
	age: number,
	dead: boolean,
}

--- Something placed in a level: plants, rocks, chests, characters...
export type Prop = {
	--- Key of data/props.luau
	kind: string,
	--- Cell of the prop
	cell: Vec2,
	--- Unique key for props whose state is saved (chests, fruit bushes, teleporters...)
	key: string,
	--- Extra data from the level: the content of a chest, the text of a sign, the id of a character...
	data: { [string]: any },
}

export type Link = { level: string, entry: string }

--- Sandstorms of a level: `calm` seconds of calm, then a storm of `duration` seconds (or `always`).
--- `wind` is the direction the wind blows to, `strength` how fast it pushes, in tiles per second.
export type Storm = { calm: number?, duration: number?, always: boolean?, wind: Vec2, strength: number }

--- A level, once loaded (see level.luau). Only bombs change it while it is played (they open cracked walls).
export type LevelRuntime = {
	id: string,
	name: string,
	subtitle: string,
	width: number,
	height: number,
	--- How dark the level is: 0 = full light, 1 = pitch black
	darkness: number,
	--- The floor of the Explorium the level is on (1 = the oasis, 2 = the desert)
	depth: number,
	--- How fast the player gets thirsty, compared to the oasis
	heat: number,
	storm: Storm?,
	--- Snow falling over the level, from 0 to 1 (drawn only)
	snowfall: number,
	--- Parts of the level without light: x1, y1, x2, y2 (cells, included) and their darkness
	darkAreas: { { number } },
	--- Increases every time the terrain changes (a bomb opened a wall), so that it is drawn again
	version: number,
	props: { Prop },
	--- Cells of each entry point, by name ("1", "2"...)
	entries: { [string]: { Vec2 } },
	links: { [string]: Link },
	start: Vec2?,
	--- Monsters to create when entering the level
	spawns: { { kind: string, cell: Vec2, uniqueKey: string?, group: string? } },
	pickups: { { item: string, count: number, cell: Vec2, key: string } },
}

export type Speaker = { name: string, sprite: string? }

--- Something that happened, for the screen to show. `kind` says what ("message", "dialog", "levelUp"...).
export type GameEvent = { [string]: any }

--- A step of a dialog: a line of text, or a function that does something (give an item, start a quest...)
--- and optionally returns a line of narration to show.
export type DialogStep = string | (game: any) -> string?

export type Game = {
	levelId: string,
	player: Player,
	--- Story progress and everything that must be remembered: opened chests, killed bosses, quests...
	flags: { [string]: any },
	--- Items in chests, by chest key. The quantum chest is the key "quantum".
	chests: { [string]: { ItemStack } },
	notebook: {
		--- Creatures killed, by kind (a creature is in the bestiary once seen, even with 0 kills)
		creatures: { [string]: number },
		--- Lore texts read, by id
		lore: { [string]: boolean },
		--- Explored cells of each level, as rows of "0"/"1" (see notebook.luau)
		explored: { [string]: { string } },
	},
	--- Where to wake up after fainting
	respawn: { level: string, pos: Vec2 },
	--- Total play time, in seconds
	time: number,

	-- Not saved: rebuilt when entering a level
	level: LevelRuntime,
	monsters: { Monster },
	projectiles: { Projectile },
	pickups: { Pickup },
	bombs: { Bomb },
	nextMonsterId: number,
	--- Something the screen must do: enter a level, show a title card, the death screen...
	events: { GameEvent },
}

return {}
