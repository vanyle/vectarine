const game_screen = require("./game_screen")
const loader = require("@vectarine/loader")
const menu_backdrop = require("./menu_backdrop")
const settings = require("./settings")
const theme = require("./theme")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

local function onOff(value: boolean): string
	return if value then "ON" else "OFF"
end

--- Text speeds the player can cycle through, in characters per second
const TEXT_SPEEDS = {
	{ name = "Slow", speed = 25 },
	{ name = "Normal", speed = 45 },
	{ name = "Fast", speed = 90 },
}

local function currentTextSpeedIndex(): number
	for index, option in TEXT_SPEEDS do
		if option.speed == settings.values.textSpeed then
			return index
		end
	end
	return 2
end

local menu = ui_components.menu({
	{
		label = function()
			return `Key hints: {onOff(settings.values.showPrompts)}`
		end,
		action = function()
			settings.values.showPrompts = not settings.values.showPrompts
			settings.save()
		end,
	},
	{
		label = function()
			return `Text speed: {TEXT_SPEEDS[currentTextSpeedIndex()].name}`
		end,
		action = function()
			local next = currentTextSpeedIndex() % #TEXT_SPEEDS + 1
			settings.values.textSpeed = TEXT_SPEEDS[next].speed
			settings.save()
		end,
	},
	{
		label = function()
			return `Screen shake: {onOff(settings.values.screenShake)}`
		end,
		action = function()
			settings.values.screenShake = not settings.values.screenShake
			settings.save()
		end,
	},
	{
		label = function()
			return `Fullscreen: {onOff(settings.values.fullscreen)}`
		end,
		action = function()
			settings.toggleFullscreen()
		end,
	},
	{
		label = "Restart the game",
		action = function(extra: any)
			game_screen.game = nil
			extra.currentScreen = "game"
		end,
	},
	{
		label = "Back",
		icon = "icons/back",
		action = function(extra: any)
			extra.currentScreen = ""
		end,
	},
}, vec.V2(0.8, 0.11))

module.settingsScreenWidget = ui.stack({ alignX = "center", alignY = "center" }, {
	ui.widget(vec.V2(2, 2), function()
		menu_backdrop.draw()
	end),
	ui.column({ align = "center" }, {
		ui.text("SETTINGS", { fontSize = 0.2, font = theme.font, color = theme.colors.gold }),
		ui.spacer(vec.V2(0, 0.1)),
		menu,
		ui.spacer(vec.V2(0, 0.1)),
	}),
})

return module
