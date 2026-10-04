--[[
Colors and fonts shared by every screen.

All colors come from the Waldgeist palette (https://lospec.com/palette-list/waldgeist), like the sprites.
Change the values here to reskin the whole interface.
]]
const loader = require("@vectarine/loader")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- m6x11 is a pixel font by Daniel Linssen (managore.itch.io), with Greek capitals added by tools/art/greek_font.py
--- for the writing of the sea people.
module.font = loader.loadFont("fonts/m6x11-greek.ttf")

local function hex(value: string, alpha: number?): vec4.Vec4
	local r = tonumber(string.sub(value, 1, 2), 16) or 0
	local g = tonumber(string.sub(value, 3, 4), 16) or 0
	local b = tonumber(string.sub(value, 5, 6), 16) or 0
	return vec4.createColor(r / 255, g / 255, b / 255, alpha or 1)
end
module.hex = hex

module.colors = {
	background = hex("0b1016"),
	panel = hex("141f25", 0.94),
	panelLight = hex("1f2d36", 0.96),
	panelBorder = hex("bdbbae"),
	gold = hex("a6884a"),
	goldDark = hex("46332c"),
	text = hex("e3ddd1"),
	textDim = hex("798a92"),
	shadow = hex("0b1016", 0.75),
	danger = hex("83473d"),
	health = hex("83473d"),
	healthDark = hex("562923"),
	sanity = hex("4e7499"),
	sanityDark = hex("245273"),
	water = hex("5e7e8b"),
	waterDark = hex("323b42"),
	xp = hex("73804b"),
	xpDark = hex("32534a"),
	good = hex("73804b"),
	-- The darkness of the caves
	darkness = hex("0b1016"),
}

--- Returns `color` with its alpha multiplied by `alpha`. Useful for fading things in and out.
function module.withAlpha(color: vec4.Vec4, alpha: number): vec4.Vec4
	return vec4.createColor(color.r, color.g, color.b, color.a * alpha)
end

return module
