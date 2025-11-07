local module = {}

local Vec2Impl = { vec2 = true }
Vec2Impl.__index = Vec2Impl
Vec2Impl.__add = function(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end
Vec2Impl.__sub = function(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end
Vec2Impl.__mul = function(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- The type of 2d things
export type Vec2Data = { x: number, y: number }

--- Implementation of a 2D vector
export type Vec2 = typeof(setmetatable({} :: Vec2Data, Vec2Impl))

--- Create a new 2d vector
function module.V2(x: number, y: number): Vec2
	return nil :: any
end

function Vec2Impl.scale(a: Vec2, k: number): Vec2
	error("Not implemented")
end

--- Performs the complex multiplication of a and b, treating them as complex numbers
--- `(a.x + i a.y) * (b.x + i b.y) = (a.x*b.x - a.y*b.y) + i(a.x*b.y + a.y*b.x)`
function Vec2Impl.cmul(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- Returns `math.sqrt(v.x*v.x + v.y*v.y)`
function Vec2Impl.length(v: Vec2): number
	error("Not implemented")
end

--- Returns a normalized version of the vector
--- If the vector has zero length, returns a unit vector pointing in an arbitrary direction
function Vec2Impl.normalized(v: Vec2): Vec2
	error("Not implemented")
end

--- Returns a version of the vector rounded to the given number of digits of precision
---
--- If `digits_of_precision` is not provided, it defaults to 0 (round to nearest integer)
function Vec2Impl.round(v: Vec2, digits_of_precision: number?): Vec2
	error("Not implemented")
end

module.ZERO2 = module.V2(0, 0)

return module
