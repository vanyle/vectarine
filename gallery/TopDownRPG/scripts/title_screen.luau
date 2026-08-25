const game_screen = require("./game_screen")
const loader = require("@vectarine/loader")
const menu_backdrop = require("./menu_backdrop")
const theme = require("./theme")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- Big text with a drop shadow, typical of retro title screens
local function shadowedText(text: string, fontSize: number, color: vec4.Vec4): ui.Widget<any>
	local shadowOffset = fontSize * 0.06
	return ui.stack({ alignX = "center", alignY = "center" }, {
		ui.column({}, {
			ui.spacer(vec.V2(0, shadowOffset)),
			ui.row({}, {
				ui.spacer(vec.V2(shadowOffset, 0)),
				ui.text(text, { fontSize = fontSize, font = theme.font, color = theme.colors.goldDark }),
			}),
		}),
		ui.column({}, {
			ui.row({}, {
				ui.text(text, { fontSize = fontSize, font = theme.font, color = color }),
				ui.spacer(vec.V2(shadowOffset, 0)),
			}),
			ui.spacer(vec.V2(0, shadowOffset)),
		}),
	})
end

local menu = ui_components.menu({
	{
		label = function()
			return if game_screen.game then "Continue" else "Play"
		end,
		icon = "icons/play",
		action = function(extra: any)
			extra.currentScreen = "game"
		end,
	},
	{
		label = "Settings",
		icon = "icons/gear",
		action = function(extra: any)
			extra.currentScreen = "settings"
		end,
	},
	{
		label = "Credits",
		icon = "icons/group",
		action = function(extra: any)
			extra.currentScreen = "credits"
		end,
	},
}, vec.V2(0.62, 0.13))

module.titleScreenWidget = ui.stack({ alignX = "center", alignY = "center" }, {
	ui.widget(vec.V2(2, 2), function()
		menu_backdrop.draw()
	end),
	ui.column({ align = "center" }, {
		shadowedText("TOP DOWN RPG", 0.3, theme.colors.gold),
		ui.text("~ The Stopped Clock ~", { fontSize = 0.085, font = theme.font, color = theme.colors.text }),
		ui.spacer(vec.V2(0, 0.14)),
		menu,
		ui.spacer(vec.V2(0, 0.2)),
		ui.text("Made with Vectarine", { fontSize = 0.055, font = theme.font, color = theme.colors.textDim }),
	}),
})

return module
