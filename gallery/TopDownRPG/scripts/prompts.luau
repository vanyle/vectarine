--[[
Keyboard prompts: the hints in the bar below the map telling the player which keys do what.

`forContext` decides which prompts to show depending on what the player is doing.
]]
const coord = require("@vectarine/coord")
const keyboard_icons = require("./keyboard_icons")
const loader = require("@vectarine/loader")
const theme = require("./theme")
const ui = require("@vectarine/ui")

const module = loader.init() or {}

export type Prompt = {
	--- A tile type of keyboard-input-icons.tsx
	key: string,
	label: string,
}

--- What the player is doing
export type Context = {
	--- "explore" in the overworld, "level" in a puzzle, "dead" after dying, "ending" at the end
	mode: "explore" | "level" | "dead" | "ending",
	--- What [Space] does right now, if anything ("Talk", "Mine", "Wait", ...)
	actionLabel: string?,
}

--- The size of each key icon, in pixels
const KEY_SIZES: { [string]: coord.ScreenVec } = {
	arrows = coord.pxVec(30, 30),
	space = coord.pxVec(34, 34),
}
const DEFAULT_KEY_SIZE = coord.pxVec(22, 22)

function module.forContext(context: Context): { Prompt }
	if context.mode == "ending" then
		return { { key = "space", label = "Title screen" } }
	elseif context.mode == "dead" then
		return { { key = "z", label = "Undo" }, { key = "r", label = "Restart" } }
	end
	local result: { Prompt } = { { key = "arrows", label = "Move" } }
	if context.actionLabel then
		table.insert(result, { key = "space", label = context.actionLabel })
	end
	if context.mode == "level" then
		table.insert(result, { key = "z", label = "Undo" })
		table.insert(result, { key = "r", label = "Restart" })
		table.insert(result, { key = "esc", label = "Leave" })
	end
	return result
end

--- A widget showing the prompts in a row.
function module.widget(list: { Prompt }): ui.Widget<any>
	local entries: { ui.Widget<any> } = {}
	for _, prompt in list do
		table.insert(
			entries,
			ui.row({ align = "center", gap = 0.012 }, {
				keyboard_icons.widgetForKey(prompt.key, KEY_SIZES[prompt.key] or DEFAULT_KEY_SIZE),
				ui.text(prompt.label, { fontSize = 0.055, font = theme.font, color = theme.colors.text }),
			})
		)
	end
	return ui.row({ align = "center", gap = 0.05 }, entries)
end

return module
