--[[
The first steps: short narrations (like the opening of the game) that teach the basics the first time they matter.

- The first weapon picked up (the rusty knife of the sinkhole): how to equip it from the bag of the notebook.
- The first creature met (the cave rats of the sinkhole): how to fight, health, healing, fainting.

Each one is told once: a flag of the game remembers it (saved with the game).
]]
const dream = require("./dream")
const items = require("./data/items")
const level = require("./level")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const prompts = require("./prompts")
const types = require("./types")

const module = loader.init() or {}

--- How close the first creature must be for the combat narration
const CREATURE_DISTANCE = 4.5

local function narrate(game: types.Game, steps: { types.DialogStep })
	game.events[#game.events + 1] = { kind = "dialog", speaker = nil, steps = steps }
end

--- An item went into the bag (picked up, or found in a chest)
function module.onPickup(game: types.Game, itemId: string)
	local item = items.items[itemId]
	if item.kind ~= "weapon" or item.bomb or game.flags.tutorialWeapon then
		return
	end
	game.flags.tutorialWeapon = true
	local notebook = prompts.keyName(prompts.NOTEBOOK_KEY)
	narrate(game, {
		`A {string.lower(item.name)}. Better than bare fists.`,
		`Open your notebook with [{notebook}]: your bag is on its first page.`,
		"Select the weapon with the arrows, and press [Space] to equip it. Then [Space] strikes in front of you.",
	})
end

--- The first creature that comes close: how to fight, and how to stay alive. Called every frame.
function module.checkCreatures(game: types.Game)
	if game.flags.tutorialCombat then
		return
	end
	local player = game.player
	for _, monster in game.monsters do
		local species = monsters.species[monster.kind]
		local close = (monster.pos - player.pos):length() < CREATURE_DISTANCE
		if close and not monster.dead and not species.hazard and not species.prize and dream.isPresent(game, monster)
			and level.lineIsClear(game.level :: any, monster.pos, player.pos) then
			game.flags.tutorialCombat = true
			local notebook = prompts.keyName(prompts.NOTEBOOK_KEY)
			narrate(game, {
				`Something moves in the dark... a {string.lower(species.name)}. And it has seen you.`,
				"Face it and press [Space] to strike. Creatures warn before they attack: a rat crouches, then pounces. Step aside, then hit back.",
				"Your health is the red bar at the top. It comes back slowly when nothing hurts you, and all at once when you rest by a campfire.",
				`Figs and bandages heal you: use them from the bag of your notebook [{notebook}].`,
				"If you faint, you wake up by the last fire you rested at, with everything you carry.",
			})
			return
		end
	end
end

return module
