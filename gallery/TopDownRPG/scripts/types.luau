--[[
Types shared by the gameplay modules.

They live in their own file so that data files (npcs, story, ...) can use the Game type
without creating a circular require with game_state.luau, which implements it.
]]
const legend = require("./data/legend")
const loader = require("@vectarine/loader")
const terrain = require("./data/terrain")
const tile = require("@vectarine/tile")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

export type Direction = legend.Direction
export type Terrain = terrain.Terrain
export type GateCondition = legend.GateCondition

--- Potions change how their drinker moves: "leap" jumps 2 tiles, "slant" moves diagonally
export type Effect = "leap" | "slant"

export type EntityKind =
	"monster"
	| "npc"
	| "sign"
	| "pickup"
	| "bomb"
	| "barrel"
	| "target"
	| "gate"
	| "door"
	| "ladder"

--- Anything that lives on a tile and is not part of the level: monsters, items, gates, ...
--- Most fields only make sense for one kind of entity.
export type Entity = {
	kind: EntityKind,
	sprite: string,
	--- Id of the room the entity belongs to. Entities never leave their room.
	room: string,
	--- Position in tiles. Always integers.
	pos: vec.Vec2,
	--- Removed entities (dead monsters, collected items) are kept in the list but ignored.
	removed: boolean,

	-- Visual state, only used by world_view.luau
	drawPos: vec.Vec2,
	--- Offset used to animate attacks and bumps. Decays to zero.
	bump: vec.Vec2,
	facingLeft: boolean,

	-- Monsters
	monster: string?,
	dir: Direction?,
	effect: Effect?,
	-- NPCs and signs
	npc: string?,
	lines: { string }?,
	-- Pickups
	pickup: legend.PickupKind?,
	amount: number?,
	potion: legend.Potion?,
	-- Bombs: turns before exploding
	fuse: number?,
	-- Targets
	hit: boolean?,
	-- Gates
	condition: GateCondition?,
	count: number?,
	spikes: boolean?,
	-- Ladders: the level they lead to
	level: number?,
}

export type Player = {
	pos: vec.Vec2,
	drawPos: vec.Vec2,
	bump: vec.Vec2,
	facing: Direction,
	--- The potion effect on the player, if any
	effect: Effect?,
	--- The direction the player moved in this turn, if they moved. Spiders use it.
	lastIntent: Direction?,
	dead: boolean,
	--- What killed the player: "spikes", "fall", "monster", "explosion" or "crushed"
	deathCause: string?,
}

--- What the player carries. The pickaxe is kept forever, the rest only lasts for the current level.
export type Inventory = {
	hasPickaxe: boolean,
	arrows: number,
	bombs: number,
	keys: number,
	--- The item used by [Space]
	selected: string?,
}

--- State of the level being played
export type LevelState = {
	index: number,
	roomId: string,
	startPos: vec.Vec2,
	--- The state of the level when it started, used to restart it
	initialState: string,
	--- Previous states of the level, used to undo
	undoStack: { string },
	--- Set when the gear is collected
	won: boolean,
	moves: number,
}

export type RoomInfo = {
	id: string,
	kind: "overworld" | "level",
	--- Position of the top-left tile of the room in the world.
	origin: vec.Vec2,
	--- Positions of the pressure plates of the room (they never change)
	plates: { vec.Vec2 },
}

export type World = {
	--- Size of the world in tiles
	width: number,
	height: number,
	--- The level. Layer 0 is the ground, layer 1 are the objects.
	tilemap: tile.GeneratedTilemap,
	--- Source data of the tilemap, indexed by `1 + x + y * width`. Change it with world.setTerrain.
	layers: { [number]: { number } },
	--- Copy of the layers when the world was built, used to reset levels.
	initialLayers: { [number]: { number } },
	rooms: { [string]: RoomInfo },
	--- Which room is at each position of the room grid ("x,y" -> room id)
	grid: { [string]: string },
	--- The data tables the world was built from (see world.isOutdated)
	sources: { any },
}

export type Speaker = { name: string, sprite: string? }

--- A dialog is a list of steps. A string is a line said by the speaker.
--- A function is run when the dialog reaches it. If it returns a string, it is shown as a narration line.
export type DialogStep = string | (game: Game) -> string?

--- Conditions of the gates of the current room, updated during each turn by rules.luau
export type Conditions = { plates: boolean, targets: boolean, monsters: boolean }

export type Game = {
	world: World,
	player: Player,
	--- Entities of each room
	entities: { [string]: { Entity } },
	--- What each room map creates, used to reset levels
	spawns: { [string]: { any } },
	inventory: Inventory,
	currentRoom: string,
	--- Set while playing a level
	level: LevelState?,
	--- Levels that were finished, by level number
	finished: { [number]: boolean },
	--- Free-form storage for story progress, for example flags.metFairy = true
	flags: { [string]: any },
	conditions: Conditions,
	--- Total number of turns played
	turn: number,
	--- Set by rules.luau when the player steps on a ladder. game_screen.luau then starts the level.
	enteringLevel: number?,
	--- "playing", or "ending" once the last level is finished
	status: "playing" | "ending",
	--- Where the player is sent back in the overworld after a level
	overworldReturn: vec.Vec2?,

	-- MARK: Methods (implemented in game_state.luau)
	startDialog: (self: Game, speaker: Speaker?, steps: { DialogStep }) -> (),
	--- Show a short floating text in the world
	say: (self: Game, pos: vec.Vec2, text: string, color: vec4.Vec4?) -> (),
	entityAt: (self: Game, pos: vec.Vec2) -> Entity?,
	objectAt: (self: Game, pos: vec.Vec2) -> Terrain?,
	groundAt: (self: Game, pos: vec.Vec2) -> Terrain?,
	finishedCount: (self: Game) -> number,
}

--- Unit vectors for each direction. y points down, like the rows of the room maps.
module.directions = {
	up = vec.V2(0, -1),
	down = vec.V2(0, 1),
	left = vec.V2(-1, 0),
	right = vec.V2(1, 0),
} :: { [Direction]: vec.Vec2 }

local opposite: { [Direction]: Direction } = {
	up = "down",
	down = "up",
	left = "right",
	right = "left",
}
module.opposite = opposite

--- Directions in a fixed order, to encode them as numbers
local directionList: { Direction } = { "up", "right", "down", "left" }
module.directionList = directionList

return module
