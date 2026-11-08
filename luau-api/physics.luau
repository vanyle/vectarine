local Camera2 = require("@vectarine/camera")
local Tile = require("@vectarine/tile")
local Vec = require("@vectarine/vec")

--- Provides a 2D physics engine: gravity, collisions, bouncing and joints.
---
--- This API is experimental and can change.
---
--- Create a `World2` with `physics.newWorld2`, then add objects with `World2:createObject`. Each object has a
--- collider (its shape, made with one of the collider functions of this module, for example
--- `physics.newCircleCollider`) and a body type: `"static"` objects never move,
--- `"dynamic"` objects are moved by gravity and collisions, and `"kinematic"` objects move only when you set their
--- `Object2.speed`. Call `World2:step` once per frame to advance the simulation, then read `Object2.position` and
--- draw the objects yourself with `graphics`.
---
--- Positions and sizes are in the units you choose, as long as you use them everywhere. The physics engine does not
--- draw anything and does not use screen coordinates by itself, so the example below uses values that fit
--- on screen (screen coordinates go from -1 to 1). Rectangle colliders are described by their half-extents: half of
--- their width and half of their height. For positions and vectors, see `vec`.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local world = physics.newWorld2(vec.V2(0, -1))
--- -- Sizes are half-extents: this ground is 2 wide and 0.1 tall, and the box is 0.2 by 0.2.
--- local ground = world:createObject(vec.V2(0, -0.8), 1, physics.newRectangleCollider(vec.V2(1, 0.05)), { "ground" }, "static")
--- local box = world:createObject(vec.V2(0, 0.5), 1, physics.newRectangleCollider(vec.V2(0.1, 0.1)), { "box" }, "dynamic")
---
--- function Update(deltaTime: number)
--- 	world:step(deltaTime)
--- 	graphics.drawRect(ground.position - vec.V2(1, 0.05), vec.V2(2, 0.1), vec4.GRAY)
--- 	graphics.drawRect(box.position - vec.V2(0.1, 0.1), vec.V2(0.2, 0.2), vec4.RED)
--- end
--- ```
---
--- # Common mistakes
---
--- - Rectangle colliders take half of the width and height, so `vec.V2(1, 0.5)` makes a rectangle 2 wide and 1 tall.
--- - Nothing moves until `World2:step` is called, and it must be called every frame with the `deltaTime` of `Update`.
--- - After `World2:removeObject`, an object can no longer be used: most of its fields and methods raise an error.
--- - Changing `Object2.position` teleports the object. Use `Object2.speed` to move it with the physics.
local module = {}

local World2Impl = { type = "physicsworld2" }
World2Impl.__index = World2Impl

--- The fields of a `World2`.
type WorldData = {
	--- The camera of the world, or `nil` if there is none.
	---
	--- It can be set when creating the world or changed later. It is currently only stored and returned: the physics
	--- engine does not use it to draw or to move objects.
	camera: Camera2.Camera2?,
	--- The acceleration applied to every dynamic object, per second. For example `(0, -10)` pulls objects down.
	---
	--- Use `(0, 0)` to control the gravity of each object yourself. Assigning a new vector replaces the gravity, the
	--- new value is used from the next `World2:step`.
	gravity: Vec.Vec2,
}

--- A physics simulation containing objects, colliders and joints.
---
--- You get one from `physics.newWorld2`. Add objects with `World2:createObject`, advance it with `World2:step` and
--- query it with functions like `World2:getObjects` or `World2:getObjectsAtPoint`. Objects of different worlds never
--- interact.
export type World2 = typeof(setmetatable({} :: WorldData, World2Impl))

--- Creates an empty physics world.
---
--- A world is a collection of objects that can interact with each other. Nothing moves until you call
--- `World2:step`. The gravity and the camera can be changed later through the `gravity` and `camera` fields.
---
--- @param gravity The gravity applied to dynamic objects. Defaults to `(0, 0)`, which means no gravity. The vector is
--- kept, not copied: changing it later changes the gravity of the world.
--- @param camera A camera stored in the world. Defaults to no camera. Raises an error if the value is not a `Camera2`.
--- @return A new world.
---
--- # Errors
---
--- Raises an error if `camera` is not `nil` and not a camera.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- print(world.gravity.y) -- -10
--- ```
---
--- # See also
---
--- - `World2:createObject`: adds objects to the world.
--- - `World2:step`: runs the simulation.
function module.newWorld2(gravity: Vec.Vec2?, camera: Camera2.Camera2?): World2
	error("Implemented in native code")
