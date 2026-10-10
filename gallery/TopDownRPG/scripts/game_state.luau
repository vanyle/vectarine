--[[
The state of the game: starting a new game, entering levels, fainting and resting.

Creatures come back every time the player enters a level (or rests at a campfire), except bosses once killed.
Items lying in levels and chests are only there once: their flags remember that they were taken.
]]
const effects = require("./effects")
const inventory = require("./inventory")
const level = require("./level")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const notebook = require("./notebook")
const player_rules = require("./player")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- The level where a new game starts
module.START_LEVEL = "sinkhole"

--- The center of the feet of a character standing on a cell
function module.cellCenter(cell: vec.Vec2): vec.Vec2
	return cell + vec.V2(0.5, 0.6)
end

function module.new(): types.Game
	local game: types.Game = {
		levelId = module.START_LEVEL,
		player = player_rules.new(vec.V2(0, 0)),
		flags = {},
		chests = { quantum = {} },
		notebook = { creatures = {}, lore = {}, explored = {} },
		respawn = { level = module.START_LEVEL, pos = vec.V2(0, 0) },
		time = 0,
		level = nil :: any,
		monsters = {},
		projectiles = {},
		pickups = {},
		bombs = {},
		nextMonsterId = 1,
		events = {},
	}
	module.enterLevel(game, module.START_LEVEL, nil, nil)
	game.respawn = { level = module.START_LEVEL, pos = game.player.pos }
	return game
end

local function spawnMonsters(game: types.Game)
	game.monsters = {}
	for _, spawn in game.level.spawns do
		local species = monsters.species[spawn.kind]
		-- Creatures of the canal beds don't come back once the canals are full of water
		local ground = level.groundAt(game.level :: any, spawn.cell.x, spawn.cell.y)
		local drowned = ground ~= nil and not ground.walkable and not species.flying
		if not (spawn.uniqueKey and game.flags[spawn.uniqueKey]) and not drowned then
			local pos = module.cellCenter(spawn.cell)
			local monster: types.Monster = {
				id = game.nextMonsterId,
				kind = spawn.kind,
				pos = pos,
				height = level.heightAt(game.level :: any, spawn.cell.x, spawn.cell.y),
				spawn = pos,
				hp = monsters.species[spawn.kind].hp,
				state = "idle",
				stateTime = 0,
				facing = "left",
				cooldown = 1,
				hitTime = 10,
				knockback = vec.V2(0, 0),
				uniqueKey = spawn.uniqueKey,
				group = spawn.group,
				phase = 1,
				dead = false,
			}
			table.insert(game.monsters, monster)
			game.nextMonsterId += 1
		end
	end
end

--- Where the player arrives through an entry: exits are on the edges of the map, so the player arrives one step
--- inside, next to the exit. Walking back onto it leaves the level again right away.
local function arrivalCell(lvl: level.Level, cell: vec.Vec2, player: types.Player): vec.Vec2
	local inward = vec.V2(0, 0)
	if cell.x == 0 then
		inward = vec.V2(1, 0)
	elseif cell.x == lvl.width - 1 then
		inward = vec.V2(-1, 0)
	elseif cell.y == 0 then
		inward = vec.V2(0, 1)
	elseif cell.y == lvl.height - 1 then
		inward = vec.V2(0, -1)
	end
	-- The player can arrive on water with the ferry, in miasma with the mask
	local mover: level.Mover = {
		height = level.heightAt(lvl, cell.x, cell.y),
		boat = player.artefacts.ferry_skiff == true,
		masked = player.artefacts.beaked_mask == true,
	}
	if inward:length() > 0 then
		local inside = cell + inward
		if level.isFree(lvl, inside.x, inside.y, mover) then
			return inside
		end
		return cell
	end
	-- A door inside the map (a hut): the player arrives on the free cell next to it, below it if possible
	for _, offset in { vec.V2(0, 1), vec.V2(0, -1), vec.V2(1, 0), vec.V2(-1, 0) } do
		local next = cell + offset
		local isEntry = false
		for _, cells in lvl.entries do
			for _, entryCell in cells do
				if entryCell.x == next.x and entryCell.y == next.y then
					isEntry = true
				end
			end
		end
		if not isEntry and level.isFree(lvl, next.x, next.y, mover) then
			return next
		end
	end
	return cell
end

--- Enter a level, at an entry point (`entry`, with the index of the cell in the entry for wide exits)
--- or at a position. With neither, the player starts at the "@" of the level.
function module.enterLevel(game: types.Game, levelId: string, entry: string?, pos: vec.Vec2?, entryIndex: number?)
	if game.level then
		notebook.flush(game)
	end
	local loaded = level.load(levelId, game.flags)
	game.levelId = levelId
	game.level = loaded
	game.projectiles = {}
	game.bombs = {}
	effects.clear()

	local target = pos
	if not target and entry then
		local cells = loaded.entries[entry]
		assert(cells and #cells > 0, `Level '{levelId}' has no entry '{entry}'`)
		target = module.cellCenter(arrivalCell(loaded, cells[math.clamp(entryIndex or 1, 1, #cells)], game.player))
	end
	if not target then
		target = module.cellCenter(loaded.start or vec.V2(1, 1))
	end
	local player = game.player
	player.pos = target :: vec.Vec2
	local cell = level.cellOf(player.pos)
	player.height = level.heightAt(loaded, cell.x, cell.y)
	player.knockback = vec.V2(0, 0)
	player.velocity = vec.V2(0, 0)

	spawnMonsters(game)
	game.pickups = {}
	for _, pickup in loaded.pickups do
		if not game.flags[pickup.key] then
			table.insert(game.pickups, {
				pos = module.cellCenter(pickup.cell),
				item = pickup.item,
				count = pickup.count,
				key = pickup.key,
				age = 0,
				dead = false,
			})
		end
	end
	notebook.load(game)
	game.events[#game.events + 1] = { kind = "enteredLevel", level = levelId }
end

--- The player fainted: they wake up at the last campfire, with their items, and the creatures are back.
function module.wakeUp(game: types.Game)
	local player = game.player
	player.hp = player_rules.maxHp(player)
	player.sanity = math.max(player.sanity, player_rules.maxSanity(player) * 0.5)
	player.water = math.max(player.water, 60)
	player.poison = 0
	player.invincible = 2
	module.enterLevel(game, game.respawn.level, nil, game.respawn.pos)
end

--- Rest at a campfire: heals everything, refills the bottle, and sets the place where to wake up after fainting.
--- Creatures come back to life.
function module.rest(game: types.Game, campfire: types.Prop)
	local player = game.player
	player.hp = player_rules.maxHp(player)
	player.sanity = player_rules.maxSanity(player)
	player.poison = 0
	player.sips = player_rules.bottleSize(player)
	game.respawn = { level = game.level.id, pos = player.pos }
	spawnMonsters(game)
	game.projectiles = {}
	effects.burst(campfire.cell + vec.V2(0.5, 0.3), "sparkle", 16)
end

--- Give items to the player, dropping what doesn't fit at their feet. Returns a message.
function module.giveOrDrop(game: types.Game, itemId: string, count: number): number
	local left = inventory.give(game.player, itemId, count)
	if left > 0 then
		table.insert(game.pickups, { pos = game.player.pos, item = itemId, count = left, age = 0, dead = false })
	end
	return left
end

return module
