local Vec = require("@vectarine/vec")

--- A module for dealing with coordinate systems and switching between them.
local module = {}

local ScreenVecImpl = { screen_vec = true }
ScreenVecImpl.__index = ScreenVecImpl
export type ScreenVec = typeof(setmetatable({} :: Vec.Vec2Data, ScreenVecImpl))

function ScreenVecImpl.gl(self: ScreenVec): Vec.Vec2
	error("Not implemented")
end

--- Returns the position as a Vec2 in the pixel coordinate system
function ScreenVecImpl.px(self: ScreenVec): Vec.Vec2
	error("Not implemented")
end

function ScreenVecImpl.scale(self: ScreenVec, k: number): ScreenVec
	error("Not implemented")
end

--------------------------------------

local ScreenPositionImpl = { screen_position = true }
ScreenPositionImpl.__index = ScreenPositionImpl

ScreenPositionImpl.__add = function(a: ScreenPosition, b: ScreenVec): ScreenPosition
	error("Not implemented")
end

local sub_function: ((ScreenPosition, ScreenPosition) -> ScreenVec) & ((ScreenPosition, ScreenVec) -> ScreenPosition) = function(
	a: any,
	b: any
): any
	error("Not implemented")
end
ScreenPositionImpl.__sub = sub_function

--- Returns the position as a Vec2 in the OpenGL coordinate system
function ScreenPositionImpl.gl(self: ScreenPosition): Vec.Vec2
	error("Not implemented")
end

--- Returns the position as a Vec2 in the pixel coordinate system
function ScreenPositionImpl.px(self: ScreenPosition, screen_size: Vec.Vec2?): Vec.Vec2
	error("Not implemented")
end

--- Represents somewhere on the screen. This is essentially a Vec2 with a unit.
--- You can use `px`, `gl`, `vh`, or `vw` to create a ScreenPosition from a Vec2 using your preferred coordinate system.
export type ScreenPosition = typeof(setmetatable({} :: Vec.Vec2Data, ScreenPositionImpl))

--- Converts a Vec2 in pixels to a ScreenPosition
--- The pixel convention has the origin (0,0) in the upper-left of the screen, with the lower-right at (screen_width-1, screen_height-1)
--- This is the same convention as most other graphics APIs like HTML5 Canvas, Love2d, Processing, Raylib etc...
---
--- To perform the conversion, this function uses the screen size of the current drawing target (screen or canvas) unless `screen_size_px` is provided.
function module.px(v: Vec.Vec2, screen_size_px: Vec.Vec2?): ScreenPosition
	error("Not implemented")
end

--- Converts a Vec2 in pixels to a ScreenVec
--- The pixel convention has the origin (0,0) in the upper-left of the screen
---
--- To perform the conversion, this function uses the screen size of the current drawing target (screen or canvas) unless `screen_size_px` is provided.
function module.pxVec(v: Vec.Vec2, screen_size_px: Vec.Vec2?): ScreenVec
	error("Not implemented")
end

--- Converts a Vec2 using the OpenGL convention to a ScreenPosition
--- The OpenGL convention has the origin (0,0) in the center of the screen, with the bottom-left at (-1,-1) and the upper-right at (1,1)
function module.gl(v: Vec.Vec2): ScreenPosition
	error("Not implemented")
end

--- Converts a Vec2 using the OpenGL convention to a ScreenVec
function module.glVec(v: Vec.Vec2): ScreenVec
	error("Not implemented")
end

function module.vw(v: Vec.Vec2, screen_size_px: Vec.Vec2?): ScreenPosition
	error("Not implemented")
end

function module.vwVec(v: Vec.Vec2, screen_size_px: Vec.Vec2?): ScreenVec
	error("Not implemented")
end

function module.vh(v: Vec.Vec2, screen_size_px: Vec.Vec2?): ScreenPosition
	error("Not implemented")
end

function module.vhVec(v: Vec.Vec2, screen_size_px: Vec.Vec2?): ScreenVec
	error("Not implemented")
end

module.CENTER = module.gl(Vec.V2(0, 0))
module.TOP_LEFT = module.gl(Vec.V2(-1, 1))
module.TOP_RIGHT = module.gl(Vec.V2(1, 1))
module.BOTTOM_LEFT = module.gl(Vec.V2(-1, -1))
module.BOTTOM_RIGHT = module.gl(Vec.V2(1, -1))

return module
