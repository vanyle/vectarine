local module = {}

local Fastlist = require("@vectarine/fastlist")
local Vec = require("@vectarine/vec")

local Camera2Impl = { type = "camera", position = Vec.ZERO2, rotation = 0, zoom = 1 }
Camera2Impl.__index = Camera2Impl
export type Camera2 = typeof(setmetatable({}, Camera2Impl))

--- Create a new camera for 2D
--- A camera represents a transformation between the game world and the screen
--- @return Camera
function module.new(): Camera2
	error("Not implemented")
end

--- Get the screen position of a point in the world
--- @param point Vec2
--- @return Vec2
function Camera2Impl:screen(point: Vec.Vec2): Vec.Vec2
	error("Not implemented")
end

--- The fastlist version of Camera:screen()
--- @param point FastList
--- @return FastList
function Camera2Impl:screenFastlist(point: Fastlist.FastList): Fastlist.FastList
	error("Not implemented")
end

--- Get the world position of a point on the screen
--- @param point Vec2 | FastList
--- @return Vec2 | FastList
function Camera2Impl:world(point: Vec.Vec2): Vec.Vec2
	error("Not implemented")
end

--- The fastlist version of Camera:world()
--- @param point FastList
--- @return FastList
function Camera2Impl:worldFastlist(point: Fastlist.FastList): Fastlist.FastList
	error("Not implemented")
end

--- Check if a point is visible on the screen
--- @param point Vec2
--- @return boolean
function Camera2Impl:isVisible(point: Vec.Vec2): boolean
	error("Not implemented")
end

--- Move the camera towards a point by a certain amount.
--- If called every frame, you should keep amount small, like 0.01
--- This is the same as: `Camera.position = Camera.position + (point - Camera.position):scale(amount)`
--- @param point Vec2
--- @param amount number
function Camera2Impl:moveTowards(point: Vec.Vec2, amount: number): ()
	error("Not implemented")
end

--- Rotate the camera towards an angle by a certain amount.
--- If called every frame, you should keep amount small, like 0.01
--- @param angle number
--- @param amount number
function Camera2Impl:rotateTowards(angle: number, amount: number): ()
	error("Not implemented")
end

--- Zoom the camera towards a zoom level by a certain amount.
--- If called every frame, you should keep amount small, like 0.01
--- @param zoom number
--- @param amount number
function Camera2Impl:zoomTowards(zoom: number, amount: number): ()
	error("Not implemented")
end

return module
