local Image = require("@vectarine/image")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--[[
A module for writing high performance code by storing data in a compact way

# Principles of fastlists
- A fastlist contains a list of Vec2
- Any function that takes a Vec2 and outputs another Vec2 also exists for Fastlist with the same syntax
- You can index a fastlist using .get
- You can draw the content of a fastlist quickly using functions starting with draw
]]
local module = {}

local FastListImpl = { fast_list = true }
FastListImpl.__index = FastListImpl
export type FastList = typeof(setmetatable({}, FastListImpl))

type FastListOrVec = FastList | Vec.Vec2

--- A gap is a special value used to represent empty indices in a fastlist, similar to a `nil` in a table. When you filter a fastlist, you get a new fastlist with gaps.
--- Gaps are 'contagious', meaning that adding/substracting/multiplying, etc... a gap to a number will result in a gap.
--- You can remove gaps using the `replaceGaps` or `otherwise` function.
module.GAP = Vec.V2(0 / 0, 0 / 0)

--MARK: Constructors

--- Return a list of Vec2 that are linearly spaced between min and max in a 2D grid
function module.newLinspace(min: Vec.Vec2, max: Vec.Vec2, step: Vec.Vec2): FastList
	error("Implemented in native code")
end

--- Return a list of Vec2 of size `size` filled with `Vec2.ZERO`
function module.zeros(size: number): FastList
	error("Implemented in native code")
end

--- Return a list of Vec2 of size `size` filled with `value`
function module.fromValue(value: Vec.Vec2, size: number): FastList
	error("Implemented in native code")
end

--- Convert a table of Vec2 into a FastList
function module.fromTable(tbl: { Vec.Vec2 }): FastList
	error("Implemented in native code")
end

--MARK: Table like

--- Returns the number of Vec2 in the fast list
--- `#fastlist` is equivalent to `fastlist.__len(fastlist)`
function FastListImpl.__len(self: FastList): number
	error("Implemented in native code")
end

--- Returns the Vec2 at the given index
function FastListImpl.get(self: FastList, index: number): Vec.Vec2
	error("Implemented in native code")
end

--- Returns a new fast list that is the concatenation of the two fast lists
--- If one fastlist is ABC and the other is XYZ, the result is ABCXYZ
function FastListImpl.concat(self: FastList, other: FastList): FastList
	error("Implemented in native code")
end

--- Returns a table of Vec2 from the fast list
function FastListImpl.toTable(self: FastList): { Vec.Vec2 }
	error("Implemented in native code")
end

--MARK: Manipulation

--- Apply a function to each Vec2 in the fast list. You can modify the vector in-place
--- in the function to modify elements of the fast list efficiently
--- This function is not as efficient that the other as it calls Lua code for each element.
function FastListImpl.forEach(self: FastList, fn: (Vec.Vec2, number) -> ()): FastList
	error("Implemented in native code")
end

--- Create a new FastList where every Vec2 is repeated `count` times
--- A list like XYZ with a count of 2 becomes XXYYZZ
function FastListImpl.componentRepeated(self: FastList, count: number): FastList
	error("Implemented in native code")
end

--- Create a new FastList where the list is repeated `count` times
--- A list like XYZ with a count of 2 becomes XYZXYZ
function FastListImpl.repeated(self: FastList, count: number): FastList
	error("Implemented in native code")
end

--- Create a new Fastlist by weaving elements from multiple fastlists
--- If the fastlists are ABC, DEF and GHI, the result is ADGBCEFH
function FastListImpl.weave(self: FastList, others: { FastList }): FastList
	error("Implemented in native code")
end

--- Unweave a fastlist into `groupCount` fastlists. This is the opposite of weave.
--- A fastlist like ADGBCEFH with a group count of 3 becomes ABC, DEF and GHI
--- ADGBCEFH with a group count of 2 becomes AGCF and DBEH
--- The resulting fastlists might not be of the same size if the original fastlist size is not a multiple of groupCount.
function FastListImpl.unweave(self: FastList, groupCount: number): { FastList }
	error("Implemented in native code")
end

--MARK: Filter Gap

--[[
Filter the Fastlist on the X component, keeping only the Vec2s for which the corresponding Vec2 in the mask has a x component between min and max.
The mask and self and expected to be of the same size.

The indices outside the range are replaced with `GAP`s. If there were already gaps in the mask or fastlist, they are preserved.
The resulting fastlist has the same length as the original one.

Conside the following non-fastlist code:
```lua
local filtered = {}
for i, value in pairs(mask):
	if min <= value.x <= max then
		filtered[i] = fastlist[I]
	end
end
```
The equivalent fastlist code is:
```lua
local filtered = fastlist:filterBetweenX(mask, min, max)
```

You can use `math.inf` for max or `-math.inf` for min for taking greater or smaller values.
]]
function FastListImpl.filterBetweenX(
	fastlist: FastList,
	mask: FastList,
	min: number | FastList,
	max: number | FastList
): FastList
	error("Implemented in native code")
end

--- Filter the Fastlist on the Y component, keeping only the Vec2s for which the corresponding Vec2 in the mask has a y component between min and max.
--- Gaps are preserved.
---
--- The resulting fastlist has the same length as the original one.
--- See filterBetweenX for more information
function FastListImpl.filterBetweenY(
	fastlist: FastList,
	mask: FastList,
	min: number | FastList,
	max: number | FastList
): FastList
	error("Implemented in native code")
end

