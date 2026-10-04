--[[
Hints about what blocks the way: they stay on the screen while the player stands next to it, so that there is time
to read them, and they come back when the player comes back.

- vines, without the climbing claws
- an icefall, without the rime crampons
- miasma, without the beaked mask
- a veil of frozen mist, for a player who is awake
- slush, for a player who carries the ferry (it can't sail through it)
]]
const dream = require("./dream")
const level = require("./level")
const loader = require("@vectarine/loader")
const types = require("./types")

const module = loader.init() or {}

--- The hint about the cells around the player, if something there blocks the way (nil otherwise)
function module.current(game: types.Game): string?
	local player = game.player
	local lvl = game.level :: level.Level
	local center = level.cellOf(player.pos)
	-- What is up to two cells away: the player stops a bit before a wall of mist, or a cliff
	for dy = -2, 2 do
		for dx = -2, 2 do
			local x, y = center.x + dx, center.y + dy
			local wall = level.wallAt(lvl, x, y)
			local ground = level.groundAt(lvl, x, y)
			if wall and wall.climbable and not player.artefacts.climbing_claws then
				return "The vines are too thin to climb with bare hands. With claws, maybe..."
			elseif wall and wall.icefall and not player.artefacts.rime_crampons then
				return "A frozen waterfall, smooth as glass. Your boots slide right off it."
			elseif ground and ground.miasma and not player.artefacts.beaked_mask then
				return "The air is thick and green in there. You can't breathe it."
			elseif ground and ground.freezing and not ground.sailable and player.artefacts.ferry_skiff
				and math.abs(dx) <= 1 and math.abs(dy) <= 1 then
				return "Thick slush, full of broken ice: the ferry can't cut through it. A block of ice would fill it."
			elseif ground and ground.veil and not dream.isDreaming(game) then
				return "A wall of frozen mist. It feels solid... but something in it looks like a dream."
			end
		end
	end
	return nil
end

return module
