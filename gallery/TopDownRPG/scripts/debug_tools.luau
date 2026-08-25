--[[
Helpers to test the game quickly, available as `rpg.debug` in automated tests
and as commands in the editor console ("level 3", "finish 9", "visit town").
]]
const debug = require("@vectarine/debug")
const dialog = require("./dialog")
const event = require("@vectarine/event")
const game_state = require("./game_state")
const items = require("./data/items")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const rules = require("./rules")
const types = require("./types")
const vec = require("@vectarine/vec")
const world_view = require("./world_view")

const module = loader.init() or {}

--- Characters used by `render` for the things that live on tiles
const ENTITY_CHARS: { [string]: string } = {
	rat = "r",
	bat = "b",
	spider = "s",
	gear = "$",
	key = "k",
	arrows = "a",
	bombs = "q",
	leap = "h",
	slant = "j",
	clear = "c",
	barrel = "x",
	door = "L",
	sign = "?",
	npc = "&",
	ladder = "=",
}

--- Closed gates are shown with the character of the legend: G (plates), Q (targets), N (monsters)
const GATE_CHARS: { [string]: string } = { plates = "G", targets = "Q", monsters = "N", levels = "|" }

--- The current room as text, one character per tile, for tests and agents that can't see the screen.
function module.render(game: types.Game): string
	local room = game.world.rooms[game.currentRoom]
	local lines: { string } = {}
	for y = 0, 10 do
		local line: { string } = {}
		for x = 0, 15 do
			local pos = room.origin + vec.V2(x, y)
			local ground, object = game:groundAt(pos), game:objectAt(pos)
			local char = "."
			if ground and ground.deadly == "fall" then
				char = "_"
			elseif ground and ground.plate then
				char = if game_state.isPressed(game, pos) then "*" else "+"
			elseif ground and ground.solid then
				char = "~"
			end
			if object and object.solid then
				char = if object.breaksWith and object.breaksWith.mine then "R"
					elseif object.breaksWith then "C"
					else "#"
			elseif object and object.deadly then
				char = "^"
			end
			for _, entity in game_state.entitiesAt(game, pos) do
				local open = entity.kind == "gate" and game_state.isGateOpen(game, entity)
				if entity.kind == "gate" and entity.spikes then
					char = if open then "," else "^"
				elseif entity.kind == "gate" then
					char = if open then ":" else GATE_CHARS[entity.condition or "plates"]
				elseif entity.kind == "target" then
					char = if entity.hit then "T" else "t"
				elseif entity.kind == "bomb" then
					char = tostring(entity.fuse)
				else
					char = ENTITY_CHARS[entity.monster or entity.potion or entity.pickup or entity.kind] or char
				end
			end
			if game.player.pos.x == pos.x and game.player.pos.y == pos.y then
				char = if game.player.dead then "X" else "@"
			end
			table.insert(line, char)
		end
		table.insert(lines, table.concat(line))
	end
	local inventory = game.inventory
	table.insert(
		lines,
		`turn {game.level and game.level.moves or game.turn}, facing {game.player.facing}, potion {game.player.effect or "none"}, `
			.. `arrows {inventory.arrows}, bombs {inventory.bombs}, keys {inventory.keys}, pickaxe {inventory.hasPickaxe}, `
			.. `{if game.player.dead then "DEAD (" .. (game.player.deathCause or "") .. ")" elseif game.level and game.level.won then "WON" else "playing"}`
	)
	return table.concat(lines, "\n")
end

--- Play actions separated by spaces ("up up right bow"): up, down, left, right, wait, pickaxe, bow, bomb.
--- Items are used with [Space] like a player would, so the item must be the one [Space] uses at that moment.
function module.play(game: types.Game, actions: string)
	for _, action in string.split(actions, " ") do
		if items.items[action] then
			local item = game_state.itemForSpace(game)
			assert(item == action, `[Space] would use the {item or "nothing"} here, not the {action}`)
			rules.act(game, "use")
		elseif action ~= "" then
			rules.act(game, action :: any)
		end
	end
end

--- `screen` is the game screen module: its `game` field is the running game.
function module.install(screen: { game: types.Game? })
	local function withGame(action: (game: types.Game) -> ())
		if screen.game then
			action(screen.game)
		end
	end

	local commands = {
		--- Teleport the player to a room of the overworld, at the given tile of the room
		visit = function(roomId: string, x: number?, y: number?)
			withGame(function(game)
				local room = game.world.rooms[roomId]
				assert(room, `Unknown room '{roomId}'`)
				game.level = nil
				game.currentRoom = roomId
				game.player.pos = room.origin + vec.V2(x or 7, y or 5)
				game.player.drawPos = game.player.pos
				game_state.refreshConditions(game)
				world_view.resetCamera(game)
			end)
		end,
		--- Enter a level directly, without a transition
		level = function(index: number)
			withGame(function(game)
				dialog.close()
				game_state.enterLevel(game, index)
				world_view.resetCamera(game)
			end)
		end,
		--- Play actions separated by spaces, like the solutions in levels.luau
		play = function(actions: string)
			withGame(function(game)
				module.play(game, actions)
			end)
		end,
		--- Play the solution of the current level (see levels.luau)
		playSolution = function()
			withGame(function(game)
				if game.level then
					module.play(game, levels.levels[game.level.index].solution)
				end
			end)
		end,
		--- Print the current room as text (see `module.render`)
		print = function()
			withGame(function(game)
				print(module.render(game))
			end)
		end,
		isDialogOpen = function(): boolean
			return dialog.isOpen()
		end,
		--- Go through the current dialog until the end, running its actions (giving items, ...)
		finishDialog = function()
			for _ = 1, 200 do
				if not dialog.isOpen() then
					return
				end
				dialog.advance()
			end
		end,
		--- Mark levels 1 to n as finished
		finish = function(count: number)
			withGame(function(game)
				for index = 1, count do
					game.finished[index] = true
				end
				game.flags.metFairy = true
				game.inventory.hasPickaxe = true
			end)
		end,
	}
	;(screen :: any).debug = commands

	event.getConsoleCommandEvent():on("rpgDebugCommands", function(command: string)
		local name, argument = table.unpack(string.split(command, " "))
		if name == "visit" and argument then
			commands.visit(argument)
		elseif name == "level" and argument then
			commands.level(tonumber(argument) or 1)
		elseif name == "finish" and argument then
			commands.finish(tonumber(argument) or 0)
		else
			debug.print("Commands: visit <room>, level <number>, finish <number of levels>")
		end
	end)

	-- Automated tests and the Watcher reach the game through this global.
	_G.rpg = screen
end

return module
