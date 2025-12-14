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

--- Draws a filled rectangle at the position `pos` with the size `size`
--- If color is not provided, it defaults to black
--- ```lua
--- Graphics.drawRect(Vec.V2(-0.05, -0.05), Vec.V2(0.1, 0.1), Vec4.RED)
--- ```
function module.drawRect(pos: Pos, size: Direction, color: Vec4.Vec4?): () end

--- Draws a convex filled polygon
function module.drawPolygon(points: { Pos }, color: Vec4.Vec4?): () end

--- Draws a line from `p1` to `p2`
--- If color is not provided, it defaults to black
--- If thickness is not provided, it defaults to 0.005
--- ```lua
--- -- Draw a line from the center of the screen to the top right
--- Graphics.drawLine(Coord.CENTER, Coord.TOP_RIGHT)
--- ```
function module.drawLine(p1: Pos, p2: Pos, color: Vec4.Vec4?, thickness: number?): () end

--- Draws an arrow starting at `pos`, and towards `direction`
--- You can use it to show the speed of an entity for example
function module.drawArrow(pos: Pos, direction: Direction, color: Vec4.Vec4?, size: number?): () end

--- Draws a filled circle
function module.drawCircle(center: Pos, radius: number, color: Vec4.Vec4?): () end

--- Draws an ellipse contained in the rectangle defined by `center` and `size * 2`
function module.drawEllipse(center: Pos, size: Direction, color: Vec4.Vec4?): () end

--- Runs drawFunction using the result of the maskFunction as a mask.
--- drawFunction will only be able to draw in places where maskFunction drew.
--- The output of maskFunction is not visible directly and the colors used by the mask do not matter.
---
--- This can be useful to crop images or scenes to specific shapes like a circle.
function module.drawWithMask(drawFunction: () -> (), maskFunction: () -> ()): ()
	error("Not implemented")
end

return module
