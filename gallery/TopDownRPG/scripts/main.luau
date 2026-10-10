--[[
Switches between the screens of the game: title, settings, credits and the game itself.

Screens change `extra.currentScreen` (extra is this module) to navigate.
]]
const creditsScreen = require("./credits_screen")
const gameScreen = require("./game_screen")
const graphics = require("@vectarine/graphics")
const loader = require("@vectarine/loader")
const postfx = require("./postfx")
const settingsScreen = require("./settings_screen")
const sprites = require("./sprites")
const theme = require("./theme")
const titleScreen = require("./title_screen")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
local vec = require("@vectarine/vec")

const module = loader.init() or {}

module.currentScreen = module.currentScreen or ""
--- Duration of the current frame, available to the screens as `extra.dt`
module.dt = 1 / 60

const interfaceWidget = ui.tabs({
	[""] = ui.widget(vec.V2(2, 2), function(events, extra)
		titleScreen.titleScreenWidget:draw(extra)
	end),
	settings = ui.widget(vec.V2(2, 2), function(events, extra)
		settingsScreen.settingsScreenWidget:draw(extra)
	end),
	credits = ui.widget(vec.V2(2, 2), function(events, extra)
		creditsScreen.creditsScreenWidget:draw(extra)
	end),
	game = ui.widget(vec.V2(2, 2), function(events, extra)
		gameScreen.gameScreenWidget:draw(extra)
	end),
})

--- Images, tilesets and fonts needed before anything can be drawn
local requiredResources = sprites.resources()
table.insert(requiredResources, theme.font :: any)
table.insert(requiredResources, postfx.shader :: any)

function module.Update(deltaTime: number)
	if graphics.drawSplashScreenIfNeeded(requiredResources) then
		return
	end
	module.dt = deltaTime
	ui_components.tick(deltaTime)
	-- Everything is drawn through the post-processing shader (CRT effect, flashes, transitions)
	postfx.render(function()
		interfaceWidget:draw(module)
	end)
	if module.currentScreen ~= interfaceWidget:getActiveTab() then
		interfaceWidget:setActiveTab(module.currentScreen, { duration = 0.3, animationType = "slideLeft" })
	end
end

return module
