--[[
Story beats: short dialogs shown when coming back from a level for the first time.

`afterLevel[n]` returns who speaks and what they say after the level n is finished.
Levels without an entry have no story beat.
]]
const loader = require("@vectarine/loader")
const types = require("../types")

export type Beat = (game: types.Game) -> (types.Speaker, { types.DialogStep })

const module = loader.init() or {}

const MORGANE: types.Speaker = { name = "Morgane", sprite = "dungeon/fairy" }
const YANN: types.Speaker = { name = "Yann", sprite = "dungeon/viking" }

module.afterLevel = {
	[1] = function(_game: types.Game)
		return YANN,
			{
				"A gear! Look at her shine. Barely a scratch.",
				"The lower galleries are choked with boulders. Take my old pickaxe.",
				function(game: types.Game): string?
					game.inventory.hasPickaxe = true
					game.inventory.selected = "pickaxe"
					return "You got the Pickaxe! Face a boulder and press [Space] to break it."
				end,
			}
	end,
	[4] = function(_game: types.Game)
		return MORGANE,
			{
				"Did you hear that? The tower went 'tick'. Just once.",
				"Plouneuc is waking up, one gear at a time.",
			}
	end,
	[8] = function(_game: types.Game)
		return MORGANE,
			{
				"Say, Walker... Did you ever wonder why fairies only come out at night?",
				"We fade with the first light of morning. That's the deal.",
				"Never mind me. Keep going.",
			}
	end,
	[11] = function(_game: types.Game)
		return MORGANE,
			{
				"What if the clock stayed stopped? No mornings. No goodbyes.",
				"Would that really be so bad?",
			}
	end,
	[14] = function(_game: types.Game)
		return MORGANE,
			{
				"I have to tell you something.",
				"I stopped the clock. I hid the gears in the mine.",
				"I wanted one more night with Plouneuc. Then one more. Then another one...",
				"The last gear is at the very bottom. I asked you to find them because part of me wanted you to.",
			}
	end,
	[15] = function(_game: types.Game)
		return MORGANE,
			{
				"That's the last one.",
				"Listen. Tick. Tock. The owls are finishing their hoot.",
				"A night that never ends isn't a night. It's a cage. I see that now.",
				"Thank you, Walker. Look... the sky is turning pink.",
				function(game: types.Game): string?
					game.status = "ending"
					return nil
				end,
			}
	end,
} :: { [number]: Beat }

return module
