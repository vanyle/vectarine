--[[
Colors and fonts shared by every screen.

The palette is inspired by NES-era RPGs: dark backgrounds, warm gold highlights and white text.
Change the values here to reskin the whole game.
]]
const loader = require("@vectarine/loader")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- m6x11 is a pixel font by Daniel Linssen (managore.itch.io).
module.font = loader.loadFont("fonts/m6x11.ttf")

module.colors = {
	background = vec4.createColor(0.06, 0.06, 0.08, 1),
	panel = vec4.createColor(0.05, 0.05, 0.07, 0.92),
	panelBorder = vec4.createColor(0.95, 0.93, 0.88, 1),
	gold = vec4.createColor(0.98, 0.66, 0.24, 1),
	goldDark = vec4.createColor(0.55, 0.32, 0.1, 1),
	text = vec4.createColor(0.97, 0.96, 0.92, 1),
	textDim = vec4.createColor(0.6, 0.6, 0.62, 1),
	shadow = vec4.createColor(0.0, 0.0, 0.0, 0.75),
	danger = vec4.createColor(1, 0.35, 0.3, 1),
}

--- Returns `color` with its alpha multiplied by `alpha`. Useful for fading things in and out.
function module.withAlpha(color: vec4.Vec4, alpha: number): vec4.Vec4
	return vec4.createColor(color.r, color.g, color.b, color.a * alpha)
end

return module
