local Vec = require("@vectarine/vec")

--- A module for dealing with coordinate systems and switching between them.
local module = {}

local ScreenVecImpl = { screen_vec = true }
ScreenVecImpl.__index = ScreenVecImpl
export type ScreenVec = typeof(setmetatable({} :: Vec.Vec2Data, ScreenVecImpl))

function ScreenVecImpl.gl(self: ScreenVec): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the position as a Vec2 in the pixel coordinate system
function ScreenVecImpl.px(self: ScreenVec): Vec.Vec2
	error("Implemented in native code")
end

function ScreenVecImpl.scale(self: ScreenVec, k: number): ScreenVec
	error("Implemented in native code")
end

ScreenVecImpl.__add = function(a: ScreenVec, b: ScreenVec): ScreenVec
	error("Implemented in native code")
end

ScreenVecImpl.__sub = function(a: ScreenVec, b: ScreenVec): ScreenVec
	error("Implemented in native code")
end

--------------------------------------

local ScreenPositionImpl = { screen_position = true }
ScreenPositionImpl.__index = ScreenPositionImpl

ScreenPositionImpl.__add = function(a: ScreenPosition, b: ScreenVec): ScreenPosition
	error("Implemented in native code")
end

local sub_function: ((ScreenPosition, ScreenPosition) -> ScreenVec) & ((ScreenPosition, ScreenVec) -> ScreenPosition) = function(
	a: any,
	b: any
): any
	error("Implemented in native code")
end
ScreenPositionImpl.__sub = sub_function

--- Returns the position as a Vec2 in the OpenGL coordinate system
function ScreenPositionImpl.gl(self: ScreenPosition): Vec.Vec2
	error("Implemented in native code")
end

--- Returns the position as a Vec2 in the pixel coordinate system
function ScreenPositionImpl.px(self: ScreenPosition, screen_size: Vec.Vec2?): Vec.Vec2
	error("Implemented in native code")
end

--- Represents somewhere on the screen. This is essentially a Vec2 with a unit.
--- You can use `px`, `gl`, `vh`, or `vw` to create a ScreenPosition from a Vec2 using your preferred coordinate system.
export type ScreenPosition = typeof(setmetatable({} :: Vec.Vec2Data, ScreenPositionImpl))

--- Converts a Vec2 in pixels to a ScreenPosition
--- The pixel convention has the origin (0,0) in the upper-left of the screen, with the lower-right at (screen_width-1, screen_height-1)
--- This is the same convention as most other graphics APIs like HTML5 Canvas, Love2d, Processing, Raylib etc...
---
--- You can use either `coord.px(10, 10)` or `coord.px(Vec.V2(10, 10))` to make ScreenPosition.
--- To perform the conversion, this function uses the screen size of the current drawing target (screen or canvas) unless `screen_size_px` is provided.
function module.px(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenPosition
	error("Implemented in native code")
end

--- Converts a Vec2 in pixels to a ScreenVec
--- The pixel convention has the origin (0,0) in the upper-left of the screen
---
--- You can use either `coord.pxVec(10, 10)` or `coord.pxVec(Vec.V2(10, 10))` to make a ScreenVec.
---
--- To perform the conversion, this function uses the screen size of the current drawing target (screen or canvas) unless `screen_size_px` is provided.
function module.pxVec(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenVec
	error("Implemented in native code")
end

--- Converts a Vec2 using the OpenGL convention to a ScreenPosition
--- The OpenGL convention has the origin (0,0) in the center of the screen, with the bottom-left at (-1,-1) and the upper-right at (1,1)
function module.gl(v_or_x: Vec.Vec2 | number, y_or_nil: number | Vec.Vec2?): ScreenPosition
	error("Implemented in native code")
end

--- Converts a Vec2 using the OpenGL convention to a ScreenVec
function module.glVec(v_or_x: Vec.Vec2 | number, y_or_nil: number | Vec.Vec2?): ScreenVec
	error("Implemented in native code")
end

function module.vw(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenPosition
	error("Implemented in native code")
end

function module.vwVec(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenVec
	error("Implemented in native code")
end

function module.vh(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenPosition
	error("Implemented in native code")
end

function module.vhVec(
	v_or_x: Vec.Vec2 | number,
	screen_size_px_or_y: number | Vec.Vec2?,
	screen_size_px_or_nil: Vec.Vec2?
): ScreenVec
	error("Implemented in native code")
end

module.CENTER = module.gl(Vec.V2(0, 0))
module.TOP_LEFT = module.gl(Vec.V2(-1, 1))
module.TOP_RIGHT = module.gl(Vec.V2(1, 1))
module.BOTTOM_LEFT = module.gl(Vec.V2(-1, -1))
module.BOTTOM_RIGHT = module.gl(Vec.V2(1, -1))

return module
