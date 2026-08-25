--[[
The game screen: reads the keyboard, runs the game logic and draws everything.

Controls:
- arrows: move. [Space]: talk, read, or use the selected item (it waits a turn when there is nothing to do)
- [X]: switch item. [Z]: undo. [R]: restart the level. [Escape]: leave the level / pause

It also handles the flow of the game: entering and leaving levels with a transition,
the "gear found" celebration, restarting after dying, story beats and the ending.
]]
const debug_tools = require("./debug_tools")
const dialog = require("./dialog")
const effects = require("./effects")
const game_state = require("./game_state")
const graphics = require("@vectarine/graphics")
const hud = require("./hud")
const io = require("@vectarine/io")
const items = require("./data/items")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const npcs = require("./data/npcs")
const overlays = require("./overlays")
const persist = require("@vectarine/persist")
const postfx = require("./postfx")
const prompts = require("./prompts")
const rooms = require("./data/rooms")
const rules = require("./rules")
const settings = require("./settings")
const story = require("./data/story")
const theme = require("./theme")
const types = require("./types")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const world = require("./world")
const world_view = require("./world_view")

const module = loader.init() or {}

--- The running game. It is kept when scripts are hot-reloaded. Set it to nil to start a new game.
module.game = module.game :: types.Game?

--- What the screen is doing. Transitions close the screen, run `onMiddle`, then open it again.
type Flow =
	{ kind: "play" }
	| { kind: "transition", t: number, onMiddle: () -> (), done: boolean }
	| { kind: "levelClear", t: number }
	| { kind: "ending", t: number }

module.flow = module.flow or { kind = "play" } :: Flow

--- State of the screen that is not part of the game rules. It can be inspected in the Watcher.
const uiState = persist.onReload({
	--- Time since entering the current room, to show its name
	roomTime = 0,
	shownRoom = "",
	--- Time before a held key does its action again
	repeatTimer = 0,
	--- An action requested while an animation was playing, done when the animation ends
	bufferedAction = nil :: string?,
	deathTime = 0,
}, "ui_state")

const MOVE_KEYS: { { key: string, action: rules.Action } } = {
	{ key = "Up", action = "up" },
	{ key = "Down", action = "down" },
	{ key = "Left", action = "left" },
	{ key = "Right", action = "right" },
}
--- Delay before a held key starts repeating, then delay between repeats (in seconds)
const FIRST_REPEAT_DELAY = 0.22
const REPEAT_DELAY = 0.12
--- Duration of each half of a transition (closing, then opening)
const TRANSITION_HALF = 0.35
--- Time before the level restarts after dying
const DEATH_DELAY = 1.0
const LEVEL_CLEAR_DURATION = 1.6

function module.startNewGame()
	local game = game_state.new()
	module.game = game
	module.flow = { kind = "play" }
	dialog.close()
	effects.clear()
	world_view.resetCamera(game)
	uiState.shownRoom = ""
	uiState.bufferedAction = nil
	-- The story starts with the fairy talking to the player
	local fairy = npcs.npcs.fairy
	game:startDialog({ name = fairy.name, sprite = fairy.sprite }, fairy.talk(game))
end

-- MARK: Layout

type Layout = {
	world: world_view.Viewport,
	hudPos: vec.Vec2,
	hudSize: vec.Vec2,
	--- The bar below the map, where the keyboard prompts are
	barPos: vec.Vec2,
	barSize: vec.Vec2,
}

--- The screen is split in 3: the HUD at the top, the room, and the prompts at the bottom.
--- When possible, tiles are a multiple of 16 pixels (the size of the sprites) so that the pixel art stays crisp.
local function computeLayout(): Layout
	local window = io.getWindowSize()
	local tilePx = math.min(window.x / rooms.ROOM_WIDTH, (window.y - 72) / rooms.ROOM_HEIGHT)
	local crispTilePx = math.floor(tilePx / 16) * 16
	if crispTilePx >= tilePx * 0.9 then
		tilePx = crispTilePx
	end
	local tileSize = vec.V2(tilePx * 2 / window.x, tilePx * 2 / window.y)
	local mapSize = tileSize * vec.V2(rooms.ROOM_WIDTH, rooms.ROOM_HEIGHT)
	local free = 2 - mapSize.y
	local barHeight = free * 0.42
	return {
		world = { pos = vec.V2(-mapSize.x / 2, -1 + barHeight), size = mapSize, tileSize = tileSize },
		hudPos = vec.V2(-1, -1 + barHeight + mapSize.y),
		hudSize = vec.V2(2, free - barHeight),
		barPos = vec.V2(-1, -1),
		barSize = vec.V2(2, barHeight),
	}
