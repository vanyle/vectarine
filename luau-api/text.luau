local Coord = require("@vectarine/coord")
local Resource = require("@vectarine/resource")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local module = {}

local FontResourceImpl = { type = "font" }
FontResourceImpl.__index = FontResourceImpl
export type FontResource = typeof(setmetatable({}, FontResourceImpl)) & Resource.Resource

type Pos = Coord.ScreenPosition | Vec.Vec2
type Size = number | Coord.ScreenVec

--- Draws text at (x,y) with given font, size and color
--- size is the maximum height that a text of that font will take on the screen
--- If size is a number, it is in OpenGL coordinate (2 = height of the screen)
--- If size is a ScreenVec, only the y component is used
--- This is sometimes called the line height.
function FontResourceImpl.drawText(self: FontResource, text: string, pos: Pos, size: Size, color: Vec4.Vec4?): () end

--- Measures how much space the text will take when drawn
--- height will always be less than size.
--- bearingY will always be less than height. This is the distance from the top of the text to the baseline.
function FontResourceImpl.measureText(
	self: FontResource,
	text: string,
	size: Size
): { width: number, height: number, bearingY: number }
	error("Not implemented")
end

local function getDefaultFontPrivate(): FontResource
	error("Not implemented")
end

--- The default font (Roboto) you can use to draw without needed to load a `.ttf` file
--- It is not a true resource and is always loaded.
module.font = getDefaultFontPrivate()

return module
