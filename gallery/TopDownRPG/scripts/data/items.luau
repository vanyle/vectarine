--[[
Items the player can use with [Space]. There is one key for everything:
facing a boulder, [Space] uses the pickaxe, otherwise the last item picked up (see game_state.itemForSpace).

- The pickaxe is a tool: once you have it, you keep it.
- Arrows and bombs are ammunition found inside each level. They are lost when you leave the level.

`action` is the name of the function in rules.luau (`itemActions`) that runs when the item is used.
Using an item always takes one turn, even if nothing happens, so it can also be used to wait.
]]
const loader = require("@vectarine/loader")

export type Item = {
	name: string,
	sprite: string,
	--- Shown next to the [Space] prompt
	verb: string,
	action: "mine" | "shoot" | "bomb",
	--- The ammunition used by the item, if any
	ammo: ("arrows" | "bombs")?,
}

const module = loader.init() or {}

--- The order of the items when the selected one runs out
module.order = { "pickaxe", "bow", "bomb" }

module.items = {
	pickaxe = {
		name = "Pickaxe",
		sprite = "town/pickaxe",
		verb = "Mine",
		action = "mine",
	},
	bow = {
		name = "Bow",
		sprite = "town/bow",
		verb = "Shoot",
		action = "shoot",
		ammo = "arrows",
	},
	bomb = {
		name = "Bomb",
		sprite = "town/bomb",
		verb = "Drop bomb",
		action = "bomb",
		ammo = "bombs",
	},
} :: { [string]: Item }

return module
