--- Provides 4D vectors, used mostly to represent colors and rotations.
---
--- A `Vec4` has four numbers: `x`, `y`, `z` and `w`. The same numbers are also available as `r`, `g`, `b` and `a`,
--- which is how colors are stored: red, green, blue and alpha (opacity), each between 0 and 1. Functions that draw
--- things, like the ones of `graphics`, take colors as `Vec4`. This module defines the common colors, like `vec4.RED`.
---
--- `Vec4` can also be used as a quaternion to rotate things in 3D (see `vec4.fromRotation` and `Vec4:qmul`), or as a
--- plain 4D vector. For 2D positions and sizes, use `vec` instead.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local sky = vec4.createColor(0.2, 0.4, 0.9, 1)
--- local dusk = sky:lerp(vec4.ORANGE, 0.5)
--- print(dusk:round(2)) -- V4(0.6, 0.45, 0.45, 1)
--- ```
---
--- # Common mistakes
---
--- - Colors go from 0 to 1, not from 0 to 255: divide by 255 if you start from values like `(200, 120, 40)`.
--- - Vectors from constants like `vec4.RED` cannot be modified. Create a copy with `vec4.createColor` to change one.
--- - `*` multiplies component by component. To scale a whole vector by a number, use `Vec4:scale`.
local module = {}

local Vec4Impl = { vec4 = true }
Vec4Impl.__index = Vec4Impl
--- Adds two vectors component by component: `a + b`.
Vec4Impl.__add = function(a: Vec4, b: Vec4): Vec4
	error("Implemented in native code")
end
--- Subtracts two vectors component by component: `a - b`.
Vec4Impl.__sub = function(a: Vec4, b: Vec4): Vec4
	error("Implemented in native code")
end
--- Negates every component of the vector: `-a`.
Vec4Impl.__unm = function(a: Vec4): Vec4
	error("Implemented in native code")
end
--- Multiplies two vectors component by component: `a * b`. This is not a dot product, see `Vec4:dot`.
Vec4Impl.__mul = function(a: Vec4, b: Vec4): Vec4
	error("Implemented in native code")
end
--- Divides two vectors component by component: `a / b`. Dividing by a zero component gives `inf` or `nan`.
Vec4Impl.__div = function(a: Vec4, b: Vec4): Vec4
	error("Implemented in native code")
end

--- The fields of a `Vec4`.
---
--- `r`, `g`, `b` and `a` are other names for `x`, `y`, `z` and `w`: changing one changes the other.
export type Vec4Data = { x: number, y: number, z: number, w: number, r: number, g: number, b: number, a: number }

--- A vector of four numbers, used for colors and rotations.
---
--- You get one from a constructor like `vec4.V4` or `vec4.createColor`, from a constant like `vec4.RED`, or as the
--- result of a method such as `Vec4:lerp`. You can add (`+`), subtract (`-`) and negate (`-v`) vectors. Multiplying
--- (`*`) and dividing (`/`) two vectors works component by component. To multiply by a number, use `Vec4:scale`.
---
--- The vectors stored in constants like `vec4.RED` cannot be modified.
export type Vec4 = typeof(setmetatable({} :: Vec4Data, Vec4Impl))

--- Creates a 4D vector from its four components.
---
--- Use `vec4.createColor` instead when the vector is a color, as it reads better.
---
--- @param x The first component. It is also available as `r`.
--- @param y The second component. It is also available as `g`.
--- @param z The third component. It is also available as `b`.
--- @param w The fourth component. It is also available as `a`.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local v = vec4.V4(1, 2, 3, 4)
--- print(v.x, v.w) -- 1	4
--- v.y = 10
--- print(v.g) -- 10
--- ```
function module.V4(x: number, y: number, z: number, w: number): Vec4
	error("Implemented in native code")
end

