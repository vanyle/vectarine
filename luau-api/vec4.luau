--- Module for 4D vectors.
--- 4D vectors can represent colors and 3D rotations.
--- You can use them for 3D by ignoring the last component.
local module = {}

local Vec4Impl = { vec4 = true }
Vec4Impl.__index = Vec4Impl
Vec4Impl.__add = function(a: Vec4, b: Vec4): Vec4
	error("Not implemented")
end
Vec4Impl.__sub = function(a: Vec4, b: Vec4): Vec4
	error("Not implemented")
end
Vec4Impl.__mul = function(a: Vec4, b: Vec4): Vec4
	error("Not implemented")
end

--- The type of 4D things
export type Vec4Data = { x: number, y: number, z: number, w: number, r: number, g: number, b: number, a: number }

--- Implementation of a 4D vector
export type Vec4 = typeof(setmetatable({} :: Vec4Data, Vec4Impl))

--- Create a new 4D vector
function module.V4(x: number, y: number, z: number, w: number): Vec4
	return nil :: any
end

--- Scales the vector by the given factor
function Vec4Impl.scale(a: Vec4, k: number): Vec4
	error("Not implemented")
end

--- Returns the dot product of the two vectors
function Vec4Impl.dot(a: Vec4, b: Vec4): number
	error("Not implemented")
end

--- Returns the length of the vector, defined as `math.sqrt(v.x*v.x + v.y*v.y + v.z*v.z + v.w*v.w)`
function Vec4Impl.length(v: Vec4): number
	error("Not implemented")
end

--- Returns the square of the length of the vector, defined as `v.x*v.x + v.y*v.y + v.z*v.z + v.w*v.w`
function Vec4Impl.lengthSq(v: Vec4): number
	error("Not implemented")
end

--- Returns the distance between the two vectors.
--- This is the same as `(a - b):length()`
function Vec4Impl.distance(a: Vec4, b: Vec4): number
	error("Not implemented")
end

--- Returns a normalized version of the vector
--- If the vector has zero length, returns a unit vector pointing in an arbitrary direction
function Vec4Impl.normalized(v: Vec4): Vec4
	error("Not implemented")
end

--- Returns a version of the vector rounded to the given number of digits of precision
---
--- If `digits_of_precision` is not provided, it defaults to 0 (round to nearest integer)
function Vec4Impl.round(v: Vec4, digits_of_precision: number?): Vec4
	error("Not implemented")
end

--- Returns a vector with all components rounded down to the nearest integer
function Vec4Impl.floor(v: Vec4): Vec4
	error("Not implemented")
end

--- Returns a vector with all components rounded up to the nearest integer
function Vec4Impl.ceil(v: Vec4): Vec4
	error("Not implemented")
end

--- Returns the component-wise maximum of the two vectors
function Vec4Impl.max(a: Vec4, b: Vec4): Vec4
	error("Not implemented")
end

--- Returns the component-wise minimum of the two vectors
function Vec4Impl.min(a: Vec4, b: Vec4): Vec4
	error("Not implemented")
end

--- Returns a vector linearly interpolated between `a` and `b` by the factor `k`
function Vec4Impl.lerp(a: Vec4, b: Vec4, k: number): Vec4
	error("Not implemented")
end

--- Returns the vector containing the sign of each component of the vector
function Vec4Impl.sign(v: Vec4): Vec4
	error("Not implemented")
end

--- The zero vector
module.ZERO4 = module.V4(0, 0, 0, 0)

--MARK: Colors

--- Alias for V4, but indicate that the Vec4 will be used as a color.
function module.createColor(r: number, g: number, b: number, a: number): Vec4
	error("Not implemented")
end

module.BLACK = module.createColor(0, 0, 0, 1)
module.WHITE = module.createColor(1, 1, 1, 1)
module.TRANSPARENT = module.createColor(0, 0, 0, 0)

module.RED = module.createColor(1, 0, 0, 1)
module.GREEN = module.createColor(0, 1, 0, 1)
module.BLUE = module.createColor(0, 0, 1, 1)

module.YELLOW = module.createColor(1, 1, 0, 1)
module.MAGENTA = module.createColor(1, 0, 1, 1)
module.CYAN = module.createColor(0, 1, 1, 1)

module.AZURE = module.createColor(0, 0.5, 1, 1)
module.ORANGE = module.createColor(1, 0.5, 0, 1)
module.PURPLE = module.createColor(0.5, 0, 1, 1)
module.SPRING = module.createColor(0, 1, 0.5, 1)
module.LIME = module.createColor(0.5, 1, 0, 1)
module.PINK = module.createColor(1, 0, 0.5, 1)

module.LIGHT_GRAY = module.createColor(0.7, 0.7, 0.7, 1)
module.DARK_GRAY = module.createColor(0.3, 0.3, 0.3, 1)

module.GRAY = module.createColor(0.5, 0.5, 0.5, 1)
module.DARK_RED = module.createColor(0.5, 0, 0, 1)
module.DARK_GREEN = module.createColor(0, 0.5, 0, 1)
module.DARK_BLUE = module.createColor(0, 0, 0.5, 1)

return module
