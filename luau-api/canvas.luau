local coord = require("@vectarine/coord")
local resource = require("@vectarine/resource")
local vec = require("@vectarine/vec")
local vec4 = require("@vectarine/vec4")

--- A position given as a `ScreenPosition`, or as a `Vec2` in screen coordinates.
type Pos = coord.ScreenPosition | vec.Vec2
--- A size or direction given as a `ScreenVec`, or as a `Vec2` in screen coordinates.
type Direction = coord.ScreenVec | vec.Vec2

--[[
Creates canvases, which are images you draw on, and the shaders that can change how a canvas looks when it is drawn.

A canvas is a surface that you paint on with the functions of `graphics`, `text` and the other drawing modules, instead
of painting on the screen. Once painted, you draw the canvas like an image, which lets you reuse a complex drawing, draw
it scaled or in several places, or apply an effect to it. A shader is a small GPU program loaded with `loader.loadShader`
that receives the canvas as a texture and decides the color of each pixel, for example for blur, outlines or color
changes. See `graphics` for the drawing functions and `Canvas:paint` to paint on a canvas.

# Example

```luau
const canvas = require("@vectarine/canvas")
const graphics = require("@vectarine/graphics")
const coord = require("@vectarine/coord")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

local minimap = canvas.createCanvas(128, 128)

function Update(deltaTime: number)
	minimap:paint(function()
		graphics.clear(vec4.BLACK)
		graphics.drawCircle(vec.V2(0, 0), 0.1, vec4.RED)
	end)
	minimap:draw(vec.V2(0.5, 0.5), vec.V2(0.4, 0.4))
end
```

# Common mistakes

- Creating a canvas is expensive. Do not call `canvas.createCanvas` every frame: create it once and reuse it.
- `Canvas:paint` does not clear the canvas, so drawings pile up from one frame to the next. Call `graphics.clear`
  first.
- A shader that is not loaded yet draws nothing, and `Canvas:setUniform` is ignored until it is ready.
]]
local module = {}

local ShaderResourceImpl = { type = "shader" }
ShaderResourceImpl.__index = ShaderResourceImpl

--- A shader that can be applied to a canvas.
---
--- You get one from `loader.loadShader`. It is a resource, so it loads in the background: check `Resource:isReady` if
--- you need to know when it can be used. Attach it to a canvas with `Canvas:setShader`.
export type ShaderResource = typeof(setmetatable({}, ShaderResourceImpl)) & resource.Resource

local Canvas = {}
Canvas.__index = Canvas

--- A drawing surface that you paint on and then draw like an image.
---
--- You get one from `canvas.createCanvas`. Paint on it with `Canvas:paint`, draw it with `Canvas:draw` or
--- `Canvas:drawPart`, and optionally change how it looks with `Canvas:setShader`. A canvas has a fixed size in pixels.
export type Canvas = typeof(setmetatable({}, Canvas))

--- Creates an empty canvas of a given size in pixels.
---
--- The canvas starts fully transparent. Its size cannot change afterwards: create a new canvas if you need another
--- size. Its pixels are smoothed when it is drawn smaller or larger.
---
--- Creating a canvas is expensive. Create canvases once, when the game starts, and reuse them. Do not create one every
--- frame.
---
--- @param width The width of the canvas in pixels. It must be a whole number.
--- @param height The height of the canvas in pixels. It must be a whole number.
--- @return The new `Canvas`.
---
--- # Errors
---
--- Raises an error if `width` or `height` cannot be converted to a non-negative whole number, for example a negative
--- number.
---
--- # Example
---
--- ```luau
--- const canvas = require("@vectarine/canvas")
---
--- -- Created once, outside of Update
--- local buffer = canvas.createCanvas(256, 144)
--- print(buffer:getSize()) -- 256, 144
--- ```
---
--- # See also
---
--- - `Canvas:paint`: draw on the canvas.
--- - `Canvas:draw`: draw the canvas on the screen.
function module.createCanvas(width: number, height: number): Canvas
	error("Implemented in native code")
end

