--[[
2D vectors, used for positions, sizes, directions and speeds.

A `Vec2` holds two numbers, `x` and `y`. Create one with `vec.V2`, or with `vec.fromAngle` when you know a direction.
Combine vectors with `+`, `-`, `*` and `/` (all component by component), and use methods such as `Vec2:scale`,
`Vec2:length` and `Vec2:distance` to move things around and measure them. Because 2D vectors are so common
(your screen is 2D!), this module is called `vec` instead of `vec2`.

When you need to process thousands of vectors at once, see `fastlist`. See `vec4` for 4D vectors and colors.

# Example

```luau
const vec = require("@vectarine/vec")

local position = vec.V2(0, 0)
local velocity = vec.V2(0.5, 0.25)
local target = position + velocity:scale(0.5)
print(target) -- V2(0.25, 0.125)
```

# Common mistakes

- Vectors are shared, not copied: after `local b = a`, `b.x = 1` also changes `a`. Build a new vector with
`vec.V2(a.x, a.y)` or `a:scale(1)` to get an independent copy.
- `v * 2` raises an error, because `*` needs two vectors. Use `Vec2:scale` to multiply by a number. The same goes for
`+`, `-` and `/`.
- Methods described as "in place", such as `Vec2:add`, change the vector they are called on.
Constants such as `vec.ZERO2` cannot be changed, so create a copy with `vec.V2` before modifying one.
]]
local module = {}

