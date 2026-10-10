--[[
What [E] does: talk, open, rest, pick, read, use a teleporter, fill the bottle...

`find(game)` returns what the player can interact with right now (in front of them, or under their feet),
with the label shown in the key prompts. `run` does it: simple things happen right away, others add an event
for the screen (open a dialog, a shop, the storage of a chest, the list of teleporters).
]]
const combat = require("./combat")
const dream = require("./dream")
const effects = require("./effects")
const game_state = require("./game_state")
const inventory = require("./inventory")
const items = require("./data/items")
const level = require("./level")
const loader = require("@vectarine/loader")
const lore = require("./data/lore")
const npcs = require("./npcs")
const player_rules = require("./player")
const props = require("./data/props")
const switches = require("./switches")
const theme = require("./theme")
const tutorials = require("./tutorials")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

export type Interaction = {
	label: string,
	run: (game: types.Game) -> (),
	--- The prop it is done with, if any (filling the bottle at a pool has none)
	prop: types.Prop?,
}

--- The bell of the ferry: the strokes rung so far, and when the last one was. Strokes must be slow: between
--- BELL_MIN and BELL_MAX seconds apart.
local bellRing = { count = 0, last = -100 }
const BELL_MIN = 1.5
const BELL_MAX = 4

--- The valves of the Great Pump: when they are all open, water flows again on the whole desert
module.VALVES = { "valve_west", "valve_east", "valve_north" }

