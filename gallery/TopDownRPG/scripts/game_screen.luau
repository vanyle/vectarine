--[[
The game screen: reads the keyboard, runs the game logic and draws everything.

Controls:
- arrows (or WASD): move. [Space]: attack with the equipped weapon (hold to keep attacking)
- [E]: interact (talk, open, rest, pick, read...). [Q] (A on AZERTY keyboards): notebook. [Escape]: pause

It also handles the flow of the game: transitions between levels, title cards, announcements, fainting,
dialogs and menus. The rules themselves are in game_logic.luau: this screen turns the keyboard into an `Input`
and turns `game.events` into things to show.
]]
const debug_tools = require("./debug_tools")
const dialog = require("./dialog")
const effects = require("./effects")
const game_logic = require("./game_logic")
const game_state = require("./game_state")
const hints = require("./hints")
const graphics = require("@vectarine/graphics")
const hud = require("./hud")
const interactions = require("./interactions")
const inventory = require("./inventory")
const io = require("@vectarine/io")
const items = require("./data/items")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const menus = require("./menus")
const monsters = require("./data/monsters")
const overlays = require("./overlays")
const persist = require("@vectarine/persist")
const player_rules = require("./player")
const postfx = require("./postfx")
const progress = require("./progress")
const prompts = require("./prompts")
const settings = require("./settings")
const theme = require("./theme")
const types = require("./types")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const world_view = require("./world_view")

const module = loader.init() or {}

--- The running game. It is kept when scripts are hot-reloaded. Set it to nil to start again from the save.
module.game = module.game :: types.Game?

--- What the screen is doing. Transitions close the screen, run `onMiddle`, then open it again.
type Flow =
	{ kind: "play" }
	| { kind: "transition", t: number, onMiddle: () -> (), done: boolean }
	| { kind: "fainted", t: number }
	| { kind: "floorEnd", t: number }

module.flow = module.flow or { kind = "play" } :: Flow
--- Seconds of play since the last autosave (see progress.luau)
module.sinceAutosave = module.sinceAutosave or 0

--- State of the screen that is not part of the game. It can be inspected in the Watcher.
const uiState = persist.onReload({
	title = nil :: { name: string, subtitle: string, t: number }?,
	message = nil :: { text: string, t: number }?,
	--- Banners shown one after the other (an artefact, then the level up it gave), never over a dialog, a menu or a title card
	banners = {} :: { { title: string, subtitle: string, icon: string?, t: number } },
	--- A menu to open once the dialog is closed (a shop opened by talking to a trader)
	pendingMenu = nil :: { kind: string, [string]: any }?,
}, "explorium_ui_state")
-- The state kept from before a reload may come from an older version of this file
uiState.banners = uiState.banners or {}

--- Duration of each half of a transition (closing, then opening)
const TRANSITION_HALF = 0.3
const FAINTED_DURATION = 2.6
const MESSAGE_DURATION = 2.8
const BANNER_DURATION = 3

const INTRO: { types.DialogStep } = {
	"You fell. Far.",
	"A bush broke your fall... and most of its branches.",
	"High above, the hole you fell through is a small circle of daylight. Far too high to climb.",
	"There must be another way out of here.",
}

--- Start the game from the save, or a new game if there is none.
function module.startGame()
	local saved = progress.load()
	local game = if saved then progress.restore(saved) else game_state.new()
	module.game = game
	module.flow = { kind = "play" }
	dialog.close()
	menus.close()
	effects.clear()
	uiState.title = nil
	uiState.message = nil
	uiState.banners = {}
	uiState.pendingMenu = nil
	if not saved then
		dialog.open(game, nil, INTRO)
	end
end

-- MARK: Layout

type Layout = {
	world: world_view.Viewport,
	hudPos: vec.Vec2,
	hudSize: vec.Vec2,
	--- The bar below the world, where the keyboard prompts are
	barPos: vec.Vec2,
	barSize: vec.Vec2,
	--- Where menus are drawn
	menuPos: vec.Vec2,
	menuSize: vec.Vec2,
}

