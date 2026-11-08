local Coord = require("@vectarine/coord")
local Resource = require("@vectarine/resource")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--[[
Draws shapes on the screen and transforms what is drawn.

Call these functions every frame from `Update`. Positions are screen coordinates: `(-1, -1)` is the bottom left of the
screen and `(1, 1)` is the top right. Every position or size can be given as a `Vec2` or as a `ScreenPosition` or
`ScreenVec` made with the `coord` module. Colors are `Vec4` values from the `vec4` module, with components between `0`
and `1`. To draw images, see `ImageResource:draw` in the `image` module. To move or scale a whole scene, see
`graphics.withTransformation` or the `camera` module.

# Example

```luau
const graphics = require("@vectarine/graphics")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

function Update(deltaTime: number)
	graphics.clear(vec4.WHITE)
	graphics.drawCircle(vec.V2(0, 0), 0.2, vec4.RED)
end
```

# Common mistakes

- Positions are screen coordinates, not pixels: `(-1, -1)` is the bottom left, `(1, 1)` the top right and `(0, 0)` the
  center. The y axis goes up.
- Drawing is not kept between frames. Call `graphics.clear` and draw everything again in every `Update`.
]]
local module = {}

--- A position on the screen. It is either a `Vec2` in screen coordinates or a `ScreenPosition` from the `coord` module.
type Pos = Coord.ScreenPosition | Vec.Vec2
--- A size or direction on the screen. It is either a `Vec2` in screen units or a `ScreenVec` from the `coord` module.
type Direction = Coord.ScreenVec | Vec.Vec2