end

-- MARK: Flow

local function startTransition(onMiddle: () -> ())
	module.flow = { kind = "transition", t = 0, onMiddle = onMiddle, done = false }
end

local function enterLevel(game: types.Game, index: number)
	if not levels.levels[index] then
		game:say(game.player.pos, "This ladder is broken", theme.colors.textDim)
		return
	end
	startTransition(function()
		game_state.enterLevel(game, index)
		world_view.resetCamera(game)
	end)
end

local function leaveLevel(game: types.Game, showStory: boolean)
	local beat = if game.level then story.afterLevel[game.level.index] else nil
	startTransition(function()
		game_state.leaveLevel(game)
		world_view.resetCamera(game)
		if showStory and beat then
			game:startDialog(beat(game))
		end
	end)
end

local function restartLevel(game: types.Game)
	game_state.restartLevel(game)
	postfx.state.aberration = 0.8
	effects.clear()
	uiState.deathTime = 0
end

local function undo(game: types.Game)
	if game_state.undo(game) then
		postfx.state.aberration = 0.5
		uiState.deathTime = 0
	end
end

local function updateFlow(game: types.Game, dt: number)
	local flow = module.flow
	if flow.kind == "transition" then
		flow.t += dt
		if not flow.done and flow.t >= TRANSITION_HALF then
			flow.done = true
			flow.onMiddle()
		end
		if flow.t >= TRANSITION_HALF * 2 then
			module.flow = { kind = "play" }
		end
	elseif flow.kind == "levelClear" then
		flow.t += dt
		if flow.t >= LEVEL_CLEAR_DURATION then
			local firstTime = not game.finished[game.level and game.level.index or 0]
			game_state.finishLevel(game)
			module.flow = { kind = "play" }
			leaveLevel(game, firstTime)
		end
	elseif flow.kind == "ending" then
		flow.t += dt
	elseif game.level and game.level.won then
		module.flow = { kind = "levelClear", t = 0 }
		effects.shake(0.2)
		effects.flash(theme.colors.gold, 0.4)
	elseif game.enteringLevel then
		local index = game.enteringLevel
		game.enteringLevel = nil
		enterLevel(game, index)
	elseif game.status == "ending" and not dialog.isOpen() then
		module.flow = { kind = "ending", t = 0 }
	elseif game.player.dead then
		uiState.deathTime += dt
		if uiState.deathTime >= DEATH_DELAY then
			startTransition(function()
				restartLevel(game)
			end)
		end
	end
end

-- MARK: Input

--- Returns true once when the key is pressed, then repeatedly while it is held down.
local function isKeyRepeated(key: string, dt: number): boolean
	if io.isKeyJustPressed(key) then
		uiState.repeatTimer = FIRST_REPEAT_DELAY
		return true
	elseif io.isKeyDown(key) then
		uiState.repeatTimer -= dt
		if uiState.repeatTimer <= 0 then
			uiState.repeatTimer = REPEAT_DELAY
			return true
		end
	end
	return false
end

local function readAction(dt: number): rules.Action?
	for _, move in MOVE_KEYS do
		if isKeyRepeated(move.key, dt) then
			return move.action
		end
	end
	if io.isKeyJustPressed("Space") or io.isKeyJustPressed("Return") then
		return "use"
	end
	return nil
end

local function handleInput(game: types.Game, dt: number, extra: any)
	local flow = module.flow
	local confirm = io.isKeyJustPressed("Space") or io.isKeyJustPressed("Return")
	if flow.kind == "ending" then
		if flow.t > 1.5 and confirm then
			module.game = nil
			extra.currentScreen = ""
		end
		return
	elseif flow.kind ~= "play" then
		return
	elseif dialog.isOpen() then
		if confirm then
			dialog.advance()
		end
		return
	elseif io.isKeyJustPressed("Escape") then
		if game.level then
			leaveLevel(game, false)
		else
			extra.currentScreen = ""
		end
		return
	end
	if game.level then
		if isKeyRepeated("Z", dt) then
			undo(game)
			return
		elseif io.isKeyJustPressed("R") then
			restartLevel(game)
			return
		end
	end
	if game.player.dead then
		return
	end

	local action = readAction(dt)
	if world_view.isAnimating(game) then
		if action then
			uiState.bufferedAction = action
		end
		return
	end
	action = action or uiState.bufferedAction :: rules.Action?
	uiState.bufferedAction = nil
	if action then
		rules.act(game, action :: rules.Action)
	end
end

-- MARK: Drawing

