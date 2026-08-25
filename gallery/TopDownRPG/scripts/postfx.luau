--[[
Post-processing: the whole frame is drawn into a canvas, which is then drawn on the screen through
a shader (shaders/postfx.glsl). The shader adds flashes, color splits and transitions.

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
	}

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
	target:setUniform("iris", state.iris)
	-- The iris center is in OpenGL coordinates (-1 to 1), the shader works with uv coordinates (0 to 1)
	target:setUniform("irisPos", state.irisCenter:scale(0.5) + vec.V2(0.5, 0.5))
	target:setUniform("resolution", size)
	target:draw(vec.V2(-1, -1), vec.V2(2, 2))
end

return module