--- Replace gaps in the Fastlist with the replacement value
function FastListImpl.replaceGaps(self: FastList, replacement: Vec.Vec2): FastList
	error("Implemented in native code")
end

--- Remove gaps from the Fastlist, resulting in a shorter list
function FastListImpl.filterGaps(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the elements of the fastlist at indices where there are no gaps, otherwise, the gap is replaced with the value from other.
--- Self and other are expected to be of the same size.
--- This is similar to how `or` works for numbers: `4 or 5` is `4`, but `nil or 5` is `5`
function FastListImpl.otherwise(self: FastList, other: FastList): FastList
	error("Implemented in native code")
end

--- Returns the elements of `other` at indices where there is no gaps inside both fastlists. If there is a gap in one of the fastlists, a gap will be there in the result.
--- This is similar to how `and` works for numbers: `4 and 5` is `5`, but `nil and 5` is `nil`
function FastListImpl.also(self: FastList, other: FastList): FastList
	error("Implemented in native code")
end

--- Return the number of gaps in the fastlist
function FastListImpl.countGaps(self: FastList): number
	error("Implemented in native code")
end

--MARK: Vec2-like
-- These functions are the same as Vec2 with the same API

--- Element-wise addition of the fastlist and another fastlist
function FastListImpl.__add(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Element-wise subtraction of the fastlist by another fastlist
function FastListImpl.__sub(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Element-wise multiplication of the fastlist by another fastlist
function FastListImpl.__mul(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Element-wise division of the fastlist by another fastlist
function FastListImpl.__div(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Scales the fastlist by the given factor
function FastListImpl.scale(self: FastList, k: number): FastList
	error("Implemented in native code")
end

--- Multiply (as complex numbers) the fastlist by another fastlist, element-wise
function FastListImpl.cmul(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Compute the dot product of the fastlist and another fastlist, element-wise
function FastListImpl.dot(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Normalize each element in the fastlist
function FastListImpl.normalized(self: FastList): FastList
	error("Implemented in native code")
end

--- Round each element in the fastlist to the given number of digits of precision
--- If `digits_of_precision` is not provided, it defaults to 0 (round to nearest integer)
function FastListImpl.round(self: FastList, digits_of_precision: number?): FastList
	error("Implemented in native code")
end

--- Round down each element in the fastlist
function FastListImpl.floor(self: FastList): FastList
	error("Implemented in native code")
end

--- Round up each element in the fastlist
function FastListImpl.ceil(self: FastList): FastList
	error("Implemented in native code")
end

--- Compute the component-wise maximum of the fastlist and another fastlist
function FastListImpl.max(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Compute the component-wise minimum of the fastlist and another fastlist
function FastListImpl.min(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Linearly interpolate between the fastlist and another fastlist, element-wise
function FastListImpl.lerp(self: FastList, other: FastListOrVec, k: number): FastList
	error("Implemented in native code")
end

--- Compute the sign of every element in the fastlist
function FastListImpl.sign(self: FastList): FastList
	error("Implemented in native code")
end

--- Compute simplex noise for each Vec2 in the fast list and return a new fast list
--- where the Vec2 both contain the noise value.
function FastListImpl.noise(self: FastList): FastList
	error("Implemented in native code")
end

--- Compute worley noise for each Vec2 in the fast list and return a new fast list
function FastListImpl.worleyNoise(self: FastList): FastList
	error("Implemented in native code")
end

--- Convert each Vec2 in the fastlist to polar coordinates, element-wise
--- So, the x component becomes the length and the y component becomes the angle
function FastListImpl.toPolar(self: FastList): FastList
	error("Implemented in native code")
end

--- Convert each Vec2 in the fastlist to Cartesian coordinates, element-wise
--- So, the first component (the length) becomes x and the second component (the angle) becomes the y.
function FastListImpl.toCartesian(self: FastList): FastList
	error("Implemented in native code")
end

--MARK: Drawing

--- Take the fastlist by groups of 4 Vec2 (pos, size and 2 Vec2 for the color which is a Vec4).
--- For each of these groups, perform the same operation as Graphics.drawRect()
--- This is a lot faster than calling Graphics.drawRect() in a loop
function FastListImpl.drawRects(self: FastList)
	error("Implemented in native code")
end

--- Take the fastlist by groups of 6 Vec2 (p1, p2, p3, p4 and 2 Vec2 for the color which is a Vec4).
--- For each of these groups, perform the same operation as `Graphics.drawPolygon({p1, p2, p3, p4}, color)`
--- This is a lot faster than calling `Graphics.drawPolygon({p1, p2, p3, p4}, color)` in a loop
function FastListImpl.drawQuads(self: FastList)
	error("Implemented in native code")
end

--- Take the fastlist by groups of 2 Vec2 (pos and size).
--- For each of these groups, perform the same operation as `image:drawImage(pos, size)`
--- This is a lot faster than calling `image:drawImage(pos, size)` in a loop
function FastListImpl.drawImages(self: FastList, image: Image.ImageResource, color: Vec4.Vec4?)
	error("Implemented in native code")
end

--- Take the fastlist by groups of 6 Vec2 (p1, p2, p3, p4, src_pos and src_size).
--- For each of these groups, perform the same operation as `image:drawImagePart(p1, p2, p3, p4, src_pos, src_size)`
--- This is a lot faster than calling `image:drawImagePart(p1, p2, p3, p4, src_pos, src_size)` in a loop
function FastListImpl.drawImageParts(self: FastList, image: Image.ImageResource, color: Vec4.Vec4?)
	error("Implemented in native code")
end

return module
