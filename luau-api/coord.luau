local Vec = require("@vectarine/vec")

--[[
Creates and converts positions and sizes written in different coordinate systems.

Most drawing functions work in screen coordinates: (-1, -1) is the bottom left of the screen, (1, 1) is the top
right and (0, 0) is the center. The whole screen is 2 wide and 2 high. This is what `gl` means in function names
like `coord.gl`. Pixels are easier to think about when you place things by hand, so this module lets you write a
position in pixels (`coord.px`), in screen coordinates (`coord.gl`), or relative to the width (`coord.vw`) or the
height (`coord.vh`) of the screen. Every function returns a `ScreenPosition` (a point on the screen) or, for the `Vec`
variants, a `ScreenVec` (a distance on the screen). Both can be passed to the functions of `graphics` and `text` in
place of a `Vec2`, and both can be converted back to a `Vec2` with their `gl` and `px` methods, for example `ScreenPosition:px`.

A plain `Vec2` given to a drawing function is read as screen coordinates. Constants such as `coord.CENTER` give
ready-made positions for the corners and the center of the screen.

# Example

```luau
const coord = require("@vectarine/coord")
const graphics = require("@vectarine/graphics")
const vec4 = require("@vectarine/vec4")

function Update(deltaTime: number)
	-- A 100x50 pixels rectangle, 10 pixels away from the top left corner of the window
	graphics.drawRect(coord.px(10, 60), coord.pxVec(100, -50), vec4.RED)
end
```

# Common mistakes

- Pixel functions (`coord.px`, `coord.pxVec`, `ScreenPosition:px`) use the size of the window, not the size of the
  canvas being painted. Pass the canvas size as an extra argument when painting on a `Canvas`.
- In pixels, y goes down (the origin is the top left). In every other system, y goes up. Use a negative y with
  `coord.pxVec` to go up.
- A rectangle is drawn from its bottom left corner, so a position in pixels is the bottom of the rectangle, not its top.
]]
local module = {}

local ScreenVecImpl = { screen_vec = true }
ScreenVecImpl.__index = ScreenVecImpl

--- A distance or a size on the screen, independent of the coordinate system it was written in.
---
--- You get one from `coord.pxVec`, `coord.glVec`, `coord.vwVec` or `coord.vhVec`, or by subtracting two
--- `ScreenPosition`. Add and subtract them with `+` and `-`, scale them with `ScreenVec:scale`, and use them
--- wherever a function expects a size or a direction. Internally it is stored in screen coordinates.
export type ScreenVec = typeof(setmetatable({} :: Vec.Vec2Data, ScreenVecImpl))

