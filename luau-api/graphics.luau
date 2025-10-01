local Canvas = require('@vectarine/canvas')
local Coords = require('@vectarine/coords')
local Res = require('@vectarine/resources')
local Vec = require('@vectarine/vec')

local module = {}

type Pos = Coords.ScreenPosition | Vec.Vec2
type Direction = Coords.ScreenDelta | Vec.Vec2

export type Color = { r: number; g: number; b: number; a: number }

--- Clear the canvas
function module.clear(color: Color): () end

--- Draws a filled rectangle
function module.drawRect(pos: Pos, size: Direction, color: Color): () end

--- Draws a convex filled polygon
function module.drawPolygon(points: {Pos}, color: Color): () end

--- Draws an arrow starting at `pos`, and towards `direction`
function module.drawArrow(pos: Pos, direction: Direction, color: Color?, size: number?): () end

--- Draws a filled circle
function module.drawCircle(center: Pos, radius: number, color: Color): () end

--- Draws an image
function module.drawImage(image: Res.ImageResource, pos: Pos, size: Direction): () end

--- Draws a rectangular part of an image delimited by a position and a size
--- This section is deformed to match the quadrilateral delimited by the 4 destination points
--- You can swap the destination points to rotate, flip the image.
--- Use `src_pos = V2(0, 0)` and `src_size = V2(1, 1)` to draw the full image.
function module.drawImagePart(image: Res.ImageResource, dest_p1: Pos, dest_p2: Pos, dest_p3: Pos, dest_p4: Pos, src_pos: Vec.Vec2, src_size: Vec.Vec2): () end

--- Draws text at (x,y) with given font, size and color
--- size is the maximum height that a text of that font will take on the screen
--- This is sometimes called the line height.
function module.drawText(text: string, font: Res.FontResource, pos: Pos, size: number, color: Color): () end

--- Measures how much space the text will take when drawn
--- height will always be less than size.
--- bearingY will always be less than height. This is the distance from the top of the text to the baseline.
function module.measureText(text: string, font: Res.FontResource, size: number): {width: number, height: number, bearingY: number}
    error("Not implemented")
end

--- Draws a canvas at given position and size
function module.drawCanvas(canvas: Canvas.Canvas, pos: Vec.Vec2, size: Direction): ()
    error("Not implemented")
end

--- Draws a part canvas stretched to fit the quadrilateral delimited by the 4 destination points
--- This is the same as `drawImagePart` but for canvases
function module.drawCanvasPart(canvas: Canvas.Canvas, dest_p1: Pos, dest_p2: Pos, dest_p3: Pos, dest_p4: Pos, src_pos: Vec.Vec2, src_size: Vec.Vec2): ()
    error("Not implemented")
end

return module
