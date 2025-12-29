local Coord = require("@vectarine/coord")
local ResourceTypes = require("@vectarine/resource")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local module = {}

local ImageResourceImpl = { image = true }
ImageResourceImpl.__index = ImageResourceImpl
export type ImageResource = typeof(setmetatable({}, ImageResourceImpl)) & ResourceTypes.Resource

type Pos = Coord.ScreenPosition | Vec.Vec2
type Direction = Coord.ScreenVec | Vec.Vec2

--- Draws the image at the position and with the size given
--- You can optionally tint the image with the given color. The color of the pixels of the image
--- are multiplied by the tint.
function ImageResourceImpl.draw(self: ImageResource, pos: Pos, size: Direction, color: Vec4.Vec4?): ()
	error("Not implemented")
end

--- Draws a rectangular part of the image delimited by a position and a size
--- This section is deformed to match the quadrilateral delimited by the 4 destination points
--- You can swap the destination points to rotate, flip the image.
--- Use `src_pos = V2(0, 0)` and `src_size = V2(1, 1)` to draw the full image.
function ImageResourceImpl.drawPart(
	self: ImageResource,
	dest_p1: Pos,
	dest_p2: Pos,
	dest_p3: Pos,
	dest_p4: Pos,
	src_pos: Vec.Vec2,
	src_size: Vec.Vec2,
	color: Vec4.Vec4?
): Vec.Vec2
	error("Not implemented")
end

--- Gets the size of the image in pixels
function ImageResourceImpl.getSize(self: ImageResource): Vec.Vec2
	error("Not implemented")
end

return module