--- Creates a quaternion representing a rotation of `angle` radians around an axis.
---
--- Only the `x`, `y` and `z` components of `axis` are used, the `w` component is ignored. The axis should have a length
--- of 1 (see `Vec4:normalized`): the result has a length of 1 only in that case. The result stores the rotation as
--- `(cos(angle / 2), axis.x * sin(angle / 2), axis.y * sin(angle / 2), axis.z * sin(angle / 2))`, so its `x`
--- component is the scalar part. Combine rotations with `Vec4:qmul`.
---
--- @param axis The direction of the axis of rotation. Only `x`, `y` and `z` are read.
--- @param angle The rotation angle in radians.
--- @return A new quaternion.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local quarterTurn = vec4.fromRotation(vec4.V4(0, 0, 1, 0), math.pi / 2)
--- print(quarterTurn:length()) -- 1
--- ```
function module.fromRotation(axis: Vec4, angle: number): Vec4
	error("Implemented in native code")
end

--- Multiplies every component of the vector by a number.
---
--- @param k The factor. A negative value flips the vector, 0 gives the zero vector.
--- @return A new vector. The original is not modified.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local v = vec4.V4(1, 2, 3, 4):scale(0.5)
--- print(v.x, v.w) -- 0.5	2
--- ```
function Vec4Impl.scale(a: Vec4, k: number): Vec4
	error("Implemented in native code")
end

--- Multiplies two quaternions, treating both vectors as quaternions.
---
--- Quaternions store a rotation. Multiplying two of them gives the rotation that applies `b`, then `a`. The `x`
--- component is the scalar part and `y`, `z`, `w` are the vector part, the same layout as `vec4.fromRotation`.
--- The order matters: `a:qmul(b)` is in general not equal to `b:qmul(a)`.
---
--- @param b The second quaternion.
--- @return A new quaternion. It is not normalized: its length is the product of both lengths.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local axis = vec4.V4(0, 0, 1, 0)
--- local quarter = vec4.fromRotation(axis, math.pi / 2)
--- local half = quarter:qmul(quarter)
--- print(half:round(3).w) -- 1
--- ```
function Vec4Impl.qmul(a: Vec4, b: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns the dot product of two vectors.
---
--- This is `a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w`.
---
--- @param b The other vector.
--- @return The dot product.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(1, 2, 3, 4):dot(vec4.V4(1, 0, 1, 0))) -- 4
--- ```
function Vec4Impl.dot(a: Vec4, b: Vec4): number
	error("Implemented in native code")
end

--- Returns the length of the vector.
---
--- The length is `math.sqrt(v.x*v.x + v.y*v.y + v.z*v.z + v.w*v.w)`. If you only compare lengths, `Vec4:lengthSq`
--- is faster because it skips the square root.
---
--- @return The length, always 0 or more.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(2, 0, 0, 0):length()) -- 2
--- ```
function Vec4Impl.length(v: Vec4): number
	error("Implemented in native code")
end

--- Returns the square of the length of the vector.
---
--- This is `v.x*v.x + v.y*v.y + v.z*v.z + v.w*v.w`. It avoids a square root, so it is cheaper than `Vec4:length`.
---
--- @return The squared length, always 0 or more.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(1, 2, 2, 0):lengthSq()) -- 9
--- ```
function Vec4Impl.lengthSq(v: Vec4): number
	error("Implemented in native code")
end

--- Returns the distance between two vectors.
---
--- This is the same as `(a - b):length()`.
---
--- @param b The other vector.
--- @return The distance, always 0 or more.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(0, 0, 0, 0):distance(vec4.V4(3, 4, 0, 0))) -- 5
--- ```
function Vec4Impl.distance(a: Vec4, b: Vec4): number
	error("Implemented in native code")
end

--- Returns the vector scaled to a length of 1.
---
--- The direction is kept. The zero vector cannot be scaled, so it is returned as the zero vector.
---
--- @return A new vector of length 1, or the zero vector if the length is 0.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local axis = vec4.V4(0, 0, 5, 0):normalized()
--- print(axis.z) -- 1
--- ```
function Vec4Impl.normalized(v: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns the vector with the absolute value of each component.
---
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(-1, 2, -3, 0):abs()) -- V4(1, 2, 3, 0)
--- ```
function Vec4Impl.abs(v: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns the vector with each component rounded to a number of decimal digits.
---
--- Halves are rounded away from zero. It is useful to print vectors or to remove tiny floating point errors.
---
--- @param digits_of_precision The number of digits to keep after the decimal point. Defaults to 0, which rounds to the
--- nearest integer. It cannot be negative.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local v = vec4.V4(0.123456, 1.5, -2.5, 0)
--- print(v:round()) -- V4(0, 2, -3, 0)
--- print(v:round(2)) -- V4(0.12, 1.5, -2.5, 0)
--- ```
---
--- # Errors
---
--- Raises an error if `digits_of_precision` is negative.
function Vec4Impl.round(v: Vec4, digits_of_precision: number?): Vec4
	error("Implemented in native code")