--- Returns the vector as a `Vec2` in screen coordinates.
---
--- The whole screen is 2 units wide and 2 units high in this system, and the y axis goes up. No screen size is
--- needed for this conversion.
---
--- @return The vector in screen coordinates.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- local size = coord.glVec(0.5, 0.25)
--- print(size:gl()) -- the same numbers as a Vec2: 0.5, 0.25
--- ```
function ScreenVecImpl.gl(self: ScreenVec): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the vector as a `Vec2` in pixels.
---
--- The y axis of the result goes down, as for `coord.px`. The pixel size is computed from the size of the window
--- (in the same units as `coord.px`), not from the canvas currently being painted, unless you pass `screen_size`.
---
--- @param screen_size The size in pixels to convert against, for example the size of a canvas. Defaults to the size of
--- the window.
--- @return The vector in pixels.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- local size = coord.glVec(0.5, 0.5)
--- -- In an 800x600 window, a quarter of the width and of the height:
--- print(size:px()) -- 200, -150
--- ```
function ScreenVecImpl.px(self: ScreenVec, screen_size: Vec.Vec2?): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the vector multiplied by a number.
---
--- @param k The factor. A negative value flips the direction of the vector.
--- @return A new `ScreenVec`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- local step = coord.pxVec(10, 0)
--- local bigStep = step:scale(3) -- 30 pixels to the right
--- ```
function ScreenVecImpl.scale(self: ScreenVec, k: number): ScreenVec
	error("Implemented in native code")
end

--- Adds two `ScreenVec`: `a + b` is the distance covered by walking `a` and then `b`.
ScreenVecImpl.__add = function(a: ScreenVec, b: ScreenVec): ScreenVec
	error("Implemented in native code")
end

--- Subtracts two `ScreenVec`: `a - b` is the distance that goes from `b` to `a`.
ScreenVecImpl.__sub = function(a: ScreenVec, b: ScreenVec): ScreenVec
	error("Implemented in native code")
end

--------------------------------------

local ScreenPositionImpl = { screen_position = true }
ScreenPositionImpl.__index = ScreenPositionImpl

--- Moves a `ScreenPosition` by a `ScreenVec`: `position + vector` is the position reached after walking `vector`.
---
--- Adding two positions raises an error, since it has no meaning. Subtract them to get a `ScreenVec` instead.
ScreenPositionImpl.__add = function(a: ScreenPosition, b: ScreenVec): ScreenPosition
	error("Implemented in native code")
end

local sub_function: ((ScreenPosition, ScreenPosition) -> ScreenVec) & ((ScreenPosition, ScreenVec) -> ScreenPosition) & ((ScreenPosition, Vec.Vec2) -> ScreenPosition) = function(
	a: any,
	b: any
): any
	error("Implemented in native code")
end
--- Subtracts from a `ScreenPosition`. The result depends on the right operand:
---
--- - `position - position` is the `ScreenVec` that goes from the second position to the first one.
--- - `position - vector` is the `ScreenPosition` reached by walking back along `vector`.
ScreenPositionImpl.__sub = sub_function

--- Returns the position as a `Vec2` in screen coordinates.
---
--- In this system, (-1, -1) is the bottom left of the screen, (0, 0) its center and (1, 1) its top right. No screen
--- size is needed for this conversion.
---
--- @return The position in screen coordinates.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- print(coord.TOP_LEFT:gl()) -- -1, 1
--- ```
function ScreenPositionImpl.gl(self: ScreenPosition): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the position as a `Vec2` in pixels.
---
--- The origin (0, 0) is the top left of the screen and the y axis goes down, as for `coord.px`.
---
--- @param screen_size The size in pixels to convert against. Defaults to the size of the window, as given by
--- `io.getWindowSize` (the canvas currently being painted is not used).
--- @return The position in pixels.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- -- In an 800x600 window:
--- print(coord.CENTER:px()) -- 400, 300
--- print(coord.BOTTOM_RIGHT:px()) -- 800, 600
--- ```
function ScreenPositionImpl.px(self: ScreenPosition, screen_size: Vec.Vec2?): Vec.Vec2
	error("Implemented in native code")
end

--- A point on the screen, independent of the coordinate system it was written in.
---
--- You get one from `coord.px`, `coord.gl`, `coord.vw` or `coord.vh`, from the constants such as `coord.CENTER`, or by
--- adding a `ScreenVec` to another position. Pass it to any function that draws at a position. Internally it is stored
--- in screen coordinates. Convert it back with `ScreenPosition:gl` or `ScreenPosition:px`.
export type ScreenPosition = typeof(setmetatable({} :: Vec.Vec2Data, ScreenPositionImpl))

--- Creates a `ScreenPosition` from a position in pixels.
---
--- The origin (0, 0) is the top left of the screen, x goes right and y goes down. The bottom right corner is
--- (width, height). This is the same convention as HTML5 canvas, Love2D, Processing or Raylib.
---
--- You can call it in three ways: `coord.px(10, 20)`, `coord.px(vec.V2(10, 20))`, or with an explicit screen size as
--- the last argument: `coord.px(10, 20, size)` or `coord.px(vec.V2(10, 20), size)`.
---
--- The conversion needs the size of the screen. By default it uses the size of the window (the same numbers as
--- `io.getWindowSize`), even while a canvas is being painted. Pass a size to convert for something else, for example
--- the size of a canvas.
---
--- @param v_or_x The position as a `Vec2`, or only its x component (in pixels) when the second argument is the y
--- component.
--- @param screen_size_px_or_y The y component (in pixels) when the first argument is a number, otherwise the screen
--- size as a `Vec2`.
--- @param screen_size_px_or_nil The screen size as a `Vec2`, when the first two arguments are numbers.
--- @return The position as a `ScreenPosition`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the first two arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A 50x50 pixels square, 20 pixels from the top left corner. A rectangle is drawn from its bottom left
--- 	-- corner, so the position is 50 pixels lower than the top of the square and the size goes up.
--- 	graphics.drawRect(coord.px(20, 70), coord.pxVec(50, -50), vec4.BLUE)
--- 	-- The same, using a Vec2 and a fixed screen size of 320x240
--- 	graphics.drawRect(coord.px(vec.V2(20, 70), vec.V2(320, 240)), coord.pxVec(50, -50, vec.V2(320, 240)), vec4.RED)
--- end
--- ```
---
--- # See also
---
--- - `coord.pxVec`: a size or distance in pixels.
--- - `coord.vw`: positions relative to the screen width.
function module.px(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenPosition
	error("Implemented in native code")
end

--- Creates a `ScreenVec` from a distance in pixels.
---
--- A distance has no origin. As in `coord.px`, positive x goes right and positive y goes down, so use a negative y
--- to go up.
---
--- It takes the same arguments as `coord.px`: two numbers, or a `Vec2`, optionally followed by a screen size. By
--- default the size of the window is used, even while a canvas is being painted.
---
--- @param v_or_x The distance as a `Vec2`, or only its x component (in pixels) when the second argument is the y
--- component.
--- @param screen_size_px_or_y The y component (in pixels, positive goes down) when the first argument is a number,
--- otherwise the screen size as a `Vec2`.
--- @param screen_size_px_or_nil The screen size as a `Vec2`, when the first two arguments are numbers.
--- @return The distance as a `ScreenVec`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the first two arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- local size = coord.pxVec(100, 40)
--- -- In an 800x600 window, 100 pixels are a quarter of the screen width in screen coordinates:
--- print(size:gl()) -- 0.25, -0.1333
--- ```
function module.pxVec(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenVec
	error("Implemented in native code")
end

--- Creates a `ScreenPosition` from screen coordinates.
---
--- The origin (0, 0) is the center of the screen, the bottom left is (-1, -1) and the top right is (1, 1). The x axis
--- goes right and the y axis goes up. The screen is 2 units wide and 2 units high, whatever its shape, so a square of
--- size (1, 1) is not square on a window that is not square.
---
--- This is the system that a `Vec2` uses when you pass it to a drawing function.
---
--- @param v_or_x The position as a `Vec2`, or only its x component when the second argument is the y component.
--- @param y_or_nil The y component when the first argument is a number.
--- @return The position as a `ScreenPosition`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
--- const graphics = require("@vectarine/graphics")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A circle in the upper right quarter of the screen
--- 	graphics.drawCircle(coord.gl(0.5, 0.5), 0.1, vec4.GREEN)
--- end
--- ```
---
--- # See also
---
--- - `coord.glVec`: a size or distance in screen coordinates.
--- - `ScreenPosition:gl`: convert back.
function module.gl(v_or_x: Vec.Vec2 | number, y_or_nil: number | Vec.Vec2?): ScreenPosition
	error("Implemented in native code")
end

--- Creates a `ScreenVec` from a distance in screen coordinates.
---
--- The whole screen is 2 units wide and 2 units high, and the y axis goes up.
---
--- @param v_or_x The distance as a `Vec2`, or only its x component when the second argument is the y component.
--- @param y_or_nil The y component when the first argument is a number.
--- @return The distance as a `ScreenVec`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- local half = coord.glVec(1, 1) -- half the width and half the height of the screen
--- print(half:px()) -- 400, -300 in an 800x600 window
--- ```
function module.glVec(v_or_x: Vec.Vec2 | number, y_or_nil: number | Vec.Vec2?): ScreenVec
	error("Implemented in native code")
end

--- Creates a `ScreenPosition` measured in hundredths of the screen width.
---
--- The origin is the bottom left of the screen and both axes go up and right (unlike `coord.px`). One unit is 1% of
--- the screen width on both axes, so `coord.vw(50, 0)` is the middle of the bottom edge, and `coord.vw(10, 10)` is
--- 10% of the width away from the left edge and from the bottom edge, whatever the shape of the window. This name
--- comes from the CSS unit of the same name.
---
--- It takes the same arguments as `coord.px`: two numbers, or a `Vec2`, optionally followed by a screen size. By
--- default the size of the window is used, even while a canvas is being painted.
---
--- @param v_or_x The position as a `Vec2`, or only its x component when the second argument is the y component.
--- @param screen_size_px_or_y The y component when the first argument is a number, otherwise the screen size as a
--- `Vec2`.
--- @param screen_size_px_or_nil The screen size as a `Vec2`, when the first two arguments are numbers.
--- @return The position as a `ScreenPosition`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the first two arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
--- const graphics = require("@vectarine/graphics")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A square whose sides are 10% of the screen width, 5% from the left and bottom edges
--- 	graphics.drawRect(coord.vw(5, 5), coord.vwVec(10, 10), vec4.RED)
--- end
--- ```
function module.vw(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenPosition
	error("Implemented in native code")
end

--- Creates a `ScreenVec` measured in hundredths of the screen width.
---
--- One unit is 1% of the screen width on both axes, and the y axis goes up. See `coord.vw`.
---
--- @param v_or_x The distance as a `Vec2`, or only its x component when the second argument is the y component.
--- @param screen_size_px_or_y The y component when the first argument is a number, otherwise the screen size as a
--- `Vec2`.
--- @param screen_size_px_or_nil The screen size as a `Vec2`, when the first two arguments are numbers.
--- @return The distance as a `ScreenVec`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the first two arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- local size = coord.vwVec(50, 0) -- half of the screen width
--- print(size:gl()) -- 1, 0
--- ```
function module.vwVec(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenVec
	error("Implemented in native code")
end

--- Creates a `ScreenPosition` measured in hundredths of the screen height.
---
--- The origin is the bottom left of the screen and both axes go up and right. One unit is 1% of the screen height on
--- both axes, so `coord.vh(0, 50)` is the middle of the left edge. This is the same as `coord.vw`, but based on the
--- height. It is handy for games that scale with the height of the window.
---
--- It takes the same arguments as `coord.px`: two numbers, or a `Vec2`, optionally followed by a screen size. By
--- default the size of the window is used, even while a canvas is being painted.
---
--- @param v_or_x The position as a `Vec2`, or only its x component when the second argument is the y component.
--- @param screen_size_px_or_y The y component when the first argument is a number, otherwise the screen size as a
--- `Vec2`.
--- @param screen_size_px_or_nil The screen size as a `Vec2`, when the first two arguments are numbers.
--- @return The position as a `ScreenPosition`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the first two arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
--- const graphics = require("@vectarine/graphics")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A square whose sides are 20% of the screen height, in the bottom left corner
--- 	graphics.drawRect(coord.vh(0, 0), coord.vhVec(20, 20), vec4.BLUE)
--- end
--- ```
function module.vh(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenPosition
	error("Implemented in native code")
end

--- Creates a `ScreenVec` measured in hundredths of the screen height.
---
--- One unit is 1% of the screen height on both axes, and the y axis goes up. See `coord.vh`.
---
--- @param v_or_x The distance as a `Vec2`, or only its x component when the second argument is the y component.
--- @param screen_size_px_or_y The y component when the first argument is a number, otherwise the screen size as a
--- `Vec2`.
--- @param screen_size_px_or_nil The screen size as a `Vec2`, when the first two arguments are numbers.
--- @return The distance as a `ScreenVec`.
---
--- # Errors
---
--- Raises an error (`"Invalid arguments"`) if the first two arguments are neither two numbers nor a `Vec2`.
---
--- # Example
---
--- ```luau
--- const coord = require("@vectarine/coord")
---
--- local size = coord.vhVec(0, 50) -- half of the screen height
--- print(size:gl()) -- 0, 1
--- ```
function module.vhVec(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenVec
	error("Implemented in native code")
end

--- The center of the screen, (0, 0) in screen coordinates.
module.CENTER = module.gl(Vec.V2(0, 0))
--- The top left corner of the screen, (-1, 1) in screen coordinates.
module.TOP_LEFT = module.gl(Vec.V2(-1, 1))
--- The top right corner of the screen, (1, 1) in screen coordinates.
module.TOP_RIGHT = module.gl(Vec.V2(1, 1))
--- The bottom left corner of the screen, (-1, -1) in screen coordinates.
module.BOTTOM_LEFT = module.gl(Vec.V2(-1, -1))
--- The bottom right corner of the screen, (1, -1) in screen coordinates.
module.BOTTOM_RIGHT = module.gl(Vec.V2(1, -1))

return module