--- Runs a function whose drawing goes to the canvas instead of the screen.
---
--- All drawing functions called by `draw_function` (those of `graphics`, `text`, images, and also `Canvas:draw` of
--- another canvas) draw on this canvas until the function returns. You can call `paint` inside another `paint`: the
--- innermost canvas receives the drawing while its function runs.
---
--- The canvas uses screen coordinates, like the screen: (-1, -1) is its bottom left, (0, 0) its center and (1, 1) its
--- top right, whatever its size in pixels. Pixel positions made with `coord.px` are computed from the size of the
--- window, not of the canvas, unless you pass the canvas size to them.
---
--- Painting does not clear the canvas: what was drawn before stays, and new drawings are added on top. Call
--- `graphics.clear` at the start of the function to repaint it from scratch.
---
--- Once painted, draw the canvas with `Canvas:draw` or `Canvas:drawPart`. If you paint the same content every frame,
--- consider painting only when it changes.
---
--- @param draw_function The function that does the drawing. It receives no argument.
---
--- # Errors
---
--- If `draw_function` raises an error, the error is raised again by `paint`, after the drawing it already did has been
--- sent to the canvas.
---
--- # Example
---
--- ```luau
--- const canvas = require("@vectarine/canvas")
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local sprite = canvas.createCanvas(64, 64)
---
--- function Update(deltaTime: number)
--- 	sprite:paint(function()
--- 		graphics.clear(vec4.TRANSPARENT)
--- 		graphics.drawRect(vec.V2(-0.5, -0.5), vec.V2(1, 1), vec4.GREEN)
--- 	end)
--- end
--- ```
---
--- # See also
---
--- - `Canvas:draw`: draw the painted canvas.
--- - `graphics.clear`: start from an empty canvas.
function Canvas.paint(self: Canvas, draw_function: () -> ()): ()
	error("Implemented in native code")
end

