local Camera2 = require("@vectarine/camera")
local Vec = require("@vectarine/vec")
local module = {}

local World2Impl = { type = "physicsworld2" }
World2Impl.__index = World2Impl

type WorldData = {
	--- A camera to show debugging information like colliders and make objects clickable and draggable
	camera: Camera2.Camera2?,
	--- The gravity of the world that will be applied to all objects
	--- Set to 0 if you want to control the gravity separately for each object
	gravity: Vec.Vec2,
}

export type World2 = typeof(setmetatable({} :: WorldData, World2Impl))

--- Create a new physics world
--- A world is a collection of objects that can interact with each other
--- You can optionally pass a camera to show debugging information like colliders and make objects clickable and draggable
--- @return World2
function module.newWorld2(gravity: Vec.Vec2?, camera: Camera2.Camera2?): World2
	error("Not implemented")
end

--- Remove an object from the world
--- @param object Object
function World2Impl:removeObject(object: Object2)
	error("Not implemented")
end

--- Step the physics simulation forward in time
--- If you don't call this, nothing will move!
--- @param delta_time number
function World2Impl:step(delta_time: number)
	error("Not implemented")
end

--- Get all objects containing the given tags. Returns all objects if no tags are given.
--- @param tags string[]?
--- @return Object2[]
function World2Impl:getObjects(tags: { string }?): { Object2 }
	error("Not implemented")
end

--- Get all objects at a given point
--- @param point Vec2
--- @return Object2[]
function World2Impl:getObjectsAtPoint(point: Vec.Vec2): { Object2 }
	error("Not implemented")
end

--MARK: Collider

--- Represents a 2d collider. Used for collisions
local Collider2Impl = { type = "collider2" }
Collider2Impl.__index = Collider2Impl
export type Collider2 = typeof(setmetatable({}, Collider2Impl))

--- Create a rectangle collider, this is a shorthand for newPolygonShape as rectangles are very common
--- @param size Vec2
--- @return Collider2
function module.newRectangleCollider(size: Vec.Vec2): Collider2
	error("Not implemented")
end

--MARK: Object2

local Object2Impl = { type = "object2" }
Object2Impl.__index = Object2Impl
type Object2Type = {
	position: Vec.Vec2,
	speed: Vec.Vec2,
	rotation: number,
	rotationSpeed: number,
	tags: { string },
	extra: any,
}
export type Object2 = typeof(setmetatable({} :: Object2Type, Object2Impl))

--- Return the points of the collider in the object space
--- @return Vec2[]
function Object2Impl:getPoints(): { Vec.Vec2 }
	error("Not implemented")
end

--- Create a new object into the world. Objects are specific to a given world.
--- You can give the object tags to find it later
--- Objects can have 3 body types:
--- - static: objects that don't move and don't react to collisions (like the ground)
--- - dynamic: objects that move and react to collisions
--- - kinematic: objects that can be manually moved with code but don't react to collisions (like moving platforms or elevators)
--- @param position Vec2
--- @param mass number Must be greater than 0
--- @param collider Collider
--- @param tags string[]
--- @param bodyType "static" | "dynamic" | "kinematic"
--- @return Object2
function World2Impl:createObject(
	position: Vec.Vec2,
	mass: number,
	collider: Collider2,
	tags: { string },
	bodyType: "static" | "dynamic" | "kinematic"
): Object2
	error("Not implemented")
end

return module
