local Coord = require("@vectarine/coord")
local Resource = require("@vectarine/resource")
local Vec = require("@vectarine/vec")

type Pos = Coord.ScreenPosition | Vec.Vec2
type Direction = Coord.ScreenVec | Vec.Vec2

--- Functions to create canvas and shaders
--- A canvas is a drawing surface and a shader is a program that changes how a canvas is drawn to another drawing surface.
local module = {}

local ShaderResourceImpl = { type = "shader" }
ShaderResourceImpl.__index = ShaderResourceImpl
export type ShaderResource = typeof(setmetatable({}, ShaderResourceImpl)) & Resource.Resource

local Canvas = {}
Canvas.__index = Canvas
export type Canvas = typeof(setmetatable({}, Canvas))

--- Create a new canvas of given width and height
--- The canvas is empty and fully transparent by default
--- Creating a canvas is an expensive operation! For performance reasons, you shouldn't create canvases every frame!
function module.createCanvas(width: number, height: number): Canvas
	error("Not implemented")
end

--- Paint on the canvas.
---
--- All function from graphics will draw to this canvas instead of the screen until this function returns.
--- You can nest calls to `paint`, in which case the innermost `paint` is the one that receives the draw calls.
---
--- The canvas uses the same coordinate system as the screen with (-1,-1) being the bottom-left, (0,0) the center and (1,1) the top-right.
---
--- Once you have finished painting the canvas, you can use `graphics.drawCanvas` to draw it on the screen or on another canvas.
function Canvas.paint(self: Canvas, draw_function: () -> ()): ()
	error("Not implemented")
end

--- When the canvas is drawn, instead of being drawn as-is, it will be used as a texture input to the shader that
--- you provide. The output of the shader is what will be drawn. This can be used for post-processing effects, outlines, recoloring, etc.
--- To disable the shader, call `setShader(nil)`.
--- You can see examples of shaders on [ShaderToy](https://www.shadertoy.com/)
function Canvas.setShader(self: Canvas, shader: ShaderResource?): ()
	error("Not implemented")
end

--- Sets a uniform float value for the shader used when drawing this canvas.
function Canvas.setUniform(self: Canvas, name: string, value: number): ()
	error("Not implemented")
end

--- Gets the size of the canvas or image in pixels
function Canvas.getSize(self: Canvas): Vec.Vec2
	error("Not implemented")
end

--- Draws a canvas at given position and size
function Canvas.draw(self: Canvas, pos: Vec.Vec2, size: Direction): ()
	error("Not implemented")
end

--- Draws a part canvas stretched to fit the quadrilateral delimited by the 4 destination points
--- This is the same as `drawImagePart` but for canvases
function Canvas.drawPart(
	self: Canvas,
	dest_p1: Pos,
	dest_p2: Pos,
	dest_p3: Pos,
	dest_p4: Pos,
	src_pos: Vec.Vec2,
	src_size: Vec.Vec2
): ()
	error("Not implemented")
end

return module
