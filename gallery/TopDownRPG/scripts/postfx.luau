--[[
Post-processing: the whole frame is drawn into a canvas, which is then drawn on the screen through
a shader (shaders/postfx.glsl). The shader adds flashes, color splits, transitions, and colors the edges of the screen
when the player is in danger.

Usage (see main.luau):
	postfx.render(function()
		-- draw everything here
	end)

Other modules control the effects by changing the fields of `postfx.state`, or through effects.luau
for flashes. Screen shake moves the whole frame inside the canvas.
]]
const canvas = require("@vectarine/canvas")
const effects = require("./effects")
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const settings = require("./settings")
const theme = require("./theme")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

module.shader = loader.loadShader("shaders/postfx.glsl")

module.state = module.state
	or {
		--- 1 = fully open, 0 = the screen is black. Used for transitions.
		iris = 1,
		--- Center of the iris, in OpenGL coordinates
		irisCenter = vec.V2(0, 0),
		--- Color split strength, decays to 0
		aberration = 0,
		--- 0 = colors, 1 = black and white
		desaturate = 0,
		--- Strength of the red edges (low health) and the yellow edges (low water), from 0 to 1
		danger = 0,
		thirst = 0,
		cold = 0,
	}
-- The state kept from before a reload may come from an older version of this file
module.state.danger = module.state.danger or 0
module.state.thirst = module.state.thirst or 0
module.state.cold = module.state.cold or 0

--- Tints multiplied with the colors of the edges of the screen: red keeps the reds, yellow removes the blues
const DANGER_TINT = vec4.createColor(1.5, 0.35, 0.3, 1)
const THIRST_TINT = vec4.createColor(1.35, 1.1, 0.3, 1)
--- The color of the frost on the edges when the player is cold
const COLD_TINT = vec4.createColor(0.78, 0.86, 0.95, 1)

--- The canvas is recreated when the window size changes (creating a canvas is expensive).
local current: { canvas: canvas.Canvas, size: vec.Vec2 }? = nil

local function getCanvas(): canvas.Canvas
	local size = io.getWindowSize()
	if not current or current.size.x ~= size.x or current.size.y ~= size.y then
		current = { canvas = canvas.createCanvas(size.x, size.y), size = size }
	end
	return (current :: any).canvas
end

function module.update(dt: number)
	local state = module.state
	state.aberration = math.max(0, state.aberration - dt * 3)
end

--- Draw `draw` with the post-processing effects.
function module.render(draw: () -> ())
	if not module.shader:isReady() then
		draw()
		return
	end
	local target = getCanvas()
	local state = module.state
	local size = io.getWindowSize()

	target:paint(function()
		graphics.clear(vec4.BLACK)
		-- Screen shake: move everything by a small offset that changes every frame
		module.frame = (module.frame or 0) + 1
		local shake = if settings.values.screenShake then effects.shakeAmount else 0
		local offset = vec.V2(math.sin(module.frame * 1.9) * shake * 0.05, math.cos(module.frame * 2.7) * shake * 0.05)
		if shake > 0 then
			graphics.withTransformation({ translation = offset }, draw)
		else
			draw()
		end
	end)

	target:setShader(module.shader)
	target:setUniform("aberration", state.aberration + effects.flashAmount * 0.5)
	local flash = effects.flashColor
	target:setUniform("flash", vec4.createColor(flash.r, flash.g, flash.b, effects.flashAmount * 0.6))
	target:setUniform("desaturate", state.desaturate)
	target:setUniform("danger", theme.withAlpha(DANGER_TINT, state.danger * 0.9))
	target:setUniform("thirst", theme.withAlpha(THIRST_TINT, state.thirst * 0.8))
	target:setUniform("cold", theme.withAlpha(COLD_TINT, state.cold * 0.75))
	target:setUniform("iris", state.iris)
	-- The iris center is in OpenGL coordinates (-1 to 1), the shader works with uv coordinates (0 to 1)
	target:setUniform("irisPos", state.irisCenter:scale(0.5) + vec.V2(0.5, 0.5))
	target:setUniform("resolution", size)
	target:draw(vec.V2(-1, -1), vec.V2(2, 2))
end

return module