end

--- Removes an object from the world.
---
--- Also removes all joints attached to the object. Afterwards, the object can no longer be used, see `Object2`.
--- Removing an object twice does nothing.
---
--- @param object The object to remove.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local coin = world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.1), { "coin" }, "dynamic")
--- world:removeObject(coin)
--- print(#world:getObjects()) -- 0
--- ```
function World2Impl:removeObject(object: Object2)
	error("Implemented in native code")
end

--- Advances the physics simulation by the given amount of time.
---
--- Call it once per frame. If you never call it, nothing moves. This applies gravity, moves objects, resolves
--- collisions and updates `Object2.position`, `Object2.speed` and `Object2.rotation`. Before simulating, the position
--- and speed vectors of every object are read, so changing them in place (for example `box.position.x = 3`) works.
--- The simulation runs a single step of the length you give: very large values make objects tunnel through each
--- other, so avoid passing long frame times.
---
--- @param deltaTime The time to simulate, in seconds. Usually the `deltaTime` given to `Update`.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
---
--- function Update(deltaTime: number)
--- 	world:step(deltaTime)
--- end
--- ```
---
--- # See also
---
--- - `physics.newWorld2`: creates a world.
--- - `Object2:getContacts`: finds the objects touching an object.
function World2Impl:step(deltaTime: number)
	error("Implemented in native code")
end

--- Returns the objects of the world that have all the given tags.
---
--- The order of the returned objects is not defined. The result is a new table each time.
---
--- @param tags The tags to look for. Defaults to no tags, which returns every object. An object must have all the
--- listed tags to be returned.
--- @return A list of objects.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2()
--- world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.1), { "enemy", "flying" }, "dynamic")
--- world:createObject(vec.V2(1, 0), 1, physics.newCircleCollider(0.1), { "enemy" }, "dynamic")
--- print(#world:getObjects({ "enemy" })) -- 2
--- print(#world:getObjects({ "enemy", "flying" })) -- 1
--- print(#world:getObjects()) -- 2
--- ```
function World2Impl:getObjects(tags: { string }?): { Object2 }
	error("Implemented in native code")
end

--- Returns the objects whose collider contains the given point.
---
--- Most of the time, there will be 0 or 1 object at a given point. It is useful to find what the player clicked on.
---
--- @param point The point to test, in world coordinates.
--- @return A list of objects, empty if the point is in the void.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2()
--- local crate = world:createObject(vec.V2(0, 0), 1, physics.newRectangleCollider(vec.V2(1, 1)), { "crate" }, "static")
--- print(#world:getObjectsAtPoint(vec.V2(0.5, 0.5))) -- 1
--- ```
function World2Impl:getObjectsAtPoint(point: Vec.Vec2): { Object2 }
	error("Implemented in native code")
end

--- Returns the objects whose collider touches a rectangular area.
---
--- The area extends from `position` to `position + size`.
---
--- @param position The bottom left corner of the area, in world coordinates.
--- @param size The width and height of the area.
--- @return A list of objects, empty if the area is free.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2()
--- world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.5), { "bomb" }, "static")
--- local nearby = world:getObjectsInArea(vec.V2(-1, -1), vec.V2(2, 2))
--- print(#nearby) -- 1
--- ```
function World2Impl:getObjectsInArea(position: Vec.Vec2, size: Vec.Vec2): { Object2 }
	error("Implemented in native code")
end

--- Returns the objects hit by a ray, with the time at which the ray reaches each of them.
---
--- A ray starts at `position` and moves by `direction` every unit of time. The direction does not have to be
--- normalized and represents the "velocity" of the ray. The time of impact is the time the ray takes to reach the
--- object. For a direction of length 1, this is the distance. The list is not sorted. If the ray starts inside an
--- object, the time of impact for that object is 0.
---
--- @param position The start of the ray, in world coordinates.
--- @param direction The direction and speed of the ray.
--- @param maxTimeOfImpact The maximum time the ray travels. Defaults to 10000. Setting a smaller value makes the
--- query faster and ignores far objects.
--- @return A list of tables with two fields: `object` is the object that was hit and `timeOfImpact` is the time at which
--- the ray reaches it.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2()
--- world:createObject(vec.V2(3, 0), 1, physics.newCircleCollider(1), { "wall" }, "static")
--- local hits = world:getObjectsIntersectingRay(vec.V2(0, 0), vec.V2(1, 0), 10)
--- print(#hits, hits[1].timeOfImpact) -- 1	2
--- ```
function World2Impl:getObjectsIntersectingRay(
	position: Vec.Vec2,
	direction: Vec.Vec2,
	maxTimeOfImpact: number?
): { { object: Object2, timeOfImpact: number } }
	error("Implemented in native code")
end

--- Returns all the joints of the world.
---
--- @return A list of joints, empty if there are none.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
---
--- local world = physics.newWorld2()
--- print(#world:getJoints()) -- 0
--- ```
function World2Impl:getJoints(): { Joint2 }
	error("Implemented in native code")
end

--MARK: Collider

local Collider2Impl = { type = "collider2" }
Collider2Impl.__index = Collider2Impl
--- The shape of an object, used to detect collisions.
---
--- You get one from the collider functions of this module, for example `physics.newCircleCollider`, and pass it to
--- `World2:createObject`. A collider has no position: it is placed at the
--- position of the object. The collider is copied when creating an object, so one collider can be reused for several
--- objects.
export type Collider2 = typeof(setmetatable({}, Collider2Impl))

--- Creates a rectangular collider centered on the object.
---
--- Rectangles are very common, so this is cheaper than `physics.newPolygonCollider`.
---
--- @param size Half of the width and half of the height of the rectangle. A `size` of `(1, 0.5)` makes a rectangle
--- that is 2 wide and 1 tall.
--- @return A new collider.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2()
--- local wall = world:createObject(vec.V2(0, 0), 1, physics.newRectangleCollider(vec.V2(1, 0.5)), { "wall" }, "static")
--- print(#wall:getPoints()) -- 4
--- ```
function module.newRectangleCollider(size: Vec.Vec2): Collider2
	error("Implemented in native code")
end

--- Creates a circular collider centered on the object.
---
--- @param radius The radius of the circle.
--- @return A new collider.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local ball = world:createObject(vec.V2(0, 1), 1, physics.newCircleCollider(0.25), { "ball" }, "dynamic")
--- ball:setRestitution(0.8)
--- ```
function module.newCircleCollider(radius: number): Collider2
	error("Implemented in native code")
end

--- Creates a collider from the outline of a polygon.
---
--- The outline goes through the points in order and is closed by a line from the last point back to the first one.
--- Only the outline collides: the inside of the shape is not solid, so a small object can sit inside it. Polygon
--- colliders are more expensive than circle or rectangle colliders. The list of points must not be empty.
---
--- @param points The corners of the polygon, in the space of the object (relative to its position).
--- @return A new collider.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local triangle = physics.newPolygonCollider({ vec.V2(-1, 0), vec.V2(1, 0), vec.V2(0, 1) })
--- local world = physics.newWorld2()
--- local ramp = world:createObject(vec.V2(0, 0), 1, triangle, { "ramp" }, "static")
--- ```
function module.newPolygonCollider(points: { Vec.Vec2 }): Collider2
	error("Implemented in native code")
end

--- Creates a collider made of the filled tiles of a tilemap.
---
--- A voxel collider is an efficient representation of a 2D grid of square colliders. It is useful for building levels.
--- The tiles whose tile ID is in `filledTileIds` become solid, all the other tiles are empty. You can split your terrain
--- in parts and use multiple voxel colliders to have infinite terrain if needed.
---
--- You can create multiple voxel colliders on the same region with different `filledTileIds` to have different objects
--- react differently to entities (for example, walls block the player, spikes hurt them). If no tile is filled in the
--- region, the collider is empty and never collides.
---
--- @param voxelSize The size of one voxel (one tile) in world units.
--- @param voxelsData The tilemap holding the tile IDs: a `TilemapResource` or a `GeneratedTilemap`.
--- @param layer The layer of the tilemap to read.
--- @param low The lowest coordinate of the region, in tiles. Each component is rounded down.
--- @param high The highest coordinate of the region, in tiles. Each component is rounded up.
--- @param filledTileIds The tile IDs that are solid.
--- @return A new collider.
---
--- # Errors
---
--- Raises an error if the tilemap resource is not loaded.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const tile = require("@vectarine/tile")
--- const vec = require("@vectarine/vec")
---
--- local function buildWalls(world: physics.World2, level: tile.TilemapResource)
--- 	local walls = physics.newVoxelCollider(vec.V2(1, 1), level, 0, vec.V2(0, 0), vec.V2(32, 18), { 1, 2 })
--- 	world:createObject(vec.V2(0, 0), 1, walls, { "wall" }, "static")
--- end
--- ```
function module.newVoxelCollider(
	voxelSize: Vec.Vec2,
	voxelsData: Tile.TilemapResource | Tile.GeneratedTilemap,
	layer: number,
	low: Vec.Vec2,
	high: Vec.Vec2,
	filledTileIds: { number }
): Collider2
	error("Implemented in native code")
end

--MARK: Object2

local Object2Impl = { type = "object2" }
Object2Impl.__index = Object2Impl
--- The fields of an `Object2`.
type Object2Type = {
	--- The center of the object, in world coordinates.
	---
	--- It is updated by `World2:step`. Assigning a new vector moves the object there immediately. Changing the vector
	--- in place (`object.position.x = 3`) also works and is applied at the next `World2:step`. The object keeps the
	--- vector you give, so the same vector is updated as the object moves.
	---
	--- After `World2:removeObject`, reading it returns the last vector and assigning it raises an error.
	position: Vec.Vec2,
	--- The velocity of the object, in world units per second.
	---
	--- It is updated by `World2:step`. Like `Object2.position`, it can be assigned or changed in place. Kinematic
	--- objects move at this speed without being affected by forces.
	---
	--- After `World2:removeObject`, reading it returns the last vector and assigning it raises an error.
	speed: Vec.Vec2,
	--- How much the object slows down while moving, even when alone. 0 means no slowdown.
	---
	--- Values above 1 make the object move as if in a fluid. This is different from the friction set with
	--- `Object2:setFriction`, which only acts when objects touch each other.
	---
	--- Raises an error after `World2:removeObject`.
	linearDamping: number,
	--- The angle of the object, in radians.
	---
	--- Raises an error after `World2:removeObject`.
	rotation: number,
	--- How fast the object turns, in radians per second.
	---
	--- Raises an error after `World2:removeObject`.
	rotationSpeed: number,
	--- How much the object slows down its rotation. 0 means no slowdown.
	---
	--- Raises an error after `World2:removeObject`.
	angularDamping: number,
	--- The tags of the object, used by `World2:getObjects` to find it.
	---
	--- The list given to `World2:createObject` is kept, not copied. Assigning a new list replaces the tags.
	---
	--- Raises an error after `World2:removeObject`.
	tags: { string },
	--- Any extra data you want to store on the object, like its health or its type. It is `nil` by default.
	---
	--- Raises an error after `World2:removeObject`.
	extra: any,
}
--- A body in a `World2`: something with a position, a speed and a collider.
---
--- You get one from `World2:createObject`, or from queries like `World2:getObjects`. The same object always gives
--- the same Lua value, so you can compare objects with `==` and use them as table keys. Use its fields to read and
--- change it, and its methods to configure how it reacts to collisions. After `World2:removeObject`, the object is
--- out of this world: each field and method says what it does then.
export type Object2 = typeof(setmetatable({} :: Object2Type, Object2Impl))

--- Returns the points of the outline of the collider, in world coordinates.
---
--- Use it to draw the outline of the object, for example with `graphics.drawLine`. A rectangle gives its 4 corners
--- and a polygon gives its points. If the collider is not a polygon (for example a circle), a close approximation is returned: a circle gives
--- 32 points. For tilemap colliders, this returns 4 points per filled tile, representing the corners of the tile.
--- The points follow the position and rotation of the object at the time of the call.
---
--- @return A new list of points.
---
--- # Errors
---
--- Raises an error if the object was removed from its world.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local world = physics.newWorld2(vec.V2(0, -1))
--- local box = world:createObject(vec.V2(0, 0), 1, physics.newRectangleCollider(vec.V2(0.1, 0.1)), { "box" }, "dynamic")
---
--- function Update(deltaTime: number)
--- 	world:step(deltaTime)
--- 	local points = box:getPoints()
--- 	for i = 1, #points do
--- 		local from = points[i]
--- 		local to = points[i % #points + 1]
--- 		graphics.drawLine(from, to, vec4.BLUE)
--- 	end
--- end
--- ```
function Object2Impl:getPoints(): { Vec.Vec2 }
	error("Implemented in native code")
end

--- Returns the objects that are touching this object.
---
--- Use it to react to collisions, for example to know if the player stands on the ground. Call it after
--- `World2:step`.
---
--- @return A list of objects.
---
--- # Errors
---
--- Raises an error if the object was removed from its world.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local player = world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.5), { "player" }, "dynamic")
--- world:step(1 / 60)
--- for _, other in player:getContacts() do
--- 	if other ~= player and #other.tags > 0 then
--- 		print("Touching " .. other.tags[1])
--- 	end
--- end
--- ```
function Object2Impl:getContacts(): { Object2 }
	error("Implemented in native code")
end

--- Sets the mass of the object.
---
--- Heavier objects are harder to push. This changes the mass of the collider of the object.
---
--- @param mass The new mass. It must be greater than 0.
---
--- # Errors
---
--- Raises an error if the object was removed from its world.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local crate = world:createObject(vec.V2(0, 0), 1, physics.newRectangleCollider(vec.V2(0.5, 0.5)), { "crate" }, "dynamic")
--- crate:setMass(20)
--- ```
function Object2Impl:setMass(mass: number)
	error("Implemented in native code")
end

--- Prevents an object from rotating, or allows it again.
---
--- Objects can rotate by default. A locked object can still move around. Useful for characters that should not fall
--- over.
---
--- @param lock `true` to prevent rotation, `false` to allow it.
---
--- # Errors
---
--- Raises an error if the object was removed from its world.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local player = world:createObject(vec.V2(0, 0), 1, physics.newRectangleCollider(vec.V2(0.2, 0.4)), { "player" }, "dynamic")
--- player:setLockRotation(true)
--- ```
function Object2Impl:setLockRotation(lock: boolean)
	error("Implemented in native code")
end

--- Prevents an object from moving, or allows it again.
---
--- Objects can move by default. A locked object can still rotate. Useful for rotating platforms that should not move.
---
--- @param lock `true` to prevent movement, `false` to allow it.
---
--- # Errors
---
--- Raises an error if the object was removed from its world.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local fan = world:createObject(vec.V2(0, 0), 1, physics.newRectangleCollider(vec.V2(1, 0.1)), { "fan" }, "dynamic")
--- fan:setLockTranslation(true)
--- fan.rotationSpeed = 3
--- ```
function Object2Impl:setLockTranslation(lock: boolean)
	error("Implemented in native code")
end

--- Sets how much the object bounces.
---
--- Higher values make objects bounce more. 0 means the object does not bounce, and 1 means it keeps all its energy.
---
--- @param restitution The new restitution.
---
--- # Errors
---
--- Raises an error if the object was removed from its world.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local ball = world:createObject(vec.V2(0, 2), 1, physics.newCircleCollider(0.2), { "ball" }, "dynamic")
--- ball:setRestitution(0.9)
--- ```
function Object2Impl:setRestitution(restitution: number)
	error("Implemented in native code")
end

--- Sets how much the object resists sliding on the objects it touches.
---
--- Friction hinders the movement of objects in contact. This is different from `Object2.linearDamping`, which exists
--- even when an object is alone in a scene. Higher values make objects slide less. The default friction is 0.5.
---
--- @param friction The new friction. 0 means no friction.
---
--- # Errors
---
--- Raises an error if the object was removed from its world.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local ice = world:createObject(vec.V2(0, 0), 1, physics.newRectangleCollider(vec.V2(2, 0.1)), { "ice" }, "static")
--- ice:setFriction(0)
--- ```
function Object2Impl:setFriction(friction: number)
	error("Implemented in native code")
end

--- Creates an object in the world and returns it.
---
--- Objects belong to the world that created them. The tags let you find the object later with `World2:getObjects`.
--- There are 3 body types:
---
--- - `"static"`: the object never moves (like the ground).
--- - `"dynamic"`: the object moves and reacts to collisions (player, enemies, pushable crates, etc.).
--- - `"kinematic"`: the object moves only when you change its `Object2.speed`, and other objects cannot push it
--- (like moving platforms or elevators).
---
--- @param position The initial position of the object, which is the center of its collider. The vector is copied.
--- @param mass The mass of the object, added to the mass computed from the collider shape. It must be greater than 0.
--- @param collider The shape of the object, from one of the `physics.new...Collider` functions.
--- @param tags A list of tags, for example `{ "enemy", "flying" }`, used to find the object later. Can be empty.
--- @param bodyType One of `"static"`, `"dynamic"` or `"kinematic"`.
--- @return The new object.
---
--- # Errors
---
--- Raises an error if `bodyType` is not one of the three allowed values.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local crate = world:createObject(vec.V2(0, 1), 1, physics.newRectangleCollider(vec.V2(0.5, 0.5)), { "crate" }, "dynamic")
--- print(crate.position.y) -- 1
--- ```
---
--- # See also
---
--- - `physics.newRectangleCollider`: makes a rectangle shape.
--- - `World2:removeObject`: deletes the object.
function World2Impl:createObject(
	position: Vec.Vec2,
	mass: number,
	collider: Collider2,
	tags: { string },
	bodyType: "static" | "dynamic" | "kinematic"
): Object2
	error("Implemented in native code")
end

--MARK: Joint2

local Joint2Impl = { type = "joint2" }
Joint2Impl.__index = Joint2Impl
--- The fields of a `Joint2`. Joints have no fields.
type Joint2Type = {}
--- A link between two objects of a world.
---
--- You get one from `World2:createDistanceJoint`, and you use it to find the linked objects or to destroy the link.
--- Joints are also destroyed when one of their objects is removed with `World2:removeObject`.
export type Joint2 = typeof(setmetatable({} :: Joint2Type, Joint2Impl))

--- Creates a joint that pins two objects together at their centers.
---
--- The two objects can rotate around each other but their centers are kept at the same point. Despite its name,
--- this does not keep a fixed distance between the objects: both centers are pulled together. Place the objects at the
--- same position when creating the joint.
---
--- @param object1 The first object.
--- @param object2 The second object.
--- @return The new joint.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2(vec.V2(0, -10))
--- local anchor = world:createObject(vec.V2(0, 2), 1, physics.newCircleCollider(0.1), { "anchor" }, "static")
--- local weight = world:createObject(vec.V2(0, 2), 1, physics.newCircleCollider(0.3), { "weight" }, "dynamic")
--- local joint = world:createDistanceJoint(anchor, weight)
--- print(#world:getJoints()) -- 1
--- ```
function World2Impl:createDistanceJoint(object1: Object2, object2: Object2): Joint2
	error("Implemented in native code")
end

--- Returns the first object of the joint.
---
--- @return The first object, as passed to `World2:createDistanceJoint`.
---
--- # Errors
---
--- Raises an error ("Joint is invalid") if the joint was removed, if one of its objects was removed, or if its world
--- no longer exists.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2()
--- local a = world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.1), { "a" }, "dynamic")
--- local b = world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.1), { "b" }, "dynamic")
--- local joint = world:createDistanceJoint(a, b)
--- print(joint:getObject1() == a) -- true
--- ```
function Joint2Impl:getObject1(): Object2
	error("Implemented in native code")
end

--- Returns the second object of the joint.
---
--- @return The second object, as passed to `World2:createDistanceJoint`.
---
--- # Errors
---
--- Raises an error ("Joint is invalid") if the joint was removed, if one of its objects was removed, or if its world
--- no longer exists.
function Joint2Impl:getObject2(): Object2
	error("Implemented in native code")
end

--- Removes the joint from the world, destroying it.
---
--- The objects are kept and stop being linked. Does nothing if the joint was already removed or if the world no longer
--- exists.
---
--- # Example
---
--- ```luau
--- const physics = require("@vectarine/physics")
--- const vec = require("@vectarine/vec")
---
--- local world = physics.newWorld2()
--- local a = world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.1), { "a" }, "dynamic")
--- local b = world:createObject(vec.V2(0, 0), 1, physics.newCircleCollider(0.1), { "b" }, "dynamic")
--- local joint = world:createDistanceJoint(a, b)
--- joint:remove()
--- print(#world:getJoints()) -- 0
--- ```
function Joint2Impl:remove()
	error("Implemented in native code")
end

return module