--- Sets the shader used to draw this canvas, or removes it.
---
--- When the canvas is drawn with `Canvas:draw` or `Canvas:drawPart`, it is not copied as is. Instead, the shader
--- receives the canvas as a texture and its output is what appears on screen. Use it for post-processing effects,
--- outlines, recoloring and so on. The shader sees the canvas as a texture named `tex` and the time in seconds since
--- the game started as a float named `iTime`, in the style of [ShaderToy](https://www.shadertoy.com/), where you can
--- find many examples.
---
--- Pass `nil` to go back to drawing the canvas as is. The shader only affects drawing: painting on the canvas is not
--- changed.
---
--- If the shader is not loaded yet, or failed to load or compile, nothing is drawn when the canvas is drawn. Wait for
--- `Resource:isReady` if you do not want the canvas to disappear for a few frames.
---
--- @param shader The shader to use, from `loader.loadShader`, or `nil` to remove the shader.
---
--- # Example
---
--- ```luau
--- const canvas = require("@vectarine/canvas")
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- local scene = canvas.createCanvas(320, 180)
--- local blur = loader.loadShader("shaders/blur.glsl")
--- scene:setShader(blur)
---
--- function Update(deltaTime: number)
--- 	scene:draw(vec.V2(-1, -1), vec.V2(2, 2))
--- end
--- ```
---
--- # See also
---
--- - `loader.loadShader`: load a shader.
--- - `Canvas:setUniform`: send values to the shader.
function Canvas.setShader(self: Canvas, shader: ShaderResource?): ()
	error("Implemented in native code")
end

--- Sets a uniform of the shader of this canvas, a value that the shader can read.
---
--- Use it to send values from your game to the shader, for example the strength of an effect. `name` is the name of
--- the uniform in the shader code. A number is sent as a float, a `Vec2` as a `vec2` and a `Vec4` as a `vec4`. Whole
--- numbers are also sent as floats.
---
--- The value is set on the shader itself, so canvases that use the same shader share it.
---
--- If the canvas has no shader, or the shader is not loaded or compiled yet, the call does nothing and the value is
--- lost. Call it again once the shader is ready.
---
--- If the shader has no uniform with this name (for example because it never uses it and the compiler removed it), a
--- warning is printed in the console and nothing else happens.
---
--- @param name The name of the uniform in the shader.
--- @param value The value to set: a number, a `Vec2` or a `Vec4`.
---
--- # Errors
---
--- Raises an error if `value` is of another type, for example a string. This is only checked once the shader is ready.
---
--- # Example
---
--- ```luau
--- const canvas = require("@vectarine/canvas")
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- local scene = canvas.createCanvas(320, 180)
--- local wave = loader.loadShader("shaders/wave.glsl")
--- scene:setShader(wave)
---
--- function Update(deltaTime: number)
--- 	scene:setUniform("amplitude", 0.05)
--- 	scene:draw(vec.V2(-1, -1), vec.V2(2, 2))
--- end
--- ```
function Canvas.setUniform(self: Canvas, name: string, value: number | vec.Vec2 | vec4.Vec4): ()
	error("Implemented in native code")
end

--- Returns the size of the canvas in pixels.
---
--- This is the size given to `canvas.createCanvas`. It is a size in pixels, not in screen coordinates.
---
--- @return The width in `x` and the height in `y`.
---
--- # Example
---
--- ```luau
--- const canvas = require("@vectarine/canvas")
---
--- local buffer = canvas.createCanvas(200, 100)
--- local size = buffer:getSize()
--- print(size.x / size.y) -- 2
--- ```
function Canvas.getSize(self: Canvas): vec.Vec2
	error("Implemented in native code")
end

--- Draws the canvas at a position, stretched to a size.
---
--- `pos` is the bottom left corner of the canvas and `size` goes right and up from there. Both are in screen
--- coordinates, where the screen is 2 wide and 2 high. The canvas is stretched to `size` whatever its size in pixels,
--- so keep its proportions in mind if you do not want it to look squashed.
---
--- If the canvas has a shader (see `Canvas:setShader`), the shader decides how it is drawn. The current transformation
--- of `graphics.withTransformation` applies.
---
--- @param pos The position of the bottom left corner of the canvas.
--- @param size The size to draw it at. Positive values go right and up.
---
--- # Example
---
--- ```luau
--- const canvas = require("@vectarine/canvas")
--- const vec = require("@vectarine/vec")
---
--- local buffer = canvas.createCanvas(128, 128)
---
--- function Update(deltaTime: number)
--- 	-- A square in the top right quarter of the screen
--- 	buffer:draw(vec.V2(0, 0), vec.V2(1, 1))
--- end
--- ```
---
--- # See also
---
--- - `Canvas:drawPart`: draw only a region of the canvas.
--- - `Canvas:setShader`: change how the canvas looks.
function Canvas.draw(self: Canvas, pos: Pos, size: Direction): ()
	error("Implemented in native code")
end

--- Draws a part of the canvas, stretched to fit a quadrilateral given by 4 corners.
---
--- Use it to draw only a region of a canvas, or to draw it as a rotated or skewed shape. The destination corners are
--- in screen coordinates, and must be given in this order: bottom left, bottom right, top right, top left. The source
--- region is given as fractions of the canvas rather than pixels: `src_pos` is its bottom left corner and `src_size`
--- its size, where (0, 0) is the bottom left of the canvas and (1, 1) its top right. So `(vec.V2(0, 0), vec.V2(1, 1))`
--- is the whole canvas, which is what `Canvas:draw` uses.
---
--- Unlike `ImageResource:drawPart`, where `src_pos` is the top left corner of the part and y goes down, here (0, 0) is
--- the bottom left corner of the canvas and y goes up, like everywhere else on screen.
---
--- If the canvas has a shader (see `Canvas:setShader`), the shader decides how it is drawn. The current transformation
--- of `graphics.withTransformation` applies.
---
--- @param dest_p1 The destination of the bottom left corner of the region.
--- @param dest_p2 The destination of the bottom right corner of the region.
--- @param dest_p3 The destination of the top right corner of the region.
--- @param dest_p4 The destination of the top left corner of the region.
--- @param src_pos The bottom left corner of the region to draw, as fractions from 0 to 1 of the size of the canvas.
--- @param src_size The size of the region to draw, as fractions from 0 to 1 of the size of the canvas.
---
--- # Example
---
--- ```luau
--- const canvas = require("@vectarine/canvas")
--- const vec = require("@vectarine/vec")
---
--- local atlas = canvas.createCanvas(256, 256)
---
--- function Update(deltaTime: number)
--- 	-- Draw the top right quarter of the canvas over the whole screen
--- 	atlas:drawPart(
--- 		vec.V2(-1, -1), vec.V2(1, -1), vec.V2(1, 1), vec.V2(-1, 1),
--- 		vec.V2(0.5, 0.5), vec.V2(0.5, 0.5)
--- 	)
--- end
--- ```
function Canvas.drawPart(
	self: Canvas,
	dest_p1: Pos,
	dest_p2: Pos,
	dest_p3: Pos,
	dest_p4: Pos,
	src_pos: vec.Vec2,
	src_size: vec.Vec2
): ()
	error("Implemented in native code")
end

return module
