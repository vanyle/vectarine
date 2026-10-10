--[[
The snowball gallery of the Frostborn kids, in the yard of Rimehold (see Sleet in npcs.luau).

Talking to Sleet starts a round: for ROUND_TIME seconds, attacking throws snowballs (whatever the weapon), and targets
come and go in the yard, where the level has `gallery_spot` props (`data.kind` says which target, `data.dir` where it
goes):
- snowmen pop up from behind the snow walls, and sink back a moment later (1 point)
- pucks slide back and forth on the rink, bouncing on its edges (2 points)
- the frost hare dashes across the yard from time to time (5 points)
- phantom snowmen only show up for a dreaming player (4 points): the old champion, Sorrel, "saw targets nobody else did"

Beat the kids' score (GOAL) to win the rime crampons, and the champion's (CHAMPION) for Sorrel's snowflake.
The targets are creatures with `prize` (data/monsters.luau): they don't hurt, give no experience and are not in the
bestiary. The round is not saved: leaving the level ends it.
]]
const effects = require("./effects")
const inventory = require("./inventory")
const items = require("./data/items")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

module.ROUND_TIME = 40
--- The score of the kids, and the score of Sorrel, the old champion. A bot that always walks under the best target
--- scores about 26 awake, and over 100 dreaming (the phantom snowmen): the kids' score is for a good player, Sorrel's
--- can't be reached without the dream.
module.GOAL = 13
module.CHAMPION = 40

--- The target creature of each kind of spot
const TARGETS: { [string]: string } = {
	snowman = "snow_target",
	puck = "ice_puck",
	hare = "frost_hare",
	phantom = "phantom_target",
}

module.state = module.state or {
	active = false,
	time = 0,
	score = 0,
	levelId = "",
}

local function message(game: types.Game, text: string)
	game.events[#game.events + 1] = { kind = "message", text = text }
end

--- Start a round in the yard of the current level
function module.start(game: types.Game)
	local state = module.state
	state.active = true
	state.time = module.ROUND_TIME
	state.score = 0
	state.levelId = game.level.id
	for _, prop in game.level.props do
		local kind = TARGETS[prop.data.kind or ""]
		if prop.kind == "gallery_spot" and kind then
			local dir = prop.data.dir or { 1, 0 }
			local pos = prop.cell + vec.V2(0.5, 0.6)
			table.insert(game.monsters, {
				id = game.nextMonsterId,
				kind = kind,
				pos = pos,
				height = 0,
				spawn = pos,
				hp = monsters.species[kind].hp,
				state = "hidden",
				-- Targets don't all come up at once
				stateTime = -math.random() * 2,
				facing = (if dir[1] < 0 then "left" else "right") :: types.Direction,
				target = vec.V2(dir[1], dir[2]),
				cooldown = 0,
				hitTime = 10,
				knockback = vec.V2(0, 0),
				phase = 1,
				dead = false,
			})
			game.nextMonsterId += 1
		end
	end
end

--- A snowball hit a target
function module.hit(game: types.Game, monster: types.Monster)
	local species = monsters.species[monster.kind]
	local points = species.prize or 1
	module.state.score += points
	effects.floatText(monster.pos + vec.V2(0, -1.2), `+{points}`, if points >= 3 then theme.colors.gold else theme.colors.text)
	effects.burst(monster.pos + vec.V2(0, -0.4), "snow", 10)
	monster.state = "down"
	monster.stateTime = 0
end

local function give(game: types.Game, itemId: string)
	inventory.give(game.player, itemId, 1)
	if items.items[itemId].kind == "artefact" then
		game.events[#game.events + 1] = { kind = "artefact", item = itemId }
	end
end

--- End the round: the targets go away, and Sleet gives the prizes
local function finish(game: types.Game)
	local state = module.state
	state.active = false
	for _, monster in game.monsters do
		if monsters.species[monster.kind].prize then
			monster.dead = true
		end
	end
	local score = state.score
	local flags = game.flags
	flags.galleryBest = math.max(flags.galleryBest or 0, score)
	local steps: { types.DialogStep } = { `Time! You hit {score} points.` }
	if score >= module.CHAMPION and not flags.galleryChampion then
		flags.galleryChampion = true
		table.insert(steps, "...No way. That's more than Sorrel ever did! You saw them too, didn't you? The pale ones.")
		table.insert(steps, "Sorrel left this for whoever beat the record. It's yours, champion.")
		table.insert(steps, function(g: types.Game): string?
			give(g, "sorrels_snowflake")
			return "You got: Sorrel's snowflake!"
		end)
	end
	if score >= module.GOAL and not flags.galleryWon then
		flags.galleryWon = true
		table.insert(steps, `{score}! You beat us fair and square. A deal's a deal: the champion's crampons are yours.`)
		table.insert(steps, "With those, the icefalls are stairs. The Old Watch is up there, beyond the Glare. Don't go and get frozen, two-legs.")
		table.insert(steps, function(g: types.Game): string?
			give(g, "rime_crampons")
			return "You got: Rime crampons!"
		end)
	elseif score < module.GOAL then
		table.insert(steps, `Ha! We got {module.GOAL} last time. Talk to me when you want another go.`)
	elseif not flags.galleryChampion then
		table.insert(steps, `Not bad! But Sorrel, the old champion, got {module.CHAMPION}. Nobody knows how. She said she saw targets nobody else could.`)
	end
	game.events[#game.events + 1] = { kind = "dialog", speaker = { name = "Sleet", sprite = "characters/sleet_0" }, steps = steps }
end

--- Called every frame
function module.update(game: types.Game, dt: number)
	local state = module.state
	if not state.active then
		return
	end
	if game.level.id ~= state.levelId or game.player.hp <= 0 then
		state.active = false
		message(game, "You left the yard: the round is over.")
		return
	end
	state.time -= dt
	if state.time <= 0 then
		finish(game)
	end
end

function module.isActive(): boolean
	return module.state.active
end

return module
