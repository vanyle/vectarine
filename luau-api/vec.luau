--- Module for 2D vectors.
--- Because 2D vectors are so common (your screen is 2D!), this module is called `vec` instead of `vec2`.
--- See `vec4` for 3D and 4D vectors.
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

--- Create a new 2d vector of length `length` pointing towards the given angle (in radians)
--- If `length` is not provided, it defaults to 1
--- This is the same as: `V2(math.cos(angle), math.sin(angle)) * length`
function module.fromAngle(angle: number, length: number?): Vec2
	return nil :: any
end

--- Scales the vector by the given factor
function Vec2Impl.scale(a: Vec2, k: number): Vec2
	error("Not implemented")
end

--- Performs the complex multiplication of a and b, treating them as complex numbers
--- `(a.x + i a.y) * (b.x + i b.y) = (a.x*b.x - a.y*b.y) + i(a.x*b.y + a.y*b.x)`
function Vec2Impl.cmul(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- Returns the dot product of the two vectors
function Vec2Impl.dot(a: Vec2, b: Vec2): number
	error("Not implemented")
end

--- Returns the length of the vector, defined as `math.sqrt(v.x*v.x + v.y*v.y)`
function Vec2Impl.length(v: Vec2): number
	error("Not implemented")
end

--- Returns the square of the length of the vector, defined as `v.x*v.x + v.y*v.y`
function Vec2Impl.lengthSq(v: Vec2): number
	error("Not implemented")
end

--- Returns the distance between the two vectors.
--- This is the same as `(a - b):length()`
function Vec2Impl.distance(a: Vec2, b: Vec2): number
	error("Not implemented")
end

--- Returns the angle (in radians) of the vector
function Vec2Impl.angle(v: Vec2): number
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

--- Returns a vector with all components rounded down to the nearest integer
function Vec2Impl.floor(v: Vec2): Vec2
	error("Not implemented")
end

--- Returns a vector with all components rounded up to the nearest integer
function Vec2Impl.ceil(v: Vec2): Vec2
	error("Not implemented")
end

--- Returns the component-wise maximum of the two vectors
function Vec2Impl.max(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- Returns the component-wise minimum of the two vectors
function Vec2Impl.min(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- Returns a vector linearly interpolated between `a` and `b` by the factor `k`
function Vec2Impl.lerp(a: Vec2, b: Vec2, k: number): Vec2
	error("Not implemented")
end

--- Returns the vector containing the sign of each component of the vector
function Vec2Impl.sign(v: Vec2): Vec2
	error("Not implemented")
end

--- Returns the noise value at the given point
--- Uses [simplex noise](https://en.wikipedia.org/wiki/OpenSimplex_noise)
function Vec2Impl.noise(v: Vec2): number
	error("Not implemented")
end

--- Returns the noise value at the given point
--- Uses [Worley noise](https://en.wikipedia.org/wiki/Worley_noise)
function Vec2Impl.worleyNoise(v: Vec2): number
	error("Not implemented")
end

--- Adds `b` to `a`, modifying `a` in-place
function Vec2Impl.add(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- Subtracts `b` from `a`, modifying `a` in-place. Returns `a`.
function Vec2Impl.sub(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- Multiplies `a` by `b`, modifying `a` in-place. Returns `a`.
function Vec2Impl.mul(a: Vec2, b: Vec2): Vec2
	error("Not implemented")
end

--- Multiplies `a` by `k`, modifying `a` in-place. Returns `a`.
function Vec2Impl.rescale(a: Vec2, k: number): Vec2
	error("Not implemented")
end

--- The zero vector
module.ZERO2 = module.V2(0, 0)

return module