--- The screen is split in 3: the HUD at the top, the world, and the prompts at the bottom.
--- Tiles are a multiple of 16 pixels (the size of the sprites) so that the pixel art stays crisp.
function module.layout(): Layout
	local window = io.getWindowSize()
	local hudHeight = 0.18
	local barHeight = 0.13
	local tilePx = 16 * math.max(1, math.floor(window.y / 200))
	local tileSize = vec.V2(tilePx * 2 / window.x, tilePx * 2 / window.y)
	local worldPos = vec.V2(-1, -1 + barHeight)
	local worldSize = vec.V2(2, 2 - hudHeight - barHeight)
	return {
		world = { pos = worldPos, size = worldSize, tileSize = tileSize },
		hudPos = vec.V2(-1, 1 - hudHeight),
		hudSize = vec.V2(2, hudHeight),
		barPos = vec.V2(-1, -1),
		barSize = vec.V2(2, barHeight),
		menuPos = vec.V2(-0.86, -0.78),
		menuSize = vec.V2(1.72, 1.42),
	}
end

-- MARK: Flow

local function startTransition(onMiddle: () -> ())
	module.flow = { kind = "transition", t = 0, onMiddle = onMiddle, done = false }
end

local function showMessage(text: string)
	uiState.message = { text = text, t = 0 }
end

local function showBanner(title: string, subtitle: string, icon: string?)
	table.insert(uiState.banners, { title = title, subtitle = subtitle, icon = icon, t = 0 })
end

local function openMenu(menu: { kind: string, [string]: any })
	if dialog.isOpen() then
		uiState.pendingMenu = menu
	else
		menus.open(menu)
	end
end

local function handleEvents(game: types.Game)
	local events = game.events
	game.events = {}
	for _, event in events do
		local kind = event.kind
		if kind == "message" then
			showMessage(event.text)
		elseif kind == "dialog" then
			dialog.open(game, event.speaker, event.steps)
		elseif kind == "shop" then
			openMenu({ kind = "shop", npc = event.npc })
		elseif kind == "storage" then
			openMenu({ kind = "storage", chest = event.chest })
		elseif kind == "teleport" then
			openMenu({ kind = "teleport", from = event.from })
		elseif kind == "enteredLevel" then
			local lvl = game.level
			uiState.title = { name = lvl.name, subtitle = lvl.subtitle, t = 0 }
			world_view.resetCamera(game, module.layout().world)
			-- The first time on a new floor of the Explorium
			if lvl.depth > (game.flags.deepestFloor or 1) then
				game.flags.deepestFloor = lvl.depth
				showBanner(levels.floorNames[lvl.depth] or lvl.subtitle, `Floor {lvl.depth} of the Explorium`, nil)
			end
			progress.save(game)
		elseif kind == "travel" then
			if not levels.levels[event.level] then
				showMessage("This way is blocked by fallen rocks.")
			elseif module.flow.kind == "play" then
				startTransition(function()
					game_state.enterLevel(game, event.level, event.entry, nil, event.index)
				end)
			end
		elseif kind == "fainted" and module.flow.kind ~= "fainted" then
			module.flow = { kind = "fainted", t = 0 }
			dialog.close()
			menus.close()
		elseif kind == "levelUp" then
			showBanner("Level up!", `Level {event.level}: spend your point in the notebook [{prompts.keyName(prompts.NOTEBOOK_KEY)}]`, nil)
		elseif kind == "artefact" then
			local item = items.items[event.item]
			showBanner(item.name, item.description, item.icon)
			progress.save(game)
		elseif kind == "rested" then
			showMessage("You rest by the fire. Your progress is saved.")
			progress.save(game)
		elseif kind == "killed" and event.boss then
			showBanner("Victory!", monsters.species[event.monster].victory or "", nil)
			progress.save(game)
		elseif kind == "pumpRestored" then
			-- The water flows again: the level is built again, with its canals full of water
			effects.shake(0.8)
			showBanner("The Great Pump wakes up!", "Water flows again in the canals and the cisterns of the desert.", "world/cistern_full")
			local pos = game.player.pos
			startTransition(function()
				game_state.enterLevel(game, game.levelId, nil, pos)
			end)
		elseif kind == "stairs" then
			module.flow = { kind = "floorEnd", t = 0 }
		end
	end
end

local function updateFlow(game: types.Game, dt: number)
	local flow = module.flow
	if flow.kind == "transition" then
		flow.t += dt
		postfx.state.iris = math.abs(flow.t - TRANSITION_HALF) / TRANSITION_HALF
		if not flow.done and flow.t >= TRANSITION_HALF then
			flow.done = true
			flow.onMiddle()
		end
		if flow.t >= TRANSITION_HALF * 2 then
			postfx.state.iris = 1
			module.flow = { kind = "play" }
		end
	elseif flow.kind == "fainted" then
		flow.t += dt
		postfx.state.desaturate = math.min(1, flow.t)
		if flow.t >= FAINTED_DURATION then
			postfx.state.desaturate = 0
			game_state.wakeUp(game)
			module.flow = { kind = "play" }
		end
	elseif flow.kind == "floorEnd" then
		flow.t += dt
	end
