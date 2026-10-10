--[[
Saving and loading the game with `persist`, so that "Continue" on the title screen picks up where the player was,
even after quitting.

The game is saved when resting at a campfire, when entering a level and when finding an artefact. It is also saved
every few seconds while playing (AUTOSAVE_INTERVAL), so that nothing is lost if the game stops (a crash, or reloading
the project in the editor), but only where it is safe to wake up (see canAutosave).
What is saved: the player (stats, items, equipment, artefacts), the flags of the story, the chests, the notebook,
where the player wakes up after fainting, and where they are. Creatures are not saved: they come back anyway.
]]
const game_state = require("./game_state")
const level = require("./level")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const notebook = require("./notebook")
const persist = require("@vectarine/persist")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

const SAVE_NAME = "explorium.save"
--- Increase it when the format of the save changes, to ignore old saves
const VERSION = 1
--- Seconds of play between two autosaves
module.AUTOSAVE_INTERVAL = 5

--- The save as last written or read (false when there is none), to avoid reading the disk every frame
module.saved = module.saved :: any

local function point(v: vec.Vec2): { x: number, y: number }
	return { x = v.x, y = v.y }
end

function module.save(game: types.Game)
	notebook.flush(game)
	local player = game.player
	local data = {
		version = VERSION,
		level = game.levelId,
		pos = point(player.pos),
		player = {
			height = player.height,
			facing = player.facing,
			hp = player.hp,
			sanity = player.sanity,
			water = player.water,
			warmth = player.warmth,
			sips = player.sips,
			bottle = player.bottle,
			stats = player.stats,
			level = player.level,
			xp = player.xp,
			statPoints = player.statPoints,
			inventory = player.inventory,
			equipment = player.equipment,
			artefacts = player.artefacts,
		},
		flags = game.flags,
		chests = game.chests,
		notebook = game.notebook,
		respawn = { level = game.respawn.level, pos = point(game.respawn.pos) },
		time = game.time,
	}
	module.saved = data
	persist.save(SAVE_NAME, data)
end

--- Can the game be saved right now, with the player waking up here? Only on plain, firm ground, standing still enough,
--- and not during a boss fight (the boss comes back, the player would wake up in the middle of the arena).
function module.canAutosave(game: types.Game): boolean
	local player = game.player
	if player.hp <= 0 or player.knockback:length() > 0.01 or player.velocity:length() > 0.5 then
		return false
	end
	local lvl = game.level :: level.Level
	local cell = level.cellOf(player.pos)
	local ground = level.groundAt(lvl, cell.x, cell.y)
	local wall = level.wallAt(lvl, cell.x, cell.y)
	if not ground or not ground.walkable or wall or player.height ~= level.heightAt(lvl, cell.x, cell.y) then
		return false
	end
	if ground.water or ground.sailable or ground.bog or ground.toxic or ground.miasma or ground.slippery or ground.thin
		or ground.freezing or ground.veil or ground.ladder or ground.climbable then
		return false
	end
	for _, monster in game.monsters do
		if not monster.dead and monsters.species[monster.kind].boss then
			return false
		end
	end
	return true
end

--- The saved game, or nil if there is none (or if it can't be read).
function module.load(): any?
	if module.saved == nil then
		local saved = persist.load(SAVE_NAME)
		local valid = type(saved) == "table" and saved.version == VERSION and type(saved.player) == "table"
		module.saved = if valid then saved else false
	end
	return if module.saved then module.saved else nil
end

function module.clear()
	module.saved = false
	persist.save(SAVE_NAME, false)
end

--- Build a game from a save
function module.restore(data: any): types.Game
	local game = game_state.new()
	local player = game.player
	for key, value in data.player do
		(player :: any)[key] = value
	end
	-- JSON turns empty tables into arrays or objects: make sure the tables are there
	player.inventory = player.inventory or {}
	player.equipment = player.equipment or {}
	player.artefacts = player.artefacts or {}
	game.flags = data.flags or {}
	game.chests = data.chests or { quantum = {} }
	game.notebook = data.notebook or { creatures = {}, lore = {}, explored = {} }
	game.notebook.creatures = game.notebook.creatures or {}
	game.notebook.lore = game.notebook.lore or {}
	game.notebook.explored = game.notebook.explored or {}
	game.respawn = { level = data.respawn.level, pos = vec.V2(data.respawn.pos.x, data.respawn.pos.y) }
	game.time = data.time or 0
	game.events = {}
	game_state.enterLevel(game, data.level, nil, vec.V2(data.pos.x, data.pos.y))
	return game
end

return module
