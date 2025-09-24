local Resources = require("@vectarine/resources")

local module = {}

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
function module.paint(self: Canvas, draw_function: () -> ()): ()
    error("Not implemented")
end

--- When the canvas is drawn, instead of being drawn as-is, it will be used as a texture input to the shader that
--- you provide. The output of the shader is what will be drawn. This can be used for post-processing effects, outlines, recoloring, etc.
--- To disable the shader, call `setShader(nil)`.
--- You can see examples of shaders on [ShaderToy](https://www.shadertoy.com/)
function Canvas.setShader(self: Canvas, shader: Resources.ShaderResource?): ()
    error("Not implemented")
end

return module
