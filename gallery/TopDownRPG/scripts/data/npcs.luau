--[[
Characters you can talk to with [Space].

`talk` is called every time the player talks to the character and returns the dialog to show.
It can look at the game state to say different things depending on the story progress.
`visible` (optional) decides if the character is currently in the world.

A dialog is a list of steps (see types.DialogStep):
- a string is a line said by the character
- a function runs when the dialog reaches it. If it returns a string, it is shown as a narration line.
]]
const loader = require("@vectarine/loader")
const types = require("../types")

export type Npc = {
	name: string,
	sprite: string,
	talk: (game: types.Game) -> { types.DialogStep },
	visible: ((game: types.Game) -> boolean)?,
}

const module = loader.init() or {}

module.npcs = {
	--- The fairy of the glade, who welcomes the player. She flies to the mine after the introduction.
	fairy = {
		name = "Morgane",
		sprite = "dungeon/fairy",
		visible = function(game: types.Game): boolean
			return not game.flags.metFairy
		end,
		talk = function(_game: types.Game): { types.DialogStep }
			return {
				"Oh! You can move. Good. Try not to make a fuss about it.",
				"Look up. The moon hasn't budged since midnight. The owls are stuck mid-hoot.",
				"The clock of Plouneuc stopped, and the whole valley stopped with it.",
				"Well... almost. You are a Walker. Every step you take nudges the world forward by one tick.",
				"Nothing moves unless you do. Remember that. Down there, it matters.",
				"The gears of the clock fell into the old mine, east of the village. Would you bring them back?",
				"I'll fly ahead. Meet me by the ladders.",
				function(game: types.Game): string?
					game.flags.metFairy = true
					return nil
				end,
			}
		end,
	},

	--- The same fairy, waiting in the mine
	fairy_mine = {
		name = "Morgane",
		sprite = "dungeon/fairy",
		visible = function(game: types.Game): boolean
			return game.flags.metFairy == true
		end,
		talk = function(game: types.Game): { types.DialogStep }
			local finished = game:finishedCount()
			if finished == 0 then
				return {
					"Each ladder leads down to an old gallery. Each gallery hides a gear.",
					"The creatures down there are dangerous. Watch them carefully.",
					"If you get stuck, [Z] takes back a step and [R] starts over. I won't tell anyone.",
				}
			end
			return {
				`{finished} gears so far. I can almost hear the tower breathing.`,
			}
		end,
	},

	yann = {
		name = "Yann",
		sprite = "dungeon/viking",
		talk = function(game: types.Game): { types.DialogStep }
			if not game.inventory.hasPickaxe then
				return {
					"I'm Yann, the clockmaker. Or I was, until my clock decided to retire.",
					"Bring the gears back up and I'll set them in place, one by one.",
				}
			end
			return {
				"The pickaxe cracks boulders. Not walls, mind you.",
				"Walls need something with more... enthusiasm.",
			}
		end,
	},

	gwen = {
		name = "Mamm Gwen",
		sprite = "dungeon/old_woman",
		talk = function(_game: types.Game): { types.DialogStep }
			return {
				"Midnight again. It's been midnight for a very long while, dearie.",
				"When I was a girl, fairies lit up the meadow every night. They always left before dawn.",
			}
		end,
	},

	lomig = {
		name = "Lomig",
		sprite = "farm/farmer",
		talk = function(_game: types.Game): { types.DialogStep }
			return {
				"My kite is stuck in the sky! When you walk around, it wiggles a tiny bit.",
				"Walk more! Walk faster! No wait, that doesn't help at all.",
			}
		end,
	},
} :: { [string]: Npc }

return module
