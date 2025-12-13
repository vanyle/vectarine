local Camera2 = require("@vectarine/camera")
local Vec = require("@vectarine/vec")
local module = {}

local PhysicsWorld2Impl = { type = "physicsworld2", objects = {} }
PhysicsWorld2Impl.__index = PhysicsWorld2Impl
export type PhysicsWorld2 = typeof(setmetatable({}, PhysicsWorld2Impl))

--- Create a new physics world
--- A physics world is a collection of objects
--- You can optionally pass a camera to show debugging information like colliders and make objects clickable and draggable
--- @return PhysicsWorld2
function module.newWorld(camera: Camera2.Camera2?): PhysicsWorld2
	error("Not implemented")
end

--MARK: Shape

--- Represents a 2d shape. Used for collisions
local ShapeImpl = { type = "shape" }
ShapeImpl.__index = ShapeImpl
export type Shape = typeof(setmetatable({}, ShapeImpl))

--- Create a polygon shape
--- @param points Vec2[]
--- @return Shape
function module.newPolygonShape(points: { Vec.Vec2 }): Shape
	error("Not implemented")
end

--- Create a rectangle shape, this is a shorthand for newPolygonShape as rectangles are very common
--- @param size Vec2
--- @return Shape
function module.newRectangleShape(size: Vec.Vec2): Shape
	error("Not implemented")
end

--- Create a circle shape
--- @param radius number
--- @return Shape
function module.newCircleShape(radius: number): Shape
	error("Not implemented")
end

--- Get the area of the shape
--- @return number
function ShapeImpl:area(): number
	error("Not implemented")
end

--- Get the center of gravity of the shape
--- Usually, you want this to be zero
--- @return Vec.Vec2
function ShapeImpl:centerOfGravity(): Vec.Vec2
	error("Not implemented")
end

--MARK: Object

local ObjectImpl = {
	type = "object",
	position = Vec.ZERO2,
	rotation = 0,
	scale = Vec.ZERO2,
	speed = Vec.ZERO2,
	shape = nil,
	tags = {},
}
ObjectImpl.__index = ObjectImpl
export type Object = typeof(setmetatable({}, ObjectImpl))

--- Create a new object
--- @param position Vec2
--- @param shape Shape
--- @param tags string[]
--- @return Object
function module.createObject(
	position: Vec.Vec2,
	shape: Shape,
	tags: { string },
	bodyType: "static" | "dynamic" | "kinematic"
): Object
	error("Not implemented")
end

return module