local function message(game: types.Game, text: string)
	game.events[#game.events + 1] = { kind = "message", text = text }
end

local function dialog(game: types.Game, speaker: types.Speaker?, steps: { types.DialogStep })
	game.events[#game.events + 1] = { kind = "dialog", speaker = speaker, steps = steps }
end

-- MARK: Props

function module.isChestOpen(game: types.Game, prop: types.Prop): boolean
	return game.flags[`opened:{prop.key}`] == true
end

--- Has this fruit bush (or cactus) something to pick right now? Fruits grow back after a while.
function module.hasFruit(game: types.Game, prop: types.Prop): boolean
	local fruit = props.kinds[prop.kind].fruit
	local picked = game.flags[`picked:{prop.key}`]
	return fruit ~= nil and (picked == nil or game.time - picked > fruit.regrow)
end

--- Does this cistern have water? (The colony's cistern always has some.)
function module.isCisternFull(game: types.Game, prop: types.Prop): boolean
	return game.flags.waterRestored == true or prop.data.always == true
end

--- Is this prop hidden right now? (The ferry only appears once called: `data.showIf`, and leaves: `data.hideIf`.)
function module.isHidden(game: types.Game, prop: types.Prop): boolean
	local data = prop.data
	return (data.showIf ~= nil and not game.flags[data.showIf]) or (data.hideIf ~= nil and game.flags[data.hideIf] == true)
end

--- Is this prop there for the player? Dream props (dream bridges, dream writing) are only there for dreamers.
function module.isThere(game: types.Game, prop: types.Prop): boolean
	local dreamy = props.kinds[prop.kind].dream or (prop.kind == "npc" and npcs.npcs[prop.data.npc].dream)
	return not module.isHidden(game, prop) and (not dreamy or dream.isDreaming(game))
end

--- Can [E] do something with this prop right now? (A bush without figs is just a bush.)
function module.canInteract(game: types.Game, prop: types.Prop): boolean
	if not props.kinds[prop.kind].interact or game.flags[`broken:{prop.key}`] or not module.isThere(game, prop) then
		return false
	end
	return props.kinds[prop.kind].fruit == nil or module.hasFruit(game, prop)
end

function module.isTeleporterOn(game: types.Game, prop: types.Prop): boolean
	return game.flags[`teleporter:{prop.key}`] ~= nil
end

local actions: { [string]: (game: types.Game, prop: types.Prop) -> () } = {
	talk = function(game, prop)
		local npc = npcs.npcs[prop.data.npc]
		local steps = npc.talk(game)
		if npc.shop then
			table.insert(steps, function(g: types.Game): string?
				g.events[#g.events + 1] = { kind = "shop", npc = prop.data.npc }
				return nil
			end)
		end
		dialog(game, { name = npc.name, sprite = `characters/{npc.sprite}_0` }, steps)
	end,

	chest = function(game, prop)
		if module.isChestOpen(game, prop) then
			game.events[#game.events + 1] = { kind = "storage", chest = prop.key }
			return
		end
		-- A guarded chest stays sealed while its guards are alive (no grabbing the loot in the middle of a fight)
		local guard = prop.data.guard
		if guard then
			for _, monster in game.monsters do
				if monster.group == guard and not monster.dead then
					message(game, prop.data.sealed or "The chest is sealed. Its guards are still watching over it.")
					return
				end
			end
		end
		game.flags[`opened:{prop.key}`] = true
		-- The content of the chest, from the level, becomes its storage
		local content: { types.ItemStack } = {}
		for _, entry in prop.data.items or {} do
			table.insert(content, { id = entry[1], count = entry[2] or 1 })
		end
		game.chests[prop.key] = content
		effects.burst(prop.cell + vec.V2(0.5, 0.4), "sparkle", 12)
		local found = {}
		for i = #content, 1, -1 do
			local stack = content[i]
			local left = inventory.give(game.player, stack.id, stack.count)
			if left < stack.count then
				table.insert(found, items.items[stack.id].name)
				tutorials.onPickup(game, stack.id)
				if items.items[stack.id].kind == "artefact" then
					game.events[#game.events + 1] = { kind = "artefact", item = stack.id }
				end
			end
			stack.count = left
			if left == 0 then
				table.remove(content, i)
			end
		end
		if #found == 0 then
			message(game, if #content > 0 then "Your bag is full." else "The chest is empty.")
		else
			message(game, `You found: {table.concat(found, ", ")}`)
		end
	end,

	quantum = function(game, prop)
		game.events[#game.events + 1] = { kind = "storage", chest = "quantum" }
	end,

	campfire = function(game, prop)
		game_state.rest(game, prop)
		game.events[#game.events + 1] = { kind = "rested" }
	end,

	fruit = function(game, prop)
		local fruit = props.kinds[prop.kind].fruit :: any
		if not module.hasFruit(game, prop) then
			message(game, "Nothing ripe yet. It grows back slowly.")
			return
		end
		game.flags[`picked:{prop.key}`] = game.time
		local left = game_state.giveOrDrop(game, fruit.item, fruit.count)
		effects.burst(prop.cell + vec.V2(0.5, 0.4), "leaves", 10)
		local name = string.lower(items.items[fruit.item].name)
		local picked = if fruit.count > 1 then `{fruit.count} {name}s` else `a {name}`
		message(game, if left > 0 then "Your bag is full: it falls on the ground." else `You pick {picked}.`)
	end,

	cistern = function(game, prop)
		if not module.isCisternFull(game, prop) then
			message(game, "A cistern of the sea people, dry for a long time. There is only cracked mud at the bottom.")
			return
		end
		local player = game.player
		player.water = 100
		player.sips = player_rules.bottleSize(player)
		effects.burst(prop.cell + vec.V2(0.5, 0.4), "water", 10)
		message(game, "You drink your fill, and fill your bottle.")
	end,

	valve = function(game, prop)
		switches.open(game, prop.data.id)
		effects.burst(prop.cell + vec.V2(0.5, 0.8), "water", 10)
		local turned = 0
		for _, id in module.VALVES do
			if switches.isOpen(game, id) then
				turned += 1
			end
		end
		if turned >= #module.VALVES then
			game.flags.waterRestored = true
			game.events[#game.events + 1] = { kind = "pumpRestored" }
		else
			message(game, `The wheel turns with a groan. Water rumbles in the pipes, then stops. {turned} of {#module.VALVES} valves open.`)
		end
	end,

	gate = function(game, prop)
		message(game, "A bronze gate of the sea people, sealed shut. Their words are carved on the tablet next to it.")
	end,

	council = function(game, prop)
		if game.player.equipment.weapon then
			message(game, "The marble door doesn't move. Carved on it: an olive branch, and a crossed-out sword.")
			return
		end
		switches.open(game, prop.data.id)
		message(game, "Empty hands. The door of the council slides open, in silence.")
	end,

	read = function(game, prop)
		local text = lore.texts[prop.data.lore]
		assert(text, `Unknown lore text '{prop.data.lore}'`)
		local readable = lore.canRead(game, text)
		local steps: { types.DialogStep } = {}
		for _, line in text.lines do
			steps[#steps + 1] = if readable then line else lore.scramble(line)
		end
		if not readable then
			table.insert(steps, function(_g: types.Game): string?
				return "The writing of the sea people. You can't make sense of a single word."
			end)
		end
		if not game.notebook.lore[prop.data.lore] then
			game.notebook.lore[prop.data.lore] = true
			if text.ancient then
				table.insert(steps, function(g: types.Game): string?
					player_rules.gainXp(g, 8)
					return if readable
						then "You copy the writing in your notebook. +8 experience"
						else "You copy the strange letters in your notebook, as best as you can. +8 experience"
				end)
			end
		end
		dialog(game, { name = text.author }, steps)
	end,

	teleporter = function(game, prop)
		if module.isTeleporterOn(game, prop) then
			game.events[#game.events + 1] = { kind = "teleport", from = prop.key }
			return
		end
		if inventory.take(game.player, "energy_core", 1) then
			game.flags[`teleporter:{prop.key}`] = { level = game.level.id, name = prop.data.name or game.level.name, x = prop.cell.x, y = prop.cell.y }
			effects.burst(prop.cell + vec.V2(0.5, 0.5), "spark", 24)
			effects.flash(theme.colors.sanity, 0.4)
			message(game, "The energy core clicks in place. The teleporter wakes up!")
		else
			message(game, "A machine of the sea people, cold and silent. It has a slot for an energy core.")
		end
	end,

	--- The bell of the ferry: three slow strokes call the ferry (strokes too fast or too slow start over)
	bell = function(game, prop)
		local ring = bellRing
		local since = game.time - ring.last
		ring.count = if ring.count > 0 and since >= BELL_MIN and since <= BELL_MAX then ring.count + 1 else 1
		ring.last = game.time
		effects.floatText(prop.cell + vec.V2(0.5, -0.6), "DONG", theme.colors.gold)
		effects.shake(0.1)
		if ring.count >= 3 and not game.flags.ferryCalled then
			game.flags.ferryCalled = true
			ring.count = 0
			message(game, "Far on the water, something answers the bell. The ferry of the tide glides to the dock!")
		elseif not game.flags.ferryCalled then
			message(game, "The bell rings over the water. Nothing answers.")
		end
	end,

	skiff = function(game, prop)
		game.flags.ferryTaken = true
		inventory.give(game.player, "ferry_skiff", 1)
		effects.burst(prop.cell + vec.V2(0.5, 0.5), "sparkle", 16)
		game.events[#game.events + 1] = { kind = "artefact", item = "ferry_skiff" }
	end,

	--- The door of the surgery: "the House of Healing opens its doors to the sick alone"
	sick_door = function(game, prop)
		if game.player.poison > 0 then
			switches.open(game, prop.data.id)
			message(game, "The bronze plate glows green at your touch. The door of the sick slides open.")
		else
			message(game, "The door doesn't move. On its bronze plate: a hand, and a drop of green.")
		end
	end,

	office_door = function(game, prop)
		if inventory.count(game.player, "physician_seal") > 0 or game.flags["killed:chief_mender"] then
			switches.open(game, prop.data.id)
			message(game, "The seal of the physician fits the lock. The door of the office opens.")
		else
			message(game, "Locked. The lock bears the seal of the head physician.")
		end
	end,

	shrine = function(game, prop)
		message(game, "A dry stone basin. White lilies are carved all around it.")
	end,

	--- Braziers of the frontier: light them, and they stay lit (they warm the place, see frost.luau)
	brazier = function(game, prop)
		game.flags[`lit:{prop.key}`] = true
		prop.kind = props.kinds[prop.kind].litKind :: string
		effects.burst(prop.cell + vec.V2(0.5, 0.3), "spark", 14)
		local lit = 0
		for flag in game.flags do
			if string.sub(flag, 1, 4) == "lit:" then
				lit += 1
			end
		end
		message(game, if lit == 1
			then "The old oil catches fire. The brazier warms the air around it: the cold can't reach you here."
			else "You light the brazier. Its warmth spreads around it.")
	end,

	--- A dreaming mirror of the Watch: gazing in it sinks the player into the dream
	mirror = function(game, prop)
		dream.sink(game)
		message(game, "You gaze into the frozen mirror. Your reflection blinks before you do... and the world drifts away.")
	end,

	--- The Cold Lock: a gate of ice that melts before the ember of the Watch
	cold_lock = function(game, prop)
		if game.player.artefacts.watch_ember then
			switches.open(game, prop.data.id)
			effects.burst(prop.cell + vec.V2(0.5, 0.5), "steam", 30)
			message(game, "The ember of the Watch glows in your hands. The ice of the gate weeps, then gives way: the way to the city is open.")
		else
			message(game, "A gate of ice, thick as a wall. Carved in it: a flame. The cold that comes out of it burns.")
		end
	end,

	--- Things that are only looked at: their text is in `data.text`
	look = function(game, prop)
		message(game, prop.data.text or "Nothing happens.")
	end,

	--- Stairs lead to another floor (`data.to`, `data.entry`). Stairs without a destination end the template.
	stairs = function(game, prop)
		if prop.data.to then
			game.events[#game.events + 1] = { kind = "travel", level = prop.data.to, entry = prop.data.entry }
		else
			game.events[#game.events + 1] = { kind = "stairs" }
		end
	end,
}

--- The prop the player is facing (or standing on, for flat props), and what [E] does with it.
local function facingProp(game: types.Game): types.Prop?
	local player = game.player
	local forward = combat.DIRECTIONS[player.facing]
	for _, offset in { 0.7, 0.3, 1.1 } do
		local cell = level.cellOf(player.pos + vec.V2(0, -0.2) + forward:scale(offset))
		local prop = combat.propAt(game, cell.x, cell.y)
		if prop and props.kinds[prop.kind].interact and module.isThere(game, prop) then
			return prop
		end
	end
	return nil
end

local function facingWater(game: types.Game): boolean
	local player = game.player
	local forward = combat.DIRECTIONS[player.facing]
	local cell = level.cellOf(player.pos + vec.V2(0, -0.2) + forward:scale(0.8))
	local ground = level.groundAt(game.level :: any, cell.x, cell.y)
	return ground ~= nil and ground.water == true
end

function module.find(game: types.Game): Interaction?
	local prop = facingProp(game)
	if prop then
		local name: string = prop.kind
		local kind = props.kinds[name]
		local label = kind.label or "Use"
		if name == "chest" and module.isChestOpen(game, prop) then
			label = "Search"
		elseif name == "teleporter" and module.isTeleporterOn(game, prop) then
			label = "Travel"
		elseif name == "cistern" and module.isCisternFull(game, prop) then
			label = "Drink"
		end
		local action = actions[kind.interact :: string]
		return {
			label = label,
			prop = prop,
			run = function(g: types.Game)
				action(g, prop)
			end,
		}
	end
	if facingWater(game) and game.player.sips < player_rules.bottleSize(game.player) then
		return {
			label = "Fill bottle",
			run = function(g: types.Game)
				g.player.sips = player_rules.bottleSize(g.player)
				effects.burst(g.player.pos + combat.DIRECTIONS[g.player.facing], "water", 8)
				message(g, "You fill your bottle with fresh water.")
			end,
		}
	end
	return nil
end

--- Travel to an active teleporter (the key of its prop)
--- Offerings: a bog lily laid before the shrine of the Ebb (dropped from the bag) lights it, and the tide answers.
--- Called every frame.
function module.checkOfferings(game: types.Game)
	for _, pickup in game.pickups do
		if pickup.item == "bog_lily" and not pickup.dead then
			for _, prop in game.level.props do
				if prop.kind == "shrine" and (prop.cell + vec.V2(0.5, 0.6) - pickup.pos):length() < 2 then
					pickup.count -= 1
					pickup.dead = pickup.count <= 0
					switches.open(game, prop.data.id)
					table.insert(game.pickups, { pos = prop.cell + vec.V2(0.5, 1.4), item = prop.data.reward, count = 1, age = 0, dead = false })
					message(game, "You lay the lily before the basin. Clean water rises in it... and a pearl with it.")
				end
			end
		end
	end
end

function module.teleport(game: types.Game, key: string)
	local target = game.flags[`teleporter:{key}`]
	if not target then
		return
	end
	game_state.enterLevel(game, target.level, nil, game_state.cellCenter(vec.V2(target.x, target.y)))
	effects.flash(theme.colors.sanity, 0.6)
end

return module
