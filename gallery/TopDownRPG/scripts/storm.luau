--[[
Sandstorms. Some levels of the desert have storms (`storm` in data/levels.luau): calm for a while, then the sand
rises for a while, again and again. Some never stop (`always`).

During a storm, the sand hides everything beyond a few steps (the darkness shader draws it, see lighting.luau)
and the wind pushes the player. The Eye of the Storm (an artefact) calms the storm around the player.

The strength of the storm only depends on the play time, so it is the same in tests and after loading a save.
]]
const loader = require("@vectarine/loader")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- Seconds for a storm to rise, and to calm down
const FADE = 2.5

--- How strong the storm is right now, from 0 (calm) to 1
function module.intensity(game: types.Game): number
	local storm = game.level.storm
	if not storm then
		return 0
	end
	if storm.always then
		return 1
	end
	local calm, duration = storm.calm or 40, storm.duration or 15
	local t = game.time % (calm + duration) - calm
	if t < 0 then
		return 0
	end
	return math.clamp(math.min(t, duration - t) / FADE, 0, 1)
end

--- Does the player see through the storm? (they carry the Eye of the Storm)
function module.isCalmed(game: types.Game): boolean
	return game.player.artefacts.eye_of_storm == true
end

--- How the wind pushes the player right now, in tiles per second
function module.wind(game: types.Game): vec.Vec2
	local storm = game.level.storm
	if not storm or module.isCalmed(game) then
		return vec.V2(0, 0)
	end
	return storm.wind:scale(storm.strength * module.intensity(game))
end

--- How much the sand hides, from 0 to 1 (less with the Eye of the Storm)
function module.haze(game: types.Game): number
	local intensity = module.intensity(game)
	return if module.isCalmed(game) then intensity * 0.35 else intensity
end

return module
