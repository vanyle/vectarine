--[[
Helpers to test the game quickly, available as `explorium.debug` in automated tests
and as commands in the editor console ("warp colonnade", "give bronze_spear", "heal", "levelup 2", "kit" to test any area
quickly: levels, every artefact, good gear, every teleporter on and every map explored; "kit 12 3" gives only what the
first 3 floors offer, to start the fourth floor as a player would).

They drive game_logic.luau directly, without the keyboard, so that one test step can play many frames.
]]
const debug = require("@vectarine/debug")
const dialog = require("./dialog")
const event = require("@vectarine/event")
const game_logic = require("./game_logic")
const game_state = require("./game_state")
const inventory = require("./inventory")
const items = require("./data/items")
const level = require("./level")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const menus = require("./menus")
const monsters = require("./data/monsters")
const notebook = require("./notebook")
const npcs = require("./npcs")
const player_rules = require("./player")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

const DIRECTIONS: { [string]: vec.Vec2 } = {
	up = vec.V2(0, -1),
	down = vec.V2(0, 1),
	left = vec.V2(-1, 0),
	right = vec.V2(1, 0),
	none = vec.V2(0, 0),
}

--- `screen` is the game screen module: its `game` field is the running game.
function module.install(screen: { game: types.Game? })
	local function game(): types.Game
		assert(screen.game, "No game is running")
		return screen.game :: types.Game
	end

	local helpers = {
		--- Draw the whole level at once, without darkness, to review it
		overview = false,
	}

	--- Run one update of 1/60 s. `move` is a direction name ("up", "down"...) or a vector.
	function helpers.step(move: (string | vec.Vec2)?, attack: boolean?, interact: boolean?)
		local direction = if type(move) == "string" then DIRECTIONS[move] elseif move then move :: vec.Vec2 else DIRECTIONS.none
		game_logic.update(game(), { move = direction, attack = attack == true, interact = interact == true }, 1 / 60)
	end

	--- Run `frames` updates moving in a direction (or standing still with nil), optionally attacking.
	function helpers.hold(frames: number, move: (string | vec.Vec2)?, attack: boolean?)
		for _ = 1, frames do
			helpers.step(move, attack, false)
		end
	end

	--- Press [E] once
	function helpers.interact()
		helpers.step(nil, false, true)
	end

	--- Where the player is and how they are, as text
	function helpers.describe(): string
		local g = game()
		local p = g.player
		return string.format(
			"%s (%.1f, %.1f) h%d facing %s | hp %d/%d sanity %d water %d sips %d warmth %d | lv %d xp %d | monsters %d | bag %d",
			g.level.id,
			p.pos.x,
			p.pos.y,
			p.height,
			p.facing,
			math.ceil(p.hp),
			player_rules.maxHp(p),
			math.ceil(p.sanity),
			math.floor(p.water),
			p.sips,
			math.floor(p.warmth),
			p.level,
			p.xp,
			#g.monsters,
			#p.inventory
		)
	end

	--- Go to a level, at an entry point (a digit of the map) or at a cell
	function helpers.warp(levelId: string, entryOrX: (string | number)?, y: number?)
		local g = game()
		if type(entryOrX) == "number" then
			game_state.enterLevel(g, levelId, nil, game_state.cellCenter(vec.V2(entryOrX, y or 0)))
		else
			game_state.enterLevel(g, levelId, entryOrX :: string?, nil)
		end
	end

	--- Put the player on a cell of the current level, facing a direction
	function helpers.place(x: number, y: number, facing: types.Direction?)
		local g = game()
		g.player.pos = game_state.cellCenter(vec.V2(x, y))
		g.player.facing = facing or g.player.facing
		-- Teleported: the player stands still (no sliding on from where they were)
		g.player.velocity = vec.V2(0, 0)
		g.player.knockback = vec.V2(0, 0)
		local cell = vec.V2(x, y)
		g.player.height = level.heightAt(g.level :: any, cell.x, cell.y)
	end

	function helpers.give(itemId: string, count: number?)
		inventory.give(game().player, itemId, count or 1)
	end

	function helpers.equip(itemId: string)
		local p = game().player
		if inventory.count(p, itemId) == 0 then
			inventory.give(p, itemId, 1)
		end
		-- Charms take a free charm slot. Requirements are ignored, to test any item.
		local slot = inventory.slotOf(itemId)
		if slot == "charm" then
			if not inventory.isEquipped(p, itemId) then
				inventory.toggleEquip(p, itemId)
			end
		elseif slot then
			(p.equipment :: any)[slot] = itemId
		end
	end

	function helpers.heal()
		local p = game().player
		p.hp = player_rules.maxHp(p)
		p.sanity = player_rules.maxSanity(p)
		p.water = 100
	end

	--- Gain `count` levels, as if by experience: the banner shows, and the stat points are left to spend
	function helpers.levelUp(count: number?)
		local g = game()
		for _ = 1, count or 1 do
			player_rules.gainXp(g, player_rules.xpToNext(g.player.level) - g.player.xp)
		end
	end

	--- The items a player can find on the floors down to `floor`: in chests, on the ground, in shops, on creatures
	local function itemsUpTo(floor: number): { [string]: boolean }
		local found: { [string]: boolean } = {}
		for id, data in levels.levels do
			if (data.depth or 1) <= floor then
				local lvl = level.load(id, {})
				for _, pickup in lvl.pickups do
					found[pickup.item] = true
				end
				for _, spawn in lvl.spawns do
					for _, drop in monsters.species[spawn.kind].drops or {} do
						found[drop.item] = true
					end
				end
				for _, prop in lvl.props do
					for _, entry in prop.data.items or {} do
						found[entry[1]] = true
					end
					local npc = if prop.kind == "npc" then npcs.npcs[prop.data.npc] else nil
					for _, offer in (if npc then npc.shop else nil) or {} do
						found[offer.item] = true
					end
				end
			end
		end
		return found
	end

	--- The most valuable item (by `value`, what traders give for it) of those for which `accept` is true, among
	--- `available` (all items when nil)
	local function mostValuable(accept: (items.Item) -> boolean, except: string?, available: { [string]: boolean }?): string?
		local best, bestValue = nil, -1
		for id, item in items.items do
			local allowed = available == nil or available[id] == true
			if id ~= except and allowed and accept(item) and (item.value or 0) > bestValue then
				best, bestValue = id, item.value or 0
			end
		end
		return best
	end

	--- Everything needed to test any area quickly: levels (stat points spent evenly), every artefact, the best
	--- gear equipped (the most valuable weapon, armor and charms, so items of new floors are picked up too),
	--- healing, every teleporter of the game turned on, and every map fully explored.
	--- With `floor`, only what the floors down to that one offer: their artefacts, the gear found there, their
	--- teleporters and maps (to start the next floor as a player would).
	function helpers.kit(targetLevel: number?, floor: number?)
		local g = game()
		local p = g.player
		local target = targetLevel or 8
		local available = if floor then itemsUpTo(floor) else nil
		local function within(levelId: string): boolean
			return floor == nil or (levels.levels[levelId].depth or 1) <= floor
		end
		while p.level < target do
			player_rules.gainXp(g, player_rules.xpToNext(p.level) - p.xp)
		end
		-- No banner for each of these levels
		for i = #g.events, 1, -1 do
			if g.events[i].kind == "levelUp" then
				table.remove(g.events, i)
			end
		end
		-- Every artefact (of the floors down to `floor`)
		for id, item in items.items do
			if item.kind == "artefact" then
				p.artefacts[id] = true
			end
		end
		if floor then
			for index, artefacts in items.artefactsByFloor do
				for _, id in artefacts do
					p.artefacts[id] = if index <= floor then true else nil
				end
			end
		end
		-- Gear
		local melee = mostValuable(function(item)
			return item.kind == "weapon" and not item.shot and not item.bomb
		end, nil, available)
		local ranged = mostValuable(function(item)
			return item.kind == "weapon" and item.shot ~= nil
		end, nil, available)
		local armor = mostValuable(function(item)
			return item.kind == "armor"
		end, nil, available)
		local charm = mostValuable(function(item)
			return item.kind == "charm"
		end, nil, available)
		local secondCharm = mostValuable(function(item)
			return item.kind == "charm"
		end, charm, available)
		local healing = mostValuable(function(item)
			return item.kind == "consumable" and item.use ~= nil and item.use.heal ~= nil
		end, nil, available)
		local calming = mostValuable(function(item)
			return item.kind == "consumable" and item.use ~= nil and item.use.sanity ~= nil and item.use.heal == nil
		end, nil, available)
		-- Stat points: first what the gear requires, then spread over every stat
		for _, id in { melee, ranged, armor } do
			for stat, value in items.items[id :: string].requires or {} do
				while (p.stats :: any)[stat] < value and player_rules.spendPoint(p, stat) do
				end
			end
		end
		local stats: { types.StatName } = { "health", "strength", "agility", "sanity", "intelligence" }
		local statIndex = 1
		while p.statPoints > 0 do
			player_rules.spendPoint(p, stats[statIndex])
			statIndex = statIndex % #stats + 1
		end
		for _, id in { melee, armor, charm, secondCharm } do
			if not inventory.isEquipped(p, id :: string) then
				helpers.equip(id :: string)
			end
		end
		helpers.give(ranged :: string, 1)
		helpers.give("arrows", 40)
		helpers.give("bomb", 10)
		helpers.give(healing :: string, 5)
		helpers.give(calming :: string, 5)
		helpers.heal()
		-- Every teleporter of the game (of the floors down to `floor`), turned on
		for _, id in levels.order do
			if not within(id) then
				continue
			end
			local lvl = if id == g.level.id then g.level :: level.Level else level.load(id, g.flags)
			for _, prop in lvl.props do
				if prop.kind == "teleporter" then
					g.flags[`teleporter:{prop.key}`] = { level = id, name = prop.data.name or lvl.name, x = prop.cell.x, y = prop.cell.y }
				end
			end
			-- The whole map of the level, explored
			local row = string.rep("1", lvl.width)
			local rows = table.create(lvl.height, row)
			g.notebook.explored[id] = rows
		end
		notebook.load(g)
	end

	--- Remove every creature of the level
	function helpers.clear()
		table.clear(game().monsters)
	end

	--- Go through the current dialog until its end, running its actions (giving items, opening a shop...)
	function helpers.finishDialog()
		for _ = 1, 200 do
			if not dialog.isOpen() then
				break
			end
			dialog.advance()
		end
	end

	function helpers.closeMenu()
		menus.close()
	end

	function helpers.menu(): string?
		return if menus.current then menus.current.kind else nil
	end

	function helpers.isDialogOpen(): boolean
		return dialog.isOpen()
	end

	--- The creatures of the level, as text (one line each)
	function helpers.monsters(): string
		local lines = {}
		for _, m in game().monsters do
			table.insert(lines, string.format("%s#%d (%.1f, %.1f) hp %d %s", m.kind, m.id, m.pos.x, m.pos.y, m.hp, m.state))
		end
		return table.concat(lines, "\n")
	end

	;(screen :: any).debug = helpers

	event.getConsoleCommandEvent():on("exploriumDebugCommands", function(command: string)
		local parts = string.split(command, " ")
		local name, argument = parts[1], parts[2]
		if not screen.game then
			return
		end
		if name == "warp" and argument then
			helpers.warp(argument, parts[3])
		elseif name == "give" and argument then
			helpers.give(argument, tonumber(parts[3]) or 1)
		elseif name == "heal" then
			helpers.heal()
		elseif name == "levelup" then
			helpers.levelUp(tonumber(argument) or 1)
			local p = game().player
			debug.print(`Level {p.level}: {p.statPoints} stat points to spend`)
		elseif name == "kit" then
			local floor = tonumber(parts[3])
			helpers.kit(tonumber(argument), floor)
			debug.print(if floor
				then `Kit given: levels, and what the floors 1 to {floor} offer (artefacts, gear, teleporters, maps)`
				else "Kit given: levels, artefacts, gear, every teleporter on, every map explored")
		elseif name == "clear" then
			helpers.clear()
		elseif name == "where" then
			debug.print(helpers.describe())
		elseif name == "overview" then
			-- The whole level at once, without darkness, to review it
			helpers.overview = not helpers.overview
		else
			debug.print("Commands: warp <level> [entry], give <item> [count], heal, levelup [count], kit [player level] [floor], clear, where, overview")
		end
	end)

	-- Automated tests and the Watcher reach the game through this global.
	_G.explorium = screen
end

return module