local Vec2Impl = { vec2 = true }
Vec2Impl.__index = Vec2Impl
--- Returns the component-wise sum `a + b` as a new vector. Both operands must be vectors, not numbers.
Vec2Impl.__add = function(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end
--- Returns the component-wise difference `a - b` as a new vector. Both operands must be vectors, not numbers.
Vec2Impl.__sub = function(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end
--- Returns the opposite `-a` of the vector, with both components negated.
Vec2Impl.__unm = function(a: Vec2): Vec2
	error("Implemented in native code")
end
--- Returns the component-wise product `a * b` as a new vector. To multiply by a number, use `Vec2:scale`.
Vec2Impl.__mul = function(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end
--- Returns the component-wise quotient `a / b` as a new vector. A zero component in `b` gives `inf` or `nan`.
Vec2Impl.__div = function(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end

--- The fields of a 2D vector, `x` and `y`.
---
--- You do not create this type directly: it describes what `Vec2` contains. Read and write the fields with `v.x`
--- and `v.y`.
export type Vec2Data = { x: number, y: number }

--- A 2D vector with an `x` and a `y` component.
---
--- You get one from `vec.V2` or `vec.fromAngle`, or as the result of a method such as `Vec2:scale`. Use it for
--- positions, sizes and directions, and pass it to drawing functions. Change its components with `v.x = 1`, or
--- with the methods described as "in place", such as `Vec2:add`. Vectors are objects: after `local b = a`, both names refer to the
--- same vector, so changing one changes the other. Operators and most methods return a new vector instead of
--- changing the original.
export type Vec2 = typeof(setmetatable({} :: Vec2Data, Vec2Impl))

--- Creates a 2D vector from its `x` and `y` components.
---
--- On the screen, `x` goes to the right and `y` goes up, and (-1, -1) is the bottom left corner while (1, 1) is
--- the top right corner.
---
--- @param x The horizontal component.
--- @param y The vertical component.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local position = vec.V2(0.5, -0.25)
--- print(position.x) -- 0.5
--- print(position.y) -- -0.25
--- ```
---
--- # See also
---
--- - `vec.fromAngle`: build a vector from a direction.
--- - `Vec2:length`: measure a vector.
function module.V2(x: number, y: number): Vec2
	error("Implemented in native code")
end

--- Creates a vector of a given length that points in the direction of an angle.
---
--- The angle is in radians and is measured from the positive `x` axis. Positive angles turn counterclockwise on the
--- screen. This is the same as `vec.V2(math.cos(angle), math.sin(angle)) * vec.V2(length, length)`.
---
--- @param angle The direction, in radians. 0 points right and `math.pi / 2` points up.
--- @param length The length of the vector. Defaults to 1.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local right = vec.fromAngle(0, 2)
--- print(right) -- V2(2, 0)
--- local shot = vec.fromAngle(math.rad(45), 0.5)
--- print(shot.x > 0 and shot.y > 0) -- true
--- ```
---
--- # See also
---
--- - `Vec2:angle`: get the angle of a vector.
--- - `Vec2:cmul`: rotate a vector.
function module.fromAngle(angle: number, length: number?): Vec2
	error("Implemented in native code")
end

--- Returns the vector multiplied by a number.
---
--- Both components are multiplied by `k`. The original vector is not changed. Use `Vec2:rescale` to change it in
--- place.
---
--- @param k The factor. A negative value flips the direction of the vector.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local velocity = vec.V2(1, 2)
--- local slower = velocity:scale(0.5)
--- print(slower) -- V2(0.5, 1)
--- ```
---
--- # See also
---
--- - `Vec2:rescale`: scale in place.
--- - `Vec2:normalized`: get a vector of length 1.
function Vec2Impl.scale(a: Vec2, k: number): Vec2
	error("Implemented in native code")
end

--- Returns the product of two vectors seen as complex numbers.
---
--- The vector `(x, y)` stands for the complex number `x + y i`, so that
--- `(a.x + i a.y) * (b.x + i b.y) = (a.x*b.x - a.y*b.y) + i(a.x*b.y + a.y*b.x)`.
--- Multiplying by a vector of length 1 rotates `a` by the angle of `b` without changing its length.
---
--- @param b The vector to multiply by.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local a = vec.V2(1, 2)
--- local b = vec.V2(3, 4)
--- print(a:cmul(b)) -- V2(-5, 10)
--- ```
function Vec2Impl.cmul(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns the dot product of the two vectors.
---
--- The dot product is `a.x*b.x + a.y*b.y`. It is positive when the vectors point roughly the same way, zero when
--- they are perpendicular and negative when they point in opposite ways. For vectors of length 1, it is the cosine of
--- the angle between them.
---
--- @param b The other vector.
--- @return The dot product, as a number.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local forward = vec.V2(1, 0)
--- local toEnemy = vec.V2(0.5, 0.5):normalized()
--- print(forward:dot(toEnemy) > 0) -- true, the enemy is in front
--- ```
function Vec2Impl.dot(a: Vec2, b: Vec2): number
	error("Implemented in native code")
end

--- Returns the length of the vector.
---
--- The length is `math.sqrt(v.x*v.x + v.y*v.y)`. If you only need to compare lengths, `Vec2:lengthSq` is faster
--- because it skips the square root.
---
--- @return The length, which is never negative.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local v = vec.V2(3, 4)
--- print(v:length()) -- 5
--- ```
---
--- # See also
---
--- - `Vec2:lengthSq`: compare lengths faster.
--- - `Vec2:distance`: distance between two points.
function Vec2Impl.length(v: Vec2): number
	error("Implemented in native code")
end

--- Returns the square of the length of the vector.
---
--- The result is `v.x*v.x + v.y*v.y`. It avoids a square root, so it is faster than `Vec2:length`. Use it to compare
--- distances: `a:lengthSq() < r * r` is the same test as `a:length() < r`.
---
--- @return The squared length, which is never negative.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local offset = vec.V2(3, 4)
--- print(offset:lengthSq()) -- 25
--- print(offset:lengthSq() < 2 * 2) -- false
--- ```
---
--- # See also
---
--- - `Vec2:length`: the real length.
function Vec2Impl.lengthSq(v: Vec2): number
	error("Implemented in native code")
end

--- Returns the distance between two points.
---
--- This is the same as `(a - b):length()`. Use it with vectors that are positions.
---
--- @param b The other point.
--- @return The distance, which is never negative.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local player = vec.V2(0, 0)
--- local coin = vec.V2(3, 4)
--- print(player:distance(coin)) -- 5
--- ```
---
--- # See also
---
--- - `Vec2:length`: length of one vector.
function Vec2Impl.distance(a: Vec2, b: Vec2): number
	error("Implemented in native code")
end

--- Returns the angle of the vector, in radians.
---
--- The angle is measured from the positive `x` axis and goes from -`math.pi` to `math.pi`. It is the opposite of
--- `vec.fromAngle`. The zero vector gives 0.
---
--- @return The angle, in radians. A vector pointing right gives 0 and a vector pointing up gives `math.pi / 2`.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- print(vec.V2(1, 0):angle()) -- 0
--- local up = vec.V2(0, 1)
--- print(math.floor(up:angle() * 100) / 100) -- 1.57
--- ```
---
--- # See also
---
--- - `vec.fromAngle`: the opposite conversion.
function Vec2Impl.angle(v: Vec2): number
	error("Implemented in native code")
end

--- Returns the vector scaled to a length of 1.
---
--- The direction stays the same. This is useful to get a direction from a difference of positions, then scale it to
--- a speed.
---
--- @return A new vector of length 1, or the zero vector if the vector has zero length.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local toTarget = vec.V2(0, 2)
--- local step = toTarget:normalized():scale(0.5)
--- print(step) -- V2(0, 0.5)
--- print(vec.ZERO2:normalized()) -- V2(0, 0)
--- ```
---
--- # See also
---
--- - `Vec2:length`: measure a vector.
--- - `Vec2:scale`: change the length.
function Vec2Impl.normalized(v: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns the vector with the absolute value of each component.
---
--- @return A new vector with non-negative components.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local v = vec.V2(-2, 3)
--- print(v:abs()) -- V2(2, 3)
--- ```
function Vec2Impl.abs(v: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns the vector with each component rounded to a number of decimal digits.
---
--- With 0 digits, components are rounded to the nearest integer. With 2 digits, 0.126 becomes 0.13.
---
--- @param digits_of_precision The number of digits kept after the decimal point. Defaults to 0.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local v = vec.V2(1.26, 2.5)
--- print(v:round()) -- V2(1, 3)
--- print(v:round(1)) -- V2(1.3, 2.5)
--- ```
function Vec2Impl.round(v: Vec2, digits_of_precision: number?): Vec2
	error("Implemented in native code")
end

--- Returns the vector with each component rounded down to the nearest integer.
---
--- @return A new vector. Negative numbers go further from zero: -1.5 becomes -2.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local v = vec.V2(2.7, -1.5)
--- print(v:floor()) -- V2(2, -2)
--- ```
function Vec2Impl.floor(v: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns the vector with each component rounded up to the nearest integer.
---
--- @return A new vector. Negative numbers go closer to zero: -1.5 becomes -1.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local v = vec.V2(2.1, -1.5)
--- print(v:ceil()) -- V2(3, -1)
--- ```
function Vec2Impl.ceil(v: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns a vector made of the larger component of each pair.
---
--- The result has `math.max(a.x, b.x)` as `x` and `math.max(a.y, b.y)` as `y`.
---
--- @param b The other vector.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local a = vec.V2(1, 5)
--- local b = vec.V2(3, 2)
--- print(a:max(b)) -- V2(3, 5)
--- ```
---
--- # See also
---
--- - `Vec2:min`: the lower bound.
function Vec2Impl.max(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns a vector made of the smaller component of each pair.
---
--- The result has `math.min(a.x, b.x)` as `x` and `math.min(a.y, b.y)` as `y`. Combined with `Vec2:max`, it keeps a
--- position inside an area.
---
--- @param b The other vector.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local player = vec.V2(1.5, -0.5)
--- local clamped = player:min(vec.V2(1, 1)):max(vec.V2(-1, -1))
--- print(clamped) -- V2(1, -0.5)
--- ```
---
--- # See also
---
--- - `Vec2:max`: the upper bound.
function Vec2Impl.min(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns the point between two vectors at a given fraction of the way.
---
--- The result is `a + (b - a) * k`. It is often used to move something smoothly toward a target.
---
--- @param b The vector reached when `k` is 1.
--- @param k The fraction of the way from `a` to `b`. 0 gives `a`, 1 gives `b`. The value is not limited to the
--- range 0 to 1: other values go past the ends.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local start = vec.V2(0, 0)
--- local goal = vec.V2(1, 2)
--- print(start:lerp(goal, 0.25)) -- V2(0.25, 0.5)
--- ```
---
--- # See also
---
--- - `Vec2:min`: limit a position.
--- - `Vec2:distance`: distance to the target.
function Vec2Impl.lerp(a: Vec2, b: Vec2, k: number): Vec2
	error("Implemented in native code")
end

--- Returns the sign of each component of the vector.
---
--- @return A new vector where each component is 1 if it was positive, -1 if it was negative and 0 if it was zero.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local v = vec.V2(-0.3, 4)
--- print(v:sign()) -- V2(-1, 1)
--- ```
function Vec2Impl.sign(v: Vec2): Vec2
	error("Implemented in native code")
end

--- Returns the simplex noise value at the point `v`.
---
--- Simplex noise is smooth random-looking noise: nearby points give similar values. It is useful for terrain,
--- clouds or wobbling movements. The noise is the same on every run.
--- See [simplex noise](https://en.wikipedia.org/wiki/OpenSimplex_noise).
---
--- @return The noise value at this point.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local height = vec.V2(3.2, 1.7):noise()
--- print(height)
--- ```
function Vec2Impl.noise(v: Vec2): number
	error("Implemented in native code")
end

--- Returns the Worley noise value at the point `v`.
---
--- Worley noise is based on the distance to the nearest of many scattered points. It looks like cells or stones.
--- The noise is the same on every run. See [Worley noise](https://en.wikipedia.org/wiki/Worley_noise).
---
--- @return The noise value at this point.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local cell = vec.V2(3.2, 1.7):worleyNoise()
--- print(cell)
--- ```
function Vec2Impl.worleyNoise(v: Vec2): number
	error("Implemented in native code")
end

--- Adds a vector to this one, changing this vector in place.
---
--- Unlike `+`, this does not create a new vector, which avoids extra work when done every frame. It returns `a`
--- itself, so calls can be chained.
---
--- @param b The vector to add.
--- @return The vector `a`, after the change.
---
--- # Errors
---
--- Raises an error if `a` is a constant such as `vec.ZERO2`.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local position = vec.V2(0, 0)
--- position:add(vec.V2(0.5, 0.25))
--- print(position) -- V2(0.5, 0.25)
--- ```
---
--- # See also
---
--- - `Vec2:sub`: subtract in place.
--- - `Vec2:rescale`: multiply by a number in place.
function Vec2Impl.add(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end

--- Subtracts a vector from this one, changing this vector in place.
---
--- It returns `a` itself, so calls can be chained. See `Vec2:add`.
---
--- @param b The vector to subtract.
--- @return The vector `a`, after the change.
---
--- # Errors
---
--- Raises an error if `a` is a constant such as `vec.ZERO2`.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local position = vec.V2(1, 1)
--- position:sub(vec.V2(0.25, 0.5))
--- print(position) -- V2(0.75, 0.5)
--- ```
function Vec2Impl.sub(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end

--- Multiplies this vector by another one component by component, changing this vector in place.
---
--- It returns `a` itself, so calls can be chained. To multiply by a number, use `Vec2:rescale`.
---
--- @param b The vector to multiply by.
--- @return The vector `a`, after the change.
---
--- # Errors
---
--- Raises an error if `a` is a constant such as `vec.ZERO2`.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local size = vec.V2(2, 3)
--- size:mul(vec.V2(0.5, 2))
--- print(size) -- V2(1, 6)
--- ```
function Vec2Impl.mul(a: Vec2, b: Vec2): Vec2
	error("Implemented in native code")
end

--- Multiplies this vector by a number, changing this vector in place.
---
--- It returns `a` itself, so calls can be chained. Use `Vec2:scale` to get a new vector instead.
---
--- @param k The factor to multiply both components by.
--- @return The vector `a`, after the change.
---
--- # Errors
---
--- Raises an error if `a` is a constant such as `vec.ZERO2`.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
---
--- local velocity = vec.V2(1, 2)
--- velocity:rescale(0.5)
--- print(velocity) -- V2(0.5, 1)
--- ```
---
--- # See also
---
--- - `Vec2:scale`: get a new vector instead.
function Vec2Impl.rescale(a: Vec2, k: number): Vec2
	error("Implemented in native code")
end

--- The zero vector, `V2(0, 0)`. It is a constant: changing it or calling an in-place method such as `Vec2:add` on it
--- raises an error.
module.ZERO2 = module.V2(0, 0)

return module
