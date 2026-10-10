const loader = require("@vectarine/loader")
const menu_backdrop = require("./menu_backdrop")
const theme = require("./theme")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- Each credit is a role and who did it.
const CREDITS = {
	{ role = "Engine", who = "Vectarine" },
	{ role = "Sprites", who = "Drawn for this game (tools/art), Waldgeist palette by Acid Burritos" },
	{ role = "Key and menu icons", who = "Kenney (kenney.nl)" },
	{ role = "Font", who = "m6x11 by Daniel Linssen, with Greek letters added" },
	{ role = "Game", who = "You! This is a template, make it yours." },
}

local lines: { ui.Widget<any> } = {}
for _, credit in CREDITS do
	table.insert(lines, ui.text(credit.role, { fontSize = 0.065, font = theme.font, color = theme.colors.gold }))
	table.insert(lines, ui.text(credit.who, { fontSize = 0.08, font = theme.font, color = theme.colors.text }))
	table.insert(lines, ui.spacer(vec.V2(0, 0.04)))
end

local menu = ui_components.menu({
	{
		label = "Back",
		icon = "icons/back",
		action = function(extra: any)
			extra.currentScreen = ""
		end,
	},
}, vec.V2(0.5, 0.12))

module.creditsScreenWidget = ui.stack({ alignX = "center", alignY = "center" }, {
	ui.widget(vec.V2(2, 2), function()
		menu_backdrop.draw()
	end),
	ui.column({ align = "center" }, {
		ui.text("CREDITS", { fontSize = 0.2, font = theme.font, color = theme.colors.gold }),
		ui.spacer(vec.V2(0, 0.08)),
		ui.column({ align = "center" }, lines),
		ui.spacer(vec.V2(0, 0.05)),
		menu,
		ui.spacer(vec.V2(0, 0.1)),
	}),
})

return module
