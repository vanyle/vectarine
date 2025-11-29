local Coord = require("@vectarine/coord")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--- Graphics
---
--- Functions to draw shapes, images, text, and canvases to the screen
local module = {}

type Pos = Coord.ScreenPosition | Vec.Vec2
type Direction = Coord.ScreenVec | Vec.Vec2

--- Clear the canvas
function module.clear(color: Vec4.Vec4): () end

--- Draws a filled rectangle
function module.drawRect(pos: Pos, size: Direction, color: Vec4.Vec4): () end

--- Draws a convex filled polygon
function module.drawPolygon(points: { Pos }, color: Vec4.Vec4): () end

--- Draws an arrow starting at `pos`, and towards `direction`
function module.drawArrow(pos: Pos, direction: Direction, color: Vec4.Vec4?, size: number?): () end

--- Draws a filled circle
function module.drawCircle(center: Pos, radius: number, color: Vec4.Vec4): () end

--- Draws an ellipse contained in the rectangle defined by `center` and `size * 2`
function module.drawEllipse(center: Pos, size: Direction, color: Vec4.Vec4): () end

--- Runs drawFunction using the result of the maskFunction as a mask.
--- drawFunction will only be able to draw in places where maskFunction drew.
--- The output of maskFunction is not visible directly and the colors used by the mask do not matter.
---
--- This can be useful to crop images or scenes to specific shapes like a circle.
function module.drawWithMask(drawFunction: () -> (), maskFunction: () -> ()): ()
	error("Not implemented")
end

return module
