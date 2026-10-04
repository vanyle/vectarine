--[[
The dream: on the frontier, a tired mind sees what the Watch left behind.

When the player's sanity is low (DREAM_LEVEL of its maximum, or always with the sleepless eye), they dream awake:
- veils (walls of frozen mist, the ground "veil") let them through,
- dream bridges carry them over chasms, and dream writing can be read,
- phantoms (creatures with `dream` in data/monsters.luau) are there: they can hurt, and be hurt.
Awake, phantoms are not there at all: they don't move, can't be seen, can't hurt.

Sanity is lowered on purpose: gazing in a dreaming mirror, eating a hollowcap, using sanity weapons. It comes back
in the light (lighting.luau), at campfires and with herb tea. Nothing drains it by itself.
]]
const effects = require("./effects")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const player_rules = require("./player")
const theme = require("./theme")
const types = require("./types")

const module = loader.init() or {}

--- The player dreams below this fraction of their maximum sanity
module.DREAM_LEVEL = 0.35
--- Where a dreaming mirror (or a hollowcap) leaves the player's sanity, as a fraction of the maximum
module.DEEP_DREAM = 0.15

--- Was the player dreaming last frame (to tell them when it changes)
module.wasDreaming = module.wasDreaming or false

function module.isDreaming(game: types.Game): boolean
	local player = game.player
	return player.artefacts.sleepless_eye == true or player.sanity <= player_rules.maxSanity(player) * module.DREAM_LEVEL
end

--- Is this creature there right now? Phantoms only exist for dreamers.
function module.isPresent(game: types.Game, monster: types.Monster): boolean
	return not monsters.species[monster.kind].dream or module.isDreaming(game)
end

--- Sink into the dream: sanity drops to the deep dream (a mirror, a hollowcap)
function module.sink(game: types.Game)
	local player = game.player
	player.sanity = math.min(player.sanity, player_rules.maxSanity(player) * module.DEEP_DREAM)
	effects.flash(theme.colors.sanity, 0.5)
end

local function message(game: types.Game, text: string)
	game.events[#game.events + 1] = { kind = "message", text = text }
end

--- Called every frame: tells the player when they start and stop dreaming
function module.update(game: types.Game)
	local dreaming = module.isDreaming(game)
	if dreaming ~= module.wasDreaming then
		module.wasDreaming = dreaming
		effects.flash(theme.colors.sanity, 0.3)
		-- Only on the frontier, where it matters (and only the first times: the player learns it)
		if game.level.depth >= 4 then
			local seen = (game.flags.dreamHints or 0) :: number
			if seen < 2 then
				game.flags.dreamHints = seen + 1
				message(game, if dreaming
					then "Your mind drifts... The mist of the veils thins, and pale shapes walk in it. You dream awake."
					else "You come back to your senses. The veils are walls of mist again.")
			end
		end
	end
end

return module