end

-- MARK: Input

local function readInput(): game_logic.Input
	local x = (if io.isKeyDown("Right") or io.isKeyDown("D") then 1 else 0) - (if io.isKeyDown("Left") or io.isKeyDown("A") then 1 else 0)
	local y = (if io.isKeyDown("Down") or io.isKeyDown("S") then 1 else 0) - (if io.isKeyDown("Up") or io.isKeyDown("W") then 1 else 0)
	return {
		move = vec.V2(x, y),
		attack = io.isKeyDown("Space"),
		interact = io.isKeyJustPressed("E"),
	}
end

local function handleInput(game: types.Game, dt: number, extra: any)
	if module.flow.kind == "floorEnd" then
		if module.flow.t > 2 and (io.isKeyJustPressed("Space") or io.isKeyJustPressed("Return")) then
			-- Back to the game: the player can keep exploring the oasis
			module.flow = { kind = "play" }
		end
		return
	end
	if module.flow.kind ~= "play" then
		return
	end
	if dialog.isOpen() then
		if io.isKeyJustPressed("Space") or io.isKeyJustPressed("Return") or io.isKeyJustPressed("E") then
			dialog.advance()
		end
		return
	end
	if menus.isOpen() then
		local action = menus.update(game)
		if action and action.kind == "travel" then
			startTransition(function()
				interactions.teleport(game, action.key)
			end)
		elseif action and action.kind == "quit" then
			progress.save(game)
			module.game = nil
			postfx.state.danger = 0
			postfx.state.thirst = 0
			extra.currentScreen = ""
		end
		return
	end
	if io.isKeyJustPressed(prompts.NOTEBOOK_KEY) then
		menus.open({ kind = "notebook" })
		return
	elseif io.isKeyJustPressed("Escape") then
		menus.open({ kind = "pause" })
		return
	end
	game_logic.update(game, readInput(), dt)
	module.sinceAutosave += dt
	if module.sinceAutosave >= progress.AUTOSAVE_INTERVAL and progress.canAutosave(game) then
		module.sinceAutosave = 0
		progress.save(game)
	end
end

-- MARK: Drawing

local function drawWidgetAt(widget: ui.Widget<any>, pos: vec.Vec2, extra: any)
	graphics.withTransformation({ translation = pos - vec.V2(-1, -1) }, function()
		widget:draw(extra)
	end)
end

local function currentPrompts(game: types.Game): { prompts.Prompt }
	if module.flow.kind == "floorEnd" then
		return { { key = "space", label = "Keep exploring" } }
	elseif module.flow.kind ~= "play" then
		return {}
	elseif dialog.isOpen() then
		return { { key = "space", label = "Next" } }
	elseif menus.isOpen() then
		return menus.prompts(game)
	end
	local weapon = items.items[inventory.weapon(game.player)]
	local interaction = interactions.find(game)
	return prompts.forPlay(if weapon.shot then "Shoot" else "Attack", if interaction then interaction.label else nil)
end

local function drawPrompts(game: types.Game, layout: Layout, extra: any)
	graphics.drawRect(layout.barPos, layout.barSize, theme.colors.background)
	graphics.drawRect(layout.barPos + vec.V2(0, layout.barSize.y - 0.006), vec.V2(2, 0.006), theme.colors.goldDark)
	if not settings.values.showPrompts then
		return
	end
	local bar = prompts.widget(currentPrompts(game))
	drawWidgetAt(bar, layout.barPos + (layout.barSize - bar:size({})):scale(0.5), extra)
end

--- Draw the whole level in the world view, without darkness: a tool to review levels (`explorium.debug.overview`)
local function drawOverview(game: types.Game, layout: Layout)
	local window = io.getWindowSize()
	local area = layout.world
	local areaPx = vec.V2(area.size.x * window.x / 2, area.size.y * window.y / 2)
	local tilePx = math.min(areaPx.x / game.level.width, areaPx.y / game.level.height)
	local saved = world_view.camera
	world_view.camera = vec.V2(0, 0)
	world_view.drawWorld(game, {
		pos = area.pos,
		size = area.size,
		tileSize = vec.V2(tilePx * 2 / window.x, tilePx * 2 / window.y),
	}, { time = ui_components.time, darkness = false })
	world_view.camera = saved
end

