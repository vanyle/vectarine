local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Screen = require("@vectarine/screen")

local _menuResource, menu = Loader.loadScript("scripts/menu.luau", require("menu.luau"))
local _gameResource, game = Loader.loadScript("scripts/gameplay.luau", require("gameplay.luau"))

Io.setResizeable(true)

function Load() end

function handleScreenTransitions(screen: Screen.Screen)
	if screen:name() == "Menu" and Io.isKeyDown("Space") then
		-- Transition to game screen with fade effect
		Screen.setCurrentScreen(game.screen, {
			duration = 1.0,
			transition_style = "slide_up",
		})
	elseif screen:name() == "Game" and Io.isKeyDown("Escape") then
		-- Transition back to menu with slide effect
		Screen.setCurrentScreen(menu.screen, {
			duration = 1.0,
			transition_style = "toon",
		})
	end
end

function Update(time_delta: number)
	_G.deltaTime = time_delta

	local screen = Screen.getCurrentScreen()
	if screen ~= nil then
		handleScreenTransitions(screen)
	end

	Screen.drawCurrentScreen()
end