--- Fills the whole canvas with one color.
---
--- Call it at the start of `Update` to erase what was drawn during the previous frame.
---
--- @param color The color to fill the canvas with. Defaults to white.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	graphics.clear(vec4.DARK_BLUE)
--- end
--- ```
function module.clear(color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draws a filled rectangle.
---
--- `pos` is a corner of the rectangle and `size` goes from that corner to the opposite one. With a positive size,
--- `pos` is the bottom left corner. A negative size makes the rectangle extend to the left or downward instead.
---
--- @param pos The corner of the rectangle. With a positive size, this is the bottom left corner.
--- @param size The width and height of the rectangle.
--- @param color The fill color. Defaults to black.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A red square centered on the screen.
--- 	graphics.drawRect(vec.V2(-0.05, -0.05), vec.V2(0.1, 0.1), vec4.RED)
--- end
--- ```
---
--- # See also
---
--- - `ImageResource:draw`: draws an image in a rectangle.
--- - `graphics.drawCircle`: draws a filled circle.
function module.drawRect(pos: Pos, size: Direction, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draws a filled convex polygon.
---
--- The points are joined in the order given. The polygon is filled as a fan starting at the first point, so it only
--- looks right for convex shapes. Nothing is drawn if there are fewer than `3` points.
---
--- @param points The corners of the polygon, in order.
--- @param color The fill color. Defaults to black.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A triangle.
--- 	graphics.drawPolygon({ vec.V2(-0.2, -0.2), vec.V2(0.2, -0.2), vec.V2(0, 0.2) }, vec4.GREEN)
--- end
--- ```
function module.drawPolygon(points: { Pos }, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draws a straight line between two points.
---
--- The line is a rectangle with square ends. `thickness` is measured in screen units, from the middle of the line to
--- one edge, so the visible width is twice `thickness`. Nothing visible is drawn if both points are the same.
---
--- @param p1 The start of the line.
--- @param p2 The end of the line.
--- @param color The color of the line. Defaults to black.
--- @param thickness Half the width of the line, in screen units. Defaults to `0.005`.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const coord = require("@vectarine/coord")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A line from the center of the screen to the top right corner.
--- 	graphics.drawLine(coord.CENTER, coord.TOP_RIGHT, vec4.BLACK, 0.01)
--- end
--- ```
---
--- # See also
---
--- - `graphics.drawArrow`: draws a line with an arrow head.
function module.drawLine(p1: Pos, p2: Pos, color: Vec4.Vec4?, thickness: number?): ()
	error("Implemented in native code")
end

--- Draws an arrow that starts at a position and points along a direction.
---
--- The shaft goes from `pos` to `pos + direction`. The head is drawn after the end of the shaft, so the tip of the
--- arrow is a little further than `pos + direction`. Use it to show the speed or the heading of an entity. Nothing is
--- drawn if `direction` has a length of `0`.
---
--- @param pos Where the arrow starts.
--- @param direction The vector the arrow follows. Its length is the length of the shaft.
--- @param color The color of the arrow. Defaults to black.
--- @param size The width of the shaft, in screen units. The head is twice as large. Defaults to `0.01`.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local position = vec.V2(0, 0)
--- local velocity = vec.V2(0.3, 0.1)
---
--- function Update(deltaTime: number)
--- 	graphics.drawArrow(position, velocity, vec4.RED)
--- end
--- ```
function module.drawArrow(pos: Pos, direction: Direction, color: Vec4.Vec4?, size: number?): ()
	error("Implemented in native code")
end

--- Draws a filled circle.
---
--- The radius is relative to the height of the screen, and the circle stays round whatever the shape of the window.
--- A radius of `1` reaches the top and bottom edges of the screen.
---
--- @param center The center of the circle.
--- @param radius The radius, in screen units.
--- @param color The fill color. Defaults to black.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	graphics.drawCircle(vec.V2(0.5, 0.5), 0.1, vec4.ORANGE)
--- end
--- ```
---
--- # See also
---
--- - `graphics.drawEllipse`: draws an ellipse with two radii.
--- - `graphics.drawRect`: draws a filled rectangle.
function module.drawCircle(center: Pos, radius: number, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draws a filled ellipse.
---
--- `size` holds the two radii of the ellipse, so the ellipse fits in the rectangle that is `size * 2` wide and tall,
--- centered on `center`. Unlike `graphics.drawCircle`, the radii are not corrected for the shape of the window.
---
--- @param center The center of the ellipse.
--- @param size The horizontal and vertical radii.
--- @param color The fill color. Defaults to black.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- A wide, flat ellipse, useful as a shadow under a character.
--- 	graphics.drawEllipse(vec.V2(0, -0.3), vec.V2(0.15, 0.04), vec4.DARK_GRAY)
--- end
--- ```
function module.drawEllipse(center: Pos, size: Direction, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Runs `drawFunction` so that it only appears where `maskFunction` drew.
---
--- `maskFunction` runs first. Its drawing is not visible and its colors do not matter, only the places it covered.
--- Then `drawFunction` runs and is cut to those places. Use it to crop an image or a scene to a shape, like a circle.
---
--- Everything drawn before the call is flushed first, which has a small cost. Avoid calling it many times per frame.
---
--- @param drawFunction The drawing to crop.
--- @param maskFunction The drawing that defines the visible area.
---
--- # Errors
---
--- Errors raised inside either function are raised again by `graphics.drawWithMask`.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	graphics.drawWithMask(function()
--- 		graphics.drawRect(vec.V2(-1, -1), vec.V2(2, 2), vec4.RED)
--- 	end, function()
--- 		graphics.drawCircle(vec.V2(0, 0), 0.4, vec4.WHITE)
--- 	end)
--- end
--- ```
function module.drawWithMask(drawFunction: () -> (), maskFunction: () -> ()): ()
	error("Implemented in native code")
end

--- Shows a splash screen over the whole screen.
---
--- It clears the screen to black, then draws the Vectarine logo and a progress bar. Use it to show a loading animation
--- while the game is loading. You can configure the splash screen using the `loading_animation` field in the `.vecta`
--- manifest. Call `graphics.drawSplashScreenIfNeeded` instead if you only need to wait for resources.
---
--- @param loadingText A text shown under the logo. Defaults to no text.
--- @param progress How full the progress bar is, between `0` and `1`. Values outside this range are clamped. If it is
--- `nil`, the bar shows a moving animation instead.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
---
--- function Update(deltaTime: number)
--- 	graphics.drawSplashScreen("Generating the world...", 0.5)
--- end
--- ```
function module.drawSplashScreen(loadingText: string?, progress: number?): ()
	error("Implemented in native code")
end

--- Shows the splash screen if any of the resources is not ready yet.
---
--- The progress bar shows how many of the resources are ready. A value that is not a resource counts as not ready. With
--- an empty list, it returns `false` and draws nothing. Return from `Update` when it returns `true`, so that your game
--- code does not use resources that are not loaded.
---
--- @param resourcesToWaitFor The resources to wait for, such as images or sounds from the `loader` module.
--- @param loadingText A text shown under the logo. Defaults to no text.
--- @return True if the splash screen was drawn (some resources are not ready), `false` if everything is ready.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- const ballImage = loader.loadImage("textures/ball.png")
--- const jumpSound = loader.loadAudio("audio/jump.wav")
---
--- function Update(deltaTime: number)
--- 	if graphics.drawSplashScreenIfNeeded({ ballImage, jumpSound }, "Starting...") then
--- 		return
--- 	end
--- 	ballImage:draw(vec.V2(0, 0), vec.V2(0.2, 0.2))
--- end
--- ```
---
--- # See also
---
--- - `graphics.drawSplashScreen`: shows the splash screen with a progress you choose.
--- - `Resource:isReady`: tells if one resource is ready.
function module.drawSplashScreenIfNeeded(resourcesToWaitFor: { Resource.Resource }, loadingText: string?): boolean
	error("Implemented in native code")
end

-- MARK: Transformations

--- Runs `drawFunction` with a translation, a scale and a rotation applied to everything it draws.
---
--- Use it to move, scale or rotate a group of drawings without changing their code. The transformation is
--- applied in this order: translation, then scale, then rotation around the origin `(0, 0)`. The translation is in the
--- local space, so it is also scaled and rotated. Nested calls combine their transformations, and the previous
--- transformation is restored when `drawFunction` returns. The transformation only applies to 2D drawing functions.
---
--- It is used internally to implement the UI system.
---
--- @param transformation A table with optional fields: `translation` (a `Vec2`, defaults to no translation),
--- `scale` (a `Vec2`, defaults to `V2(1, 1)`) and `rotation` (an angle in radians, defaults to `0`).
--- @param drawFunction The function to run with the transformation applied.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- The square is drawn rotated by 45 degrees and twice as big.
--- 	graphics.withTransformation({ scale = vec.V2(2, 2), rotation = math.pi / 4 }, function()
--- 		graphics.drawRect(vec.V2(-0.05, -0.05), vec.V2(0.1, 0.1), vec4.BLUE)
--- 	end)
--- end
--- ```
---
--- # See also
---
--- - `graphics.withoutTransformation`: resets the transformation.
--- - `Camera2:screen`: converts a world position with a camera.
function module.withTransformation(
	transformation: { translation: Vec.Vec2?, scale: Vec.Vec2?, rotation: number? },
	drawFunction: () -> ()
): ()
	error("Implemented in native code")
end

--- Runs `drawFunction` without any transformation.
---
--- Inside `drawFunction`, the transformation is reset to the default. You can still call `graphics.withTransformation`
--- there. The previous transformation is restored when `drawFunction` returns. Use it to draw something fixed on the
--- screen, like a score, inside a transformed scene.
---
--- @param drawFunction The function to run with no transformation.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	graphics.withTransformation({ scale = vec.V2(0.5, 0.5) }, function()
--- 		graphics.withoutTransformation(function()
--- 			-- Drawn at full size, as if there was no transformation.
--- 			graphics.drawRect(vec.V2(-0.9, 0.8), vec.V2(0.2, 0.1), vec4.RED)
--- 		end)
--- 	end)
--- end
--- ```
function module.withoutTransformation(drawFunction: () -> ()): ()
	error("Implemented in native code")
end

--- Applies the current transformation of `graphics.withTransformation` to a position.
---
--- Outside of `graphics.withTransformation`, the position is returned unchanged.
---
--- @param pos A position in the local space.
--- @return The same position, in screen coordinates.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
---
--- graphics.withTransformation({ translation = vec.V2(0.5, 0) }, function()
--- 	local onScreen = graphics.transform(vec.V2(0, 0))
--- 	print(onScreen.x) -- 0.5
--- end)
--- ```
function module.transform(pos: Vec.Vec2): Vec.Vec2
	error("Implemented in native code")
end

--- Applies the inverse of the current transformation of `graphics.withTransformation` to a position.
---
--- Use it to turn a screen position, such as the mouse position, into a position in the transformed space. If the
--- transformation cannot be inverted (for example a scale of `0`), it returns `V2(0, 0)`.
---
--- @param pos A position in screen coordinates.
--- @return The same position, in the local space.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
---
--- graphics.withTransformation({ scale = vec.V2(2, 2) }, function()
--- 	local local_ = graphics.inverseTransform(vec.V2(1, 1))
--- 	print(local_.x) -- 0.5
--- end)
--- ```
function module.inverseTransform(pos: Vec.Vec2): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the translation, scale and rotation of the current transformation.
---
--- With nested calls of `graphics.withTransformation`, it returns the combined transformation. Outside of any
--- transformation, it returns `V2(0, 0)`, `V2(1, 1)` and `0`.
---
--- @return The translation, as a `Vec2`.
--- @return The scale, as a `Vec2`.
--- @return The rotation, as an angle in radians.
---
--- # Example
---
--- ```luau
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
---
--- graphics.withTransformation({ scale = vec.V2(2, 3), rotation = 0.5 }, function()
--- 	local translation, scale, rotation = graphics.getCurrentTransform()
--- 	print(scale.x, scale.y, rotation) -- 2 3 0.5
--- end)
--- ```
function module.getCurrentTransform(): (Vec.Vec2, Vec.Vec2, number)
	error("Implemented in native code")
end

return module