end

--- Returns the vector with each component rounded down to the nearest integer.
---
--- @return A new vector. Negative values go further from zero: `-1.5` becomes `-2`.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(1.7, -1.2, 3, 0.1):floor()) -- V4(1, -2, 3, 0)
--- ```
function Vec4Impl.floor(v: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns the vector with each component rounded up to the nearest integer.
---
--- @return A new vector. Negative values go closer to zero: `-1.5` becomes `-1`.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(1.2, -1.7, 3, 0.1):ceil()) -- V4(2, -1, 3, 1)
--- ```
function Vec4Impl.ceil(v: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns the component-wise maximum of two vectors.
---
--- Each component of the result is the larger of the two matching components.
---
--- @param b The other vector.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(1, 5, 0, 2):max(vec4.V4(3, 2, 0, 1))) -- V4(3, 5, 0, 2)
--- ```
function Vec4Impl.max(a: Vec4, b: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns the component-wise minimum of two vectors.
---
--- Each component of the result is the smaller of the two matching components. Combined with `Vec4:max`, it clamps a
--- vector between two bounds.
---
--- @param b The other vector.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(1, 5, 0, 2):min(vec4.V4(3, 2, 0, 1))) -- V4(1, 2, 0, 1)
--- ```
function Vec4Impl.min(a: Vec4, b: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns a vector linearly interpolated between two vectors.
---
--- The result is `a + (b - a) * k`. With colors, it fades from one color to another. The factor is not clamped: a
--- value outside 0 to 1 goes past `a` or `b`.
---
--- @param b The vector reached when `k` is 1.
--- @param k The interpolation factor. 0 gives `a`, 1 gives `b`, 0.5 gives the middle.
--- @return A new vector.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local fade = vec4.BLACK:lerp(vec4.WHITE, 0.25)
--- print(fade.r) -- 0.25
--- ```
---
--- # See also
---
--- - `vec4.createColor`: creates a color.
function Vec4Impl.lerp(a: Vec4, b: Vec4, k: number): Vec4
	error("Implemented in native code")
end

--- Returns a vector with the sign of each component.
---
--- @return A new vector where each component is 1 if the original is positive, -1 if it is negative, and 0 if it is 0.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- print(vec4.V4(4, -0.2, 0, 7):sign()) -- V4(1, -1, 0, 1)
--- ```
function Vec4Impl.sign(v: Vec4): Vec4
	error("Implemented in native code")
end

--- Returns the simplex noise value at the position given by the vector.
---
--- Noise is a smooth pseudo-random function: close points give close values. It is useful for terrain, clouds or
--- natural-looking movement. The four components are the four coordinates of the point, and the seed is fixed, so the
--- same point always gives the same value. Uses [simplex noise](https://en.wikipedia.org/wiki/OpenSimplex_noise).
---
--- @return The noise value at that point.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local time = 0
--- function Update(deltaTime: number)
--- 	time += deltaTime
--- 	local wobble = vec4.V4(time, 0, 0, 0):noise()
--- 	print(wobble)
--- end
--- ```
function Vec4Impl.noise(v: Vec4): number
	error("Implemented in native code")
end

--- Returns the Worley noise value at the position given by the vector.
---
--- Worley noise is based on the distance to random points, which gives a cell-like pattern. The four components are
--- the four coordinates of the point, and the seed is fixed, so the same point always gives the same value. Uses
--- [Worley noise](https://en.wikipedia.org/wiki/Worley_noise). See `Vec4:noise` for a smoother alternative.
---
--- @return The noise value at that point.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local cell = vec4.V4(1.5, 2.5, 0, 0):worleyNoise()
--- print(cell)
--- ```
function Vec4Impl.worleyNoise(v: Vec4): number
	error("Implemented in native code")
end

--- The zero vector: `(0, 0, 0, 0)`. It cannot be modified.
module.ZERO4 = module.V4(0, 0, 0, 0)

--MARK: Colors

--- Creates a color from its red, green, blue and alpha components.
---
--- This does the same as `vec4.V4`. Components are numbers between 0 and 1, where `a` is the opacity: 0 is fully
--- transparent and 1 is fully opaque. Values outside that range are not clamped by this function.
---
--- @param r The amount of red, from 0 to 1.
--- @param g The amount of green, from 0 to 1.
--- @param b The amount of blue, from 0 to 1.
--- @param a The opacity, from 0 (transparent) to 1 (opaque).
--- @return A new color.
---
--- # Example
---
--- ```luau
--- const vec4 = require("@vectarine/vec4")
---
--- local seaGreen = vec4.createColor(0.1, 0.6, 0.4, 1)
--- print(seaGreen.g) -- 0.6
--- ```
---
--- # See also
---
--- - `vec4.V4`: the same function, for vectors that are not colors.
--- - `Vec4:lerp`: fades between two colors.
function module.createColor(r: number, g: number, b: number, a: number): Vec4
	error("Implemented in native code")
end

--- Opaque black: `(0, 0, 0, 1)`.
module.BLACK = module.createColor(0, 0, 0, 1)
--- Opaque white: `(1, 1, 1, 1)`.
module.WHITE = module.createColor(1, 1, 1, 1)
--- Fully transparent black: `(0, 0, 0, 0)`.
module.TRANSPARENT = module.createColor(0, 0, 0, 0)

--- Opaque red: `(1, 0, 0, 1)`.
module.RED = module.createColor(1, 0, 0, 1)
--- Opaque green: `(0, 1, 0, 1)`.
module.GREEN = module.createColor(0, 1, 0, 1)
--- Opaque blue: `(0, 0, 1, 1)`.
module.BLUE = module.createColor(0, 0, 1, 1)

--- Opaque yellow: `(1, 1, 0, 1)`.
module.YELLOW = module.createColor(1, 1, 0, 1)
--- Opaque magenta: `(1, 0, 1, 1)`.
module.MAGENTA = module.createColor(1, 0, 1, 1)
--- Opaque cyan: `(0, 1, 1, 1)`.
module.CYAN = module.createColor(0, 1, 1, 1)

--- Opaque azure, a light blue: `(0, 0.5, 1, 1)`.
module.AZURE = module.createColor(0, 0.5, 1, 1)
--- Opaque orange: `(1, 0.5, 0, 1)`.
module.ORANGE = module.createColor(1, 0.5, 0, 1)
--- Opaque purple: `(0.5, 0, 0.5, 1)`.
module.PURPLE = module.createColor(0.5, 0, 1, 1)
--- Opaque spring green: `(0, 1, 0.5, 1)`.
module.SPRING = module.createColor(0, 1, 0.5, 1)
--- Opaque lime: `(0.5, 1, 0, 1)`.
module.LIME = module.createColor(0.5, 1, 0, 1)
--- Opaque pink: `(1, 0, 1, 1)`, the same value as `vec4.MAGENTA`.
module.PINK = module.createColor(1, 0, 0.5, 1)

--- Opaque light gray: `(0.7, 0.7, 0.7, 1)`.
module.LIGHT_GRAY = module.createColor(0.7, 0.7, 0.7, 1)
--- Opaque dark gray: `(0.3, 0.3, 0.3, 1)`.
module.DARK_GRAY = module.createColor(0.3, 0.3, 0.3, 1)

--- Opaque gray: `(0.5, 0.5, 0.5, 1)`.
module.GRAY = module.createColor(0.5, 0.5, 0.5, 1)
--- Opaque dark red: `(0.5, 0, 0, 1)`.
module.DARK_RED = module.createColor(0.5, 0, 0, 1)
--- Opaque dark green: `(0, 0.5, 0, 1)`.
module.DARK_GREEN = module.createColor(0, 0.5, 0, 1)
--- Opaque dark blue: `(0, 0, 0.5, 1)`.
module.DARK_BLUE = module.createColor(0, 0, 0.5, 1)

return module
