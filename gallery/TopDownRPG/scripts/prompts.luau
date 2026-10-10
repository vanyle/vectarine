--[[
Keyboard prompts: the hints in the bar below the world telling the player which keys do what.
They change with what the player can do right now (the [E] prompt only shows when there is something to use).
]]
const coord = require("@vectarine/coord")
const io = require("@vectarine/io")
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

--- The key that opens the notebook. Keys are read by their position on the keyboard: "Q" is the key next to Tab,
--- printed A on AZERTY keyboards. `keyName` gives what is printed on the player's keyboard.
module.NOTEBOOK_KEY = "Q"
--- The key that turns the pages of the notebook (at the same place on QWERTY and AZERTY keyboards)
module.PAGE_KEY = "E"

--- What is printed on the key at the position of `scancode` on the player's keyboard ("A" for "Q" on AZERTY)
function module.keyName(scancode: string): string
	return io.getKeyName(scancode) or scancode
end

--- The icon of a key (a tile type of keyboard-input-icons.tsx), following the layout of the keyboard
function module.keyIcon(scancode: string): string
	return string.lower(module.keyName(scancode))
end

--- The size of each key icon, in pixels
const KEY_SIZES: { [string]: coord.ScreenVec } = {
	arrows = coord.pxVec(30, 30),
	space = coord.pxVec(34, 34),
}
const DEFAULT_KEY_SIZE = coord.pxVec(22, 22)

--- The prompts while exploring. `interact` is the label of what [E] does, if anything.
function module.forPlay(attack: string, interact: string?): { Prompt }
	local list: { Prompt } = {
		{ key = "arrows", label = "Move" },
		{ key = "space", label = attack },
	}
	if interact then
		table.insert(list, { key = "e", label = interact })
	end
	table.insert(list, { key = module.keyIcon(module.NOTEBOOK_KEY), label = "Notebook" })
	table.insert(list, { key = "esc", label = "Pause" })
	return list
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
	return ui.row({ align = "center", gap = 0.045 }, entries)
end

return module
