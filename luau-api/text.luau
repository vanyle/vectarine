local Color = require("@vectarine/color")
local Coord = require("@vectarine/coord")
local Resource = require("@vectarine/resource")
local Vec = require("@vectarine/vec")
local module = {}

local FontResourceImpl = { type = "font" }
FontResourceImpl.__index = FontResourceImpl
export type FontResource = typeof(setmetatable({}, FontResourceImpl)) & Resource.Resource

type Pos = Coord.ScreenPosition | Vec.Vec2

--- Draws text at (x,y) with given font, size and color
--- size is the maximum height that a text of that font will take on the screen
--- This is sometimes called the line height.
function FontResourceImpl.drawText(self: FontResource, text: string, pos: Pos, size: number, color: Color.Vec4): () end

--- Measures how much space the text will take when drawn
--- height will always be less than size.
--- bearingY will always be less than height. This is the distance from the top of the text to the baseline.
function FontResourceImpl.measureText(
	self: FontResource,
	text: string,
	size: number
): { width: number, height: number, bearingY: number }
	error("Not implemented")
end

return module
