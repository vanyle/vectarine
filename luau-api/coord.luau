local Vec = require("@vectarine/vec")

--- A module for dealing with coordinate systems and switching between them.
local module = {}

local ScreenDeltaImpl = {screen_delta=true}
ScreenDeltaImpl.__index = ScreenDeltaImpl
export type ScreenVec = typeof(setmetatable({} :: Vec.Vec2Data, ScreenDeltaImpl))

local ScreenPositionImpl = {screen_position=true}
ScreenPositionImpl.__index = ScreenPositionImpl

ScreenPositionImpl.__add = function(a: ScreenPosition, b: ScreenVec): ScreenPosition
    error("Not implemented")
end

ScreenPositionImpl.__sub = function<T>(a: ScreenPosition, b: ScreenPosition | ScreenVec): ScreenPosition | ScreenVec
    error("Not implemented")
end

--- Returns the position as a Vec2 in the OpenGL coordinate system
function ScreenPositionImpl.gl(self: ScreenPosition): Vec.Vec2
    error("Not implemented")
end

--- Returns the position as a Vec2 in the pixel coordinate system
function ScreenPositionImpl.px(self: ScreenPosition): Vec.Vec2
    error("Not implemented")
end

--- Represents somewhere on the screen. This is essentially a Vec2 with a unit.
--- You can use `px`, `gl`, `vh`, or `vw` to create a ScreenPosition from a Vec2 using your preferred coordinate system.
export type ScreenPosition = typeof(setmetatable({} :: Vec.Vec2Data, ScreenPositionImpl))


--- Converts a Vec2 in pixels to a ScreenPosition
--- The pixel convention has the origin (0,0) in the upper-left of the screen, with the lower-right at (screen_width-1, screen_height-1)
--- This is the same convention as most other graphics APIs like HTML5 Canvas, Love2d, Processing, Raylib etc...
function module.px(x: number, y: number): ScreenPosition
    error("Not implemented")
end

--- Converts a Vec2 in pixels to a ScreenDelta
--- The pixel convention has the origin (0,0) in the upper-left of the screen
function module.pxDelta(x: number, y: number): ScreenVec
    error("Not implemented")
end

--- Converts a Vec2 using the OpenGL convention to a ScreenPosition
--- The OpenGL convention has the origin (0,0) in the center of the screen, with the bottom-left at (-1,-1) and the upper-right at (1,1)
function module.gl(x: number, y: number): ScreenPosition
    error("Not implemented")
end

--- Converts a Vec2 using the OpenGL convention to a ScreenDelta
function module.glDelta(x: number, y: number): ScreenVec
    error("Not implemented")
end

function module.vw(x: number, y: number): ScreenPosition
    error("Not implemented")
end

--- Converts a Vec2 using the OpenGL convention to a ScreenDelta
function module.vwDelta(x: number, y: number): ScreenVec
    error("Not implemented")
end

function module.vh(x: number, y: number): ScreenPosition
    error("Not implemented")
end

function module.vhDelta(x: number, y: number): ScreenVec
    error("Not implemented")
end

return module