local function drawScreen(game: types.Game, layout: Layout, extra: any, dt: number)
	graphics.clear(theme.colors.background)
	local debugHelpers = (module :: any).debug
	if debugHelpers and debugHelpers.overview then
		drawOverview(game, layout)
	else
		world_view.drawWorld(game, layout.world, { time = ui_components.time })
	end

	local title = uiState.title
	if title then
		title.t += dt
		overlays.drawTitleCard(title.name, title.subtitle, vec.V2(0, layout.world.pos.y + layout.world.size.y - 0.22), title.t)
		if title.t > overlays.TITLE_DURATION then
			uiState.title = nil
		end
	end
	local banner = uiState.banners[1]
	if banner and not dialog.isOpen() and not menus.isOpen() and not uiState.title then
		banner.t += dt
		overlays.drawBanner(banner.title, banner.subtitle, banner.icon, banner.t, BANNER_DURATION)
		if banner.t > BANNER_DURATION then
			table.remove(uiState.banners, 1)
		end
	end
	-- The health bar of a boss is at the bottom of the world: messages go above it
	local bossBar = hud.drawBossBar(game, layout.world.pos.y)
	local message = uiState.message
	local messageBottom = layout.world.pos.y + (if bossBar then 0.13 else 0)
	if message then
		message.t += dt
		overlays.drawMessage(message.text, messageBottom, message.t, MESSAGE_DURATION)
		if message.t > MESSAGE_DURATION then
			uiState.message = nil
		end
	elseif not dialog.isOpen() and not menus.isOpen() then
		-- A hint about what blocks the way stays while the player is next to it
		local hint = hints.current(game)
		if hint then
			overlays.drawMessage(hint, messageBottom, 1, math.huge)
		end
	end

	hud.draw(game, layout.hudPos, layout.hudSize)
	hud.drawGallery(layout.world.pos.y + layout.world.size.y)
	drawPrompts(game, layout, extra)

	if dialog.isOpen() then
		local size = dialog.BOX_SIZE
		drawWidgetAt(dialog.widget, vec.V2(-size.x / 2, layout.world.pos.y + 0.03), extra)
	end
	if menus.isOpen() then
		menus.draw(game, { pos = layout.menuPos, size = layout.menuSize })
	end

	local flow = module.flow
	if flow.kind == "fainted" then
		overlays.drawFainted(flow.t)
	elseif flow.kind == "floorEnd" then
		overlays.drawFloorEnd(flow.t)
	end
end

-- Low sanity blurs the mind: colors start to split
local function updatePostfx(game: types.Game, dt: number)
	local player = game.player
	local fraction = player.sanity / player_rules.maxSanity(player)
	if fraction < 0.3 then
		postfx.state.aberration = math.max(postfx.state.aberration, (0.3 - fraction) * 2 + math.sin(game.time * 3) * 0.1)
	end
	-- Red edges below 35% of health, beating like a heart. Yellow edges below 25% of water.
	local health = player.hp / player_rules.maxHp(player)
	local heartbeat = 0.75 + 0.25 * math.sin(game.time * 6)
	postfx.state.danger = math.clamp((0.35 - health) / 0.35, 0, 1) * heartbeat
	postfx.state.thirst = math.clamp((25 - player.water) / 25, 0, 1)
	-- Frost on the edges below 35% of warmth
	postfx.state.cold = math.clamp((35 - player.warmth) / 35, 0, 1)
	postfx.update(dt)
end

debug_tools.install(module)

module.gameScreenWidget = ui.widget(vec.V2(2, 2), function(_events, extra)
	if not module.game then
		if extra.currentScreen ~= "game" then
			-- The screen is sliding away after quitting: don't start a new game yet.
			graphics.clear(theme.colors.background)
			return
		end
		module.startGame()
	end
	local game = module.game :: types.Game
	local dt: number = extra.dt or 1 / 60
	local layout = module.layout()

	handleInput(game, dt, extra)
	if module.game ~= game then
		return -- The player quit this frame
	end
	handleEvents(game)
	-- A menu waiting for the end of a dialog (the shop of a trader)
	if uiState.pendingMenu and not dialog.isOpen() then
		menus.open(uiState.pendingMenu :: any)
		uiState.pendingMenu = nil
	end
	updateFlow(game, dt)
	dialog.update(dt)
	if not menus.isOpen() then
		effects.update(dt)
	end
	effects.updateScreen(dt)
	world_view.updateCamera(game, layout.world, dt)
	updatePostfx(game, dt)
	drawScreen(game, layout, extra, dt)
end)

return module
