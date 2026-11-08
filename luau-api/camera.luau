--[[
Converts between world positions and screen positions, with a movable, rotating and zoomable camera.

Create a camera with `camera.new`, then change its `position`, `rotation` and `zoom` to look at another part of
your world. Use `Camera2:screen` to find where a world point appears on the screen and `Camera2:world` for the other
direction, for example to know what is under the mouse. `Camera2:moveTowards` makes the camera follow a target smoothly. For drawing a whole group of things with a transformation, see
`graphics.withTransformation`.

# Example

```luau
const camera = require("@vectarine/camera")
const graphics = require("@vectarine/graphics")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const cam = camera.new()
local player = vec.V2(3, 1)

function Update(deltaTime: number)
	cam:moveTowards(player, 0.1)
	graphics.drawCircle(cam:screen(player), 0.05, vec4.RED)
end
```

# Common mistakes

- Draw positions are screen positions. Convert world positions with `Camera2:screen` before drawing, or the camera has
  no effect.
- The `amount` of `Camera2:moveTowards` is a fraction per call, not a speed, so the movement depends on the frame
  rate unless you compute `amount` from `deltaTime`.
]]
local module = {}

local Fastlist = require("@vectarine/fastlist")
local Vec = require("@vectarine/vec")

local Camera2Impl = { type = "camera", position = Vec.ZERO2, rotation = 0, zoom = 1 }
Camera2Impl.__index = Camera2Impl

--[[
A Camera2 is a transformation between the game world and the screen.

You get one from `camera.new`. A camera has these fields that you can read and write:

- `position`: the `Vec2` of the world that is at the center of the screen. The camera shares this vector: changing
  its `x` or `y` moves the camera, and so does changing the vector you assigned to it.
- `rotation`: the rotation of the camera, in radians.
- `zoom`: how much the world is enlarged. `1` is the default, `2` shows everything twice as big, and `0.5` twice as small.

At zoom `1`, one world unit is half the width of the screen, and a square in the world stays a square on the screen
whatever the shape of the window.
]]
export type Camera2 = typeof(setmetatable({} :: typeof(Camera2Impl), Camera2Impl))

--- Creates a new 2D camera.
---
--- The camera starts at position `V2(0, 0)`, with a rotation of `0` and a zoom of `1`. At that point, world and screen
--- coordinates are the same horizontally.
---
--- @return A new camera.
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
--- const vec = require("@vectarine/vec")
---
--- local cam = camera.new()
--- cam.zoom = 2
--- cam.position = vec.V2(5, 0)
--- ```
function module.new(): Camera2
	error("Implemented in native code")
end

--- Returns the screen position of a point of the world.
---
--- The result is in screen coordinates, where `(-1, -1)` is the bottom left and `(1, 1)` the top right of the screen. It can
--- be outside of this range if the point is not visible. Use `Camera2:isVisible` to test it. To convert many points at
--- once, use `Camera2:screenFastlist`.
---
--- @param point A position in the world.
--- @return The position of this point on the screen.
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- const cam = camera.new()
---
--- function Update(deltaTime: number)
--- 	graphics.drawCircle(cam:screen(vec.V2(2, 1)), 0.05, vec4.BLUE)
--- end
--- ```
---
--- # See also
---
--- - `Camera2:world`: the opposite conversion.
--- - `Camera2:isVisible`: tests if a point is on the screen.
function Camera2Impl:screen(point: Vec.Vec2): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the screen positions of many world points at once.
---
--- It does the same as `Camera2:screen` for every point of the list, and returns a new list. It is faster than calling
--- `Camera2:screen` in a loop when there are many points.
---
--- @param point The world positions to convert.
--- @return A new list with the screen position of each point.
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
--- const fastlist = require("@vectarine/fastlist")
--- const vec = require("@vectarine/vec")
---
--- const cam = camera.new()
--- local worldPoints = fastlist.fromTable({ vec.V2(0, 0), vec.V2(1, 1) })
--- local screenPoints = cam:screenFastlist(worldPoints)
--- ```
function Camera2Impl:screenFastlist(point: Fastlist.FastList): Fastlist.FastList
	error("Implemented in native code")
end

--- Returns the world position of a point of the screen.
---
--- It is the opposite of `Camera2:screen`. Use it to find what the mouse points at in your world. To convert many points
--- at once, use `Camera2:worldFastlist`.
---
--- @param point A position on the screen, in screen coordinates.
--- @return The position of this point in the world.
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
--- const io = require("@vectarine/io")
---
--- const cam = camera.new()
---
--- function Update(deltaTime: number)
--- 	local underMouse = cam:world(io.getMouse())
--- 	print(underMouse.x, underMouse.y)
--- end
--- ```
---
--- # See also
---
--- - `Camera2:screen`: the opposite conversion.
--- - `io.getMouse`: gives the mouse position.
function Camera2Impl:world(point: Vec.Vec2): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the world positions of many screen points at once.
---
--- It does the same as `Camera2:world` for every point of the list, and returns a new list.
---
--- @param point The screen positions to convert.
--- @return A new list with the world position of each point.
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
--- const fastlist = require("@vectarine/fastlist")
--- const vec = require("@vectarine/vec")
---
--- const cam = camera.new()
--- local corners = fastlist.fromTable({ vec.V2(-1, -1), vec.V2(1, 1) })
--- local worldCorners = cam:worldFastlist(corners)
--- ```
function Camera2Impl:worldFastlist(point: Fastlist.FastList): Fastlist.FastList
	error("Implemented in native code")
end

--- Returns `true` if a point of the world is inside the screen.
---
--- A point exactly on the edge of the screen counts as visible. Use it to skip drawing things that are out of view.
---
--- @param point A position in the world.
--- @return True if the point is on the screen.
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
--- const vec = require("@vectarine/vec")
---
--- const cam = camera.new()
--- print(cam:isVisible(vec.V2(0, 0))) -- true
--- print(cam:isVisible(vec.V2(50, 0))) -- false
--- ```
function Camera2Impl:isVisible(point: Vec.Vec2): boolean
	error("Implemented in native code")
