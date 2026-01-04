local Camera2 = require("@vectarine/camera")
local Vec = require("@vectarine/vec")
--- A Physics API for 2D
---
--- This API is experimental and can change.
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
function module.newWorld2(gravity: Vec.Vec2?, camera: Camera2.Camera2?): World2
	error("Not implemented")
end

--- Remove an object from the world
--- Also removes all joints attached to the object
function World2Impl:removeObject(object: Object2)
	error("Not implemented")
end

--- Step the physics simulation forward in time
--- If you don't call this, nothing will move!
function World2Impl:step(delta_time: number)
	error("Not implemented")
end

--- Get all objects containing the given tags. Returns all objects if no tags are given.
function World2Impl:getObjects(tags: { string }?): { Object2 }
	error("Not implemented")
end

--- Get all objects at a given point
--- Most of the time, there will only be 0 or 1 object at a given point
function World2Impl:getObjectsAtPoint(point: Vec.Vec2): { Object2 }
	error("Not implemented")
end

--- Get all objects in a given rectangular area
function World2Impl:getObjectsInArea(position: Vec.Vec2, size: Vec.Vec2): { Object2 }
	error("Not implemented")
end

--- Get all objects intersecting a ray. The direction does not have to be normalized and represents the "velocity" of the ray.
--- For performance reasons, you should set a maximum time of impact which is the maximum time the ray will travel at its speed.
--- The time of impact is the time the ray will travel to reach the object. For normalized directions, this is the distance.
function World2Impl:getObjectsIntersectingRay(
	position: Vec.Vec2,
	direction: Vec.Vec2,
	max_time_of_impact: number?
): { { object: Object2, timeOfImpact: number } }
	error("Not implemented")
end

--- Get all joints in the world
--- @return { Joint2 }
function World2Impl:getJoints(): { Joint2 }
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

--- Create a circle collider
--- @param radius number
--- @return Collider2
function module.newCircleCollider(radius: number): Collider2
	error("Not implemented")
end

--- Create a polygon collider
--- Polygon colliders are more expensive than circle or rectangle colliders
--- @param points Vec2[]
--- @return Collider2
function module.newPolygonCollider(points: { Vec.Vec2 }): Collider2
	error("Not implemented")
end

--MARK: Object2

local Object2Impl = { type = "object2" }
Object2Impl.__index = Object2Impl
type Object2Type = {
	position: Vec.Vec2,
	speed: Vec.Vec2,
	-- How much friction the object has. 0 is no friction, 1> like moving in a fluid.
	linearDamping: number,
	rotation: number,
	rotationSpeed: number,
	--- How much the object resists to rotation. 0 is no resistance
	angularDamping: number,
	tags: { string },
	extra: any,
}
export type Object2 = typeof(setmetatable({} :: Object2Type, Object2Impl))

--- Return the points of the collider in the object space
--- If the collider is not a polygon (for example a circle), return a close approximation.
--- @return { Vec2 }
function Object2Impl:getPoints(): { Vec.Vec2 }
	error("Not implemented")
end

--- Returns all objects that are touching this object
function Object2Impl:getContacts(): { Object2 }
	error("Not implemented")
end

--- Set the mass of the object
--- @param mass number Must be greater than 0
function Object2Impl:setMass(mass: number)
	error("Not implemented")
end

--- Prevents an object from rotating (objects can rotate by default)
--- It can still move around. Useful for characters that shouldn't fall over.
function Object2Impl:setLockRotation(lock: boolean)
	error("Not implemented")
end

--- Prevents an object from moving (objects can move by default)
--- It can still rotate. Useful for rotating platforms that shouldn't move.
function Object2Impl:setLockTranslation(lock: boolean)
	error("Not implemented")
end

--- Set the restitution of the object. Higher values make objects bounce more.
function Object2Impl:setRestitution(restitution: number)
	error("Not implemented")
end

--- Set the friction of the object. Higher values make objects slide less.
--- Defaults to 0.5
function Object2Impl:setFriction(friction: number)
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
--- @param tags { string }
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

--MARK: Joint2

local Joint2Impl = { type = "object2" }
Joint2Impl.__index = Joint2Impl
type Joint2Type = {}
export type Joint2 = typeof(setmetatable({} :: Joint2Type, Joint2Impl))

--- Create a distance joint between two objects
--- Once the joint is created, the distance between the objects will be fixed.
--- This can be used to create pendulums, or wheels.
--- @param object1 Object2
--- @param object2 Object2
--- @return Joint2
function World2Impl:createDistanceJoint(object1: Object2, object2: Object2): Joint2
	error("Not implemented")
end

--- Get the first object of the joint.
--- Throws an error if the joint is invalid.
--- @return Object2
function Joint2Impl:getObject1(): Object2
	error("Not implemented")
end

--- Get the second object of the joint
--- Throws an error if the joint is invalid.
--- @return Object2
function Joint2Impl:getObject2(): Object2
	error("Not implemented")
end

--- Remove the joint from the world, destroying it.
function Joint2Impl:remove()
	error("Not implemented")
end

return module
