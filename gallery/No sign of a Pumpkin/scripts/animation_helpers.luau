local Debug = require("@vectarine/debug")
local Vec = require("@vectarine/vec")

local LevelDesign = require("level_design.luau")

local V2 = Vec.V2

function getDeformedPlayerSize(player: LevelDesign.Player, timer: number, correctForTimerOffset: boolean): Vec.Vec2
	local sizeDeform = correctForTimerOffset and 0.05 * math.sin(5 * timer) or 0.0
	return V2(player.size + sizeDeform, player.size - sizeDeform)
end

function animate(animatable, dt, opt_frame_delay)
	-- Strict delta-time based animation. `dt` is required (seconds since last Update).
	assert(dt ~= nil, "animate requires dt (seconds)")

	-- prefer explicit override, then per-animatable frame_delay; if missing, warn and default to 0.1s
	local delay = opt_frame_delay or animatable.frame_delay
	if delay == nil then
		Debug.fprint("animate: animatable.frame_delay missing; defaulting to 0.1s. Set frame_delay for exact control.")
		delay = 0.1
		animatable.frame_delay = delay
	end

	local delay_num = delay or 0
	local images = animatable.animation_images or {}
	local frames = #images
	if frames == 0 then
		-- no images: nothing to animate, return frame 1 as a safe default
		return 1
	end

	-- advance accumulated animation time and wrap
	animatable.anim_time = (animatable.anim_time or 0) + dt
	local at = animatable.anim_time or 0
	local total = delay_num * frames
	if total > 0 then
		at = at % total
	else
		at = 0
	end
	animatable.anim_time = at

	local frameIndex = math.floor(at / delay_num) + 1
	if frameIndex < 1 then
		frameIndex = 1
	elseif frameIndex > frames then
		frameIndex = frames
	end
	return frameIndex
end

-- Export the module functions
return {
	getDeformedPlayerSize = getDeformedPlayerSize,
	animate = animate,
}