end

--- Moves the camera a fraction of the way towards a point.
---
--- Each call moves the camera by `amount` times the remaining distance to `point`. Called every frame with a small
--- amount, like `0.01` or `0.1`, it makes the camera follow a target smoothly. The result depends on the frame rate. It is
--- close to `cam.position = cam.position + (point - cam.position):scale(amount)`, except that the vector in
--- `cam.position` is changed in place rather than replaced. To get the same speed at any frame rate, compute `amount`
--- from `deltaTime`, as in the example.
---
--- @param point The position of the world to move towards.
--- @param amount The fraction of the distance to cover, between `0` (no movement) and `1` (arrive immediately).
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
--- const vec = require("@vectarine/vec")
---
--- const cam = camera.new()
--- local player = vec.V2(4, 2)
---
--- function Update(deltaTime: number)
--- 	-- Frame rate independent: covers about 99% of the distance in one second.
--- 	cam:moveTowards(player, 1 - math.exp(-5 * deltaTime))
--- end
--- ```
function Camera2Impl:moveTowards(point: Vec.Vec2, amount: number): ()
	error("Implemented in native code")
end

--- Rotates the camera a fraction of the way towards an angle.
---
--- Each call turns the camera by `amount` times the difference between `angle` and the current rotation. Called every
--- frame with a small amount, like `0.01` or `0.1`, it makes the rotation smooth. To get the same speed at any frame rate,
--- compute `amount` from `deltaTime`, as in the example. Angles are not wrapped, so the camera
--- does not take the shortest way around the circle.
---
--- @param angle The rotation to reach, in radians.
--- @param amount The fraction of the difference to cover, between `0` (no change) and `1` (arrive immediately).
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
---
--- const cam = camera.new()
---
--- function Update(deltaTime: number)
--- 	cam:rotateTowards(math.pi / 2, 1 - math.exp(-5 * deltaTime))
--- end
--- ```
function Camera2Impl:rotateTowards(angle: number, amount: number): ()
	error("Implemented in native code")
end

--- Changes the zoom of the camera a fraction of the way towards a zoom level.
---
--- Each call changes the zoom by `amount` times the difference between `zoom` and the current zoom. Called every frame
--- with a small amount, like `0.01` or `0.1`, it makes the zoom smooth. To get the same speed at any frame rate, compute
--- `amount` from `deltaTime`, as in the example.
---
--- @param zoom The zoom level to reach. `1` is the default zoom.
--- @param amount The fraction of the difference to cover, between `0` (no change) and `1` (arrive immediately).
---
--- # Example
---
--- ```luau
--- const camera = require("@vectarine/camera")
---
--- const cam = camera.new()
---
--- function Update(deltaTime: number)
--- 	cam:zoomTowards(2, 1 - math.exp(-5 * deltaTime))
--- end
--- ```
function Camera2Impl:zoomTowards(zoom: number, amount: number): ()
	error("Implemented in native code")
end

return module