--- What [Space] does right now
local function actionLabel(game: types.Game): string?
	local entity = game:entityAt(game.player.pos + types.directions[game.player.facing])
	if entity and entity.kind == "npc" then
		return "Talk"
	elseif entity and entity.kind == "sign" then
		return "Read"
	elseif game.level then
		local item = game_state.itemForSpace(game)
		return if item then items.items[item].verb else "Wait"
	end
	return nil
end

--- Draw a widget with its bottom-left corner at `pos`
local function drawWidgetAt(widget: ui.Widget<any>, pos: vec.Vec2, extra: any)
	graphics.withTransformation({ translation = pos - vec.V2(-1, -1) }, function()
		widget:draw(extra)
	end)
end

local function drawPrompts(game: types.Game, layout: Layout, extra: any)
	graphics.drawRect(layout.barPos, layout.barSize, theme.colors.background)
	graphics.drawRect(layout.barPos + vec.V2(0, layout.barSize.y - 0.006), vec.V2(2, 0.006), theme.colors.goldDark)
	if not settings.values.showPrompts or dialog.isOpen() or module.flow.kind ~= "play" then
		return
	end
	local bar = prompts.widget(prompts.forContext({
		mode = if game.player.dead then "dead" elseif game.level then "level" else "explore",
		actionLabel = actionLabel(game),
	}))
	drawWidgetAt(bar, layout.barPos + (layout.barSize - bar:size({})):scale(0.5), extra)
end

local function drawScreen(game: types.Game, layout: Layout, extra: any)
	local viewport = layout.world
	graphics.clear(theme.colors.background)
	world_view.drawWorld(game, viewport, world_view.camera.pos, { time = ui_components.time, showPlayer = true })
	hud.draw(game, layout.hudPos, layout.hudSize)

	local room = world.roomData(game.currentRoom)
	local subtitle = if game.level then `Level {game.level.index}` else room.subtitle
	overlays.drawRoomName(room.name, subtitle, viewport.pos + viewport.size * vec.V2(0.5, 0.62), uiState.roomTime)

	local flow = module.flow
	if flow.kind == "levelClear" then
		overlays.drawLevelClear(flow.t, LEVEL_CLEAR_DURATION)
	end
	drawPrompts(game, layout, extra)

	if dialog.isOpen() then
		local size = dialog.widget:size({})
		drawWidgetAt(dialog.widget, vec.V2(-size.x / 2, viewport.pos.y + 0.05), extra)
	end

	if flow.kind == "ending" then
		overlays.drawEnding(game.turn, flow.t)
		if flow.t > 1.5 then
			local bar = prompts.widget(prompts.forContext({ mode = "ending" }))
			drawWidgetAt(bar, vec.V2(-bar:size({}).x / 2, -0.8), extra)
		end
	end
end

--- Transitions and deaths are shown with the post-processing shader.
local function updatePostfx(game: types.Game, layout: Layout, dt: number)
	local state = postfx.state
	local flow = module.flow
	state.iris = if flow.kind == "transition"
		then math.clamp(math.abs(flow.t - TRANSITION_HALF) / TRANSITION_HALF, 0, 1)
		else 1
	state.irisCenter = world_view.toScreen(layout.world, world_view.camera.pos, game.player.drawPos + vec.V2(0.5, 0.5))
	local targetDesaturate = if game.player.dead then 0.85 else 0
	state.desaturate += (targetDesaturate - state.desaturate) * (1 - math.exp(-dt * 8))
	postfx.update(dt)
end

debug_tools.install(module)

module.gameScreenWidget = ui.widget(vec.V2(2, 2), function(_events, extra)
	if not module.game then
		if extra.currentScreen ~= "game" then
			-- The screen is sliding away after the end of a game: don't start a new one yet.
			graphics.clear(theme.colors.background)
			return
		end
		module.startNewGame()
	end
	local game = module.game :: types.Game
	local dt: number = extra.dt or 1 / 60

	if world.isOutdated(game.world) then
		game_state.rebuildWorld(game)
		world_view.resetCamera(game)
	end
	handleInput(game, dt, extra)
	if module.game ~= game then
		return -- The game ended this frame
	end
	updateFlow(game, dt)

	uiState.roomTime += dt
	if uiState.shownRoom ~= game.currentRoom then
		uiState.shownRoom = game.currentRoom
		uiState.roomTime = 0
	end
	world_view.updateCamera(game, dt)
	world_view.animate(game, dt)
	effects.update(dt)
	dialog.update(dt)

	local layout = computeLayout()
	updatePostfx(game, layout, dt)
	drawScreen(game, layout, extra)
end)

return module
