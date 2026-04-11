local graphics = require("@vectarine/graphics")
local io = require("@vectarine/io")
local loader = require("@vectarine/loader")
local persist = require("@vectarine/persist")
local ui = require("@vectarine/ui")

local menuResource, menu = loader.loadScript("scripts/menu.luau", require("menu.luau"))
local gameResource, game = loader.loadScript("scripts/gameplay.luau", require("gameplay.luau"))

io.setResizeable(true)

local isReady = persist.onReload({ isReady = false }, "isReady")
local currentScreen: { currentScreen: string? } = persist.onReload({ currentScreen = "Menu" }, "currentScreen")

function initRootWidget()
	local rootWidget = ui.tabs({
		["Menu"] = menu.widget,
		["Game"] = game.widget,
	})
	rootWidget:setActiveTab("Menu")
	return rootWidget
end

local rootWidget = initRootWidget()

function Load()
	rootWidget = initRootWidget()
end

function handleScreenTransitions()
	if currentScreen.currentScreen == "Menu" and io.isKeyDown("Space") then
		-- Transition to game screen with fade effect
		rootWidget:setActiveTab("Game", {
			duration = 1.0,
			animationType = "slideUp",
		})
		currentScreen.currentScreen = "Game"
	elseif currentScreen.currentScreen == "Game" and io.isKeyDown("Escape") then
		-- Transition back to menu with slide effect
		rootWidget:setActiveTab("Menu", {
			duration = 1.0,
			animationType = "toon",
		})
		currentScreen.currentScreen = "Menu"
	end
end

function Update(time_delta: number)
	if graphics.drawSplashScreenIfNeeded({ menuResource, gameResource }) then
		return
	end
	if not isReady.isReady then
		Load()
		isReady.isReady = true
	end

	_G.deltaTime = time_delta

	handleScreenTransitions()
	rootWidget:draw()
end
