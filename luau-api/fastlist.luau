local Image = require("@vectarine/image")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--[[
Fast lists of 2D vectors, for processing and drawing thousands of objects at once.

A `FastList` is a list of `Vec2` stored in a compact way. Where a plain table of `Vec2` needs a Lua loop to change or
draw every element, a fastlist applies one operation to all its elements in a single call to native code. Use one
when you have many similar objects, such as particles, bullets, tiles or stars. For a few objects, a plain table of
`Vec2` is simpler and fast enough.

Most `Vec2` operations exist on fastlists with the same name: `FastList:scale`, `FastList:normalized`, `+`, `-`...
They work element by element and return a new fastlist. A fastlist can also be combined with a single `Vec2`, which
is then used for every element. The functions starting with `draw`, such as `FastList:drawRects` and
`FastList:drawImages`, draw many shapes in one call. Functions of this module create fastlists, for example
`fastlist.fromTable` from a table of vectors. Read elements back with `FastList:get` or `FastList:toTable`.

Indices start at 1, like in Lua tables. When a fastlist is combined with another fastlist of a different length,
the result has the length of the shorter one.

A gap is an empty slot, like a `nil` in a table. Filtering functions such as `FastList:filterBetweenX` replace the
elements they reject with `fastlist.GAP`, so that every element keeps its index. Gaps are removed with
`FastList:replaceGaps` or `FastList:filterGaps`. See `fastlist.GAP`.

See `vec` for the vectors stored in a fastlist.

# Example

```luau
const vec = require("@vectarine/vec")
const fastlist = require("@vectarine/fastlist")

local positions = fastlist.fromTable({ vec.V2(0, 0), vec.V2(0.5, 0.5) })
local velocities = fastlist.fromValue(vec.V2(0.5, 0), 2)
local moved = positions + velocities:scale(0.5)
print(moved:get(2)) -- V2(0.75, 0.5)
```

# Common mistakes

- A fastlist passed as an argument to a method whose description says it "consumes" it cannot be used afterwards.
If you still need that list, create it again, for example with `fastlist.fromTable`.
- A gap is not `nil`: `list:get(i)` returns a vector full of `nan` for it, and `list:get(i).x == 0` is `false`. Test with
`FastList:countGaps` or remove gaps with `FastList:filterGaps` before using the values.
- Some methods turn gaps into regular vectors. When a method does not keep gaps, its description says so.
- `list * 2` raises an error, because operators need a fastlist or a `Vec2`. Use `FastList:scale` to multiply by a
number.
]]
local module = {}

local FastListImpl = { fast_list = true }
FastListImpl.__index = FastListImpl
--- A list of `Vec2` stored in a compact way, which can be processed and drawn all at once.
---
--- You get one from a constructor of the `fastlist` module such as `fastlist.fromTable`, or as the result of a
--- method of this type. Methods return a new list and leave the original unchanged, unless their description says
--- they change it in place. Use `#list` for the number of elements and `FastList:get` to read one element. Methods
--- whose name starts with `draw` display the list.
---
--- Some methods consume a fastlist argument: after `a:concat(b)`, `b` can no longer be used. The description of
--- such a method says "consumes".
export type FastList = typeof(setmetatable({}, FastListImpl))

--- Either a fastlist, to combine element by element, or a single `Vec2`, used for every element.
type FastListOrVec = FastList | Vec.Vec2

--- The value of an empty slot in a fastlist, similar to a `nil` in a table.
---
--- A gap is a `Vec2` whose two components are `nan` (not a number). Filtering a fastlist with
--- `FastList:filterBetweenX` or `FastList:filterBetweenY` gives a new fastlist where the rejected elements are gaps.
--- Gaps are contagious: adding, subtracting, multiplying or dividing a gap gives a gap. Remove them with
--- `FastList:replaceGaps` or `FastList:filterGaps`. Some methods do not keep gaps: their description says so.
module.GAP = Vec.V2(0 / 0, 0 / 0)

--MARK: Constructors

--- Creates a fastlist of the points of a regular grid.
---
--- The points go from `min` to `max` (included, when a point lands exactly on it) with a distance of `step` between
--- two points. They are ordered row by row: `x` changes first, then `y`. The distance is added again and again, so
--- tiny rounding errors can accumulate on long lists.
---
--- @param min The first point of the grid, which is its bottom left corner.
--- @param max The opposite corner of the grid. No point goes past it.
--- @param step The distance between two points, on each axis. Both components must be greater than 0.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if a component of `step` is 0 or negative.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local grid = fastlist.newLinspace(vec.V2(0, 0), vec.V2(2, 1), vec.V2(1, 1))
--- print(#grid) -- 6
--- print(grid:get(2)) -- V2(1, 0)
--- print(grid:get(4)) -- V2(0, 1)
--- ```
---
--- # See also
---
--- - `fastlist.fromTable`: build a list from your own points.
function module.newLinspace(min: Vec.Vec2, max: Vec.Vec2, step: Vec.Vec2): FastList
	error("Implemented in native code")
end

--- Creates a fastlist filled with zero vectors.
---
--- @param size The number of elements. It must not be negative.
--- @return A new fastlist of `size` elements equal to `vec.ZERO2`.
---
--- # Example
---
--- ```luau
--- const fastlist = require("@vectarine/fastlist")
---
--- local speeds = fastlist.zeros(100)
--- print(#speeds) -- 100
--- ```
function module.zeros(size: number): FastList
	error("Implemented in native code")
end

--- Creates a fastlist where every element is the same vector.
---
--- @param value The vector to repeat.
--- @param size The number of elements. It must not be negative.
--- @return A new fastlist of `size` elements equal to `value`.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local sizes = fastlist.fromValue(vec.V2(0.1, 0.1), 50)
--- print(sizes:get(50)) -- V2(0.1, 0.1)
--- ```
---
--- # See also
---
--- - `fastlist.zeros`: fill with zero vectors.
function module.fromValue(value: Vec.Vec2, size: number): FastList
	error("Implemented in native code")
end

--- Creates a fastlist from a table of vectors.
---
--- The vectors are copied: changing the table or its vectors afterwards does not change the fastlist.
---
--- @param tbl A table of `Vec2`, with indices starting at 1.
--- @return A new fastlist with the same elements, in the same order.
---
--- # Errors
---
--- Raises an error if an element of the table is not a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local points = fastlist.fromTable({ vec.V2(0, 0), vec.V2(1, 1) })
--- print(#points) -- 2
--- ```
---
--- # See also
---
--- - `FastList:toTable`: convert back to a table.
--- - `fastlist.fromValue`: repeat one vector.
function module.fromTable(tbl: { Vec.Vec2 }): FastList
	error("Implemented in native code")
end

--MARK: Table like

--- Returns the number of elements of the fastlist.
---
--- You usually write `#fastlist`, which calls this function. Gaps count as elements.
---
--- @return The number of elements.
---
--- # Example
---
--- ```luau
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.zeros(8)
--- print(#list) -- 8
--- ```
function FastListImpl.__len(self: FastList): number
	error("Implemented in native code")
end

--- Returns the element at the given index.
---
--- The first element has index 1. The returned vector is a copy: changing it does not change the fastlist.
---
--- @param index The position of the element, starting at 1.
--- @return The vector at this index, or `nil` if the index is 0, negative or greater than the length of the list.
--- The type says `Vec2` and not `Vec2?` for convenience, like indexing a table: check the index yourself.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 2), vec.V2(3, 4) })
--- print(list:get(1)) -- V2(1, 2)
--- print(list:get(3)) -- nil
--- ```
---
--- # See also
---
--- - `FastList:toTable`: read all elements.
function FastListImpl.get(self: FastList, index: number): Vec.Vec2
	error("Implemented in native code")
end

--- Returns a new fastlist made of the elements of this list followed by the elements of another one.
---
--- If one fastlist is ABC and the other is XYZ, the result is ABCXYZ.
--- This method consumes `other`: it cannot be used after this call.
---
--- @param other The fastlist to append.
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local a = fastlist.fromValue(vec.V2(0, 0), 3)
--- local b = fastlist.fromValue(vec.V2(1, 1), 2)
--- print(#a:concat(b)) -- 5
--- ```
function FastListImpl.concat(self: FastList, other: FastList): FastList
	error("Implemented in native code")
end

--- Returns the elements of the fastlist as a table of vectors.
---
--- Use it when you need to loop over the elements in Lua or to use functions that expect a table, such as
--- `graphics.drawPolygon`. The table starts at index 1 and contains copies of the vectors.
---
--- @return A new table of `Vec2`.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromValue(vec.V2(1, 1), 3)
--- for i, point in list:toTable() do
--- 	print(i, point.x)
--- end
--- ```
---
--- # See also
---
--- - `fastlist.fromTable`: the opposite conversion.
function FastListImpl.toTable(self: FastList): { Vec.Vec2 }
	error("Implemented in native code")
end

--- Returns the `x` component of every element as a table of numbers.
---
--- @return A new table of numbers, starting at index 1.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 2), vec.V2(3, 4) })
--- print(table.concat(list:toNumberTableX(), ", ")) -- 1, 3
--- ```
function FastListImpl.toNumberTableX(self: FastList): { number }
	error("Implemented in native code")
end

--MARK: Manipulation

--- Replaces every element of the fastlist by the result of a function, changing the list in place.
---
--- The function is called for each element, in order, with a copy of the vector and its index (starting at 1). It must
--- return a vector, which becomes the new element. Changing the vector received by the function has no effect unless
--- it is returned. If the function raises an error, the loop stops and the elements already visited keep their new
--- value.
---
--- This method calls Lua code once per element, so it is much slower than the other methods. Use it only when
--- nothing else can express what you need.
---
--- @param fn The function called with the vector and its index. It must return the new vector.
---
--- # Errors
---
--- Raises an error if `fn` does not return a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.zeros(3)
--- list:forEach(function(v, i)
--- 	return vec.V2(i, i * 2)
--- end)
--- print(list:get(3)) -- V2(3, 6)
--- ```
---
--- # See also
---
--- - `FastList:scale`: faster operations on all elements.
function FastListImpl.forEach(self: FastList, fn: (Vec.Vec2, number) -> Vec.Vec2): ()
	error("Implemented in native code")
end

--- Returns a new fastlist where every element is repeated several times in a row.
---
--- A list like XYZ with a count of 2 becomes XXYYZZ. See `FastList:repeated` to repeat the whole list instead.
---
--- @param count How many times each element appears. With 0, the result is empty.
--- @return A new fastlist of `#self * count` elements.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 0), vec.V2(2, 0) })
--- local result = list:componentRepeated(2)
--- print(table.concat(result:toNumberTableX(), ", ")) -- 1, 1, 2, 2
--- ```
function FastListImpl.componentRepeated(self: FastList, count: number): FastList
	error("Implemented in native code")
end

--- Returns a new fastlist where the whole list is repeated several times.
---
--- A list like XYZ with a count of 2 becomes XYZXYZ. See `FastList:componentRepeated` to repeat each element instead.
---
--- @param count How many times the list appears. With 0, the result is empty.
--- @return A new fastlist of `#self * count` elements.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 0), vec.V2(2, 0) })
--- local result = list:repeated(2)
--- print(table.concat(result:toNumberTableX(), ", ")) -- 1, 2, 1, 2
--- ```
function FastListImpl.repeated(self: FastList, count: number): FastList
	error("Implemented in native code")
end

--- Returns a new fastlist that alternates the elements of several fastlists.
---
--- If the fastlists are ABC, DEF and GHI, the result is ADGBEHCFI: the first element of each list, then the second
--- element of each list, and so on. The list `self` comes first in each round, then the lists of `others` in order.
--- This is how you build the groups of vectors that the `draw` methods expect, such as a position followed by a size.
--- When the lists have different lengths, a list that has run out is skipped. If `others` is empty, the result is a
--- copy of `self`. This method consumes the lists in `others`: they cannot be used after this call.
---
--- @param others The fastlists to weave with `self`.
--- @return A new fastlist. Its length is the sum of the lengths of all the lists.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local positions = fastlist.fromTable({ vec.V2(0, 0), vec.V2(0.5, 0) })
--- local sizes = fastlist.fromValue(vec.V2(0.1, 0.1), 2)
--- local pairs = positions:weave({ sizes })
--- print(#pairs) -- 4
--- print(pairs:get(2)) -- V2(0.1, 0.1)
--- ```
---
--- # See also
---
--- - `FastList:unweave`: the opposite operation.
--- - `FastList:drawImages`: uses position and size pairs.
function FastListImpl.weave(self: FastList, others: { FastList }): FastList
	error("Implemented in native code")
end

--- Splits a fastlist into several fastlists by dealing its elements in turn. This is the opposite of
--- `FastList:weave`.
---
--- A fastlist like ADGBEHCFI with a group count of 3 becomes ABC, DEF and GHI. ADGBEH with a group count of 2 becomes
--- AGE and DBH. If the length of the list is not a multiple of `groupCount`, the first groups are one element longer
--- than the others.
---
--- @param groupCount The number of fastlists to produce. It must be at least 1.
--- @return A table of `groupCount` fastlists.
---
--- # Errors
---
--- Raises an error if `groupCount` is 0.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local mixed = fastlist.fromTable({ vec.V2(1, 0), vec.V2(2, 0), vec.V2(3, 0), vec.V2(4, 0) })
--- local groups = mixed:unweave(2)
--- print(table.concat(groups[1]:toNumberTableX(), ", ")) -- 1, 3
--- print(table.concat(groups[2]:toNumberTableX(), ", ")) -- 2, 4
--- ```
---
--- # See also
---
--- - `FastList:weave`: the opposite operation.
function FastListImpl.unweave(self: FastList, groupCount: number): { FastList }
	error("Implemented in native code")
end

--MARK: Filter Gap

--[[
Replaces the elements of the fastlist with gaps, except where the `x` component of the mask is between `min` and `max`.

The element at index `i` is kept if `min < mask[i].x < max`, which excludes both limits. Otherwise it becomes
`fastlist.GAP`. An existing gap in the fastlist stays a gap, and a gap in the mask always gives a gap. The result has
the length of the shortest of `fastlist`, `mask` and, when they are lists, `min` and `max`.

In a plain table, this is a loop that keeps `list[i]` when `min < mask[i].x and mask[i].x < max`. Here, one call does it for the whole list.

Use `math.huge` for `max` or `-math.huge` for `min` to leave one side open. When `min` or `max` is a fastlist, the
`x` component of its element at the same index is the limit for that element.

@param mask The list whose `x` components are tested.
@param min The lower limit, as a number or as a fastlist with one limit per element.
@param max The upper limit, as a number or as a fastlist with one limit per element.
@return A new fastlist where the rejected elements are gaps.

# Errors

Raises an error if `mask` is not a fastlist, or if `min` or `max` is neither a number nor a fastlist.

# Example

```luau
const vec = require("@vectarine/vec")
const fastlist = require("@vectarine/fastlist")

local ages = fastlist.fromTable({ vec.V2(1, 0), vec.V2(5, 0), vec.V2(9, 0) })
local enemies = fastlist.fromTable({ vec.V2(0, 0), vec.V2(0.5, 0), vec.V2(1, 0) })
local alive = enemies:filterBetweenX(ages, -math.huge, 6)
print(alive:countGaps()) -- 1
```

# See also

- `FastList:filterBetweenY`: filter on the y component.
- `FastList:replaceGaps`: fill the gaps.
]]
function FastListImpl.filterBetweenX(
	fastlist: FastList,
	mask: FastList,
	min: number | FastList,
	max: number | FastList
): FastList
	error("Implemented in native code")
end

--- Replaces the elements of the fastlist with gaps, except where the `y` component of the mask is between `min`
--- and `max`.
---
--- It works like `FastList:filterBetweenX`, but tests the `y` component of the mask and, when `min` or `max` is a
--- fastlist, the `y` component of its elements. Both limits are excluded.
---
--- @param mask The list whose `y` components are tested.
--- @param min The lower limit, as a number or as a fastlist with one limit per element.
--- @param max The upper limit, as a number or as a fastlist with one limit per element.
--- @return A new fastlist where the rejected elements are gaps.
---
--- # Errors
---
--- Raises an error if `mask` is not a fastlist, or if `min` or `max` is neither a number nor a fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local stars = fastlist.fromTable({ vec.V2(0, -0.5), vec.V2(0, 0.5), vec.V2(0, 2) })
--- local visible = stars:filterBetweenY(stars, -1, 1)
--- print(visible:countGaps()) -- 1
--- ```
---
--- # See also
---
--- - `FastList:filterBetweenX`: filter on the x component.
--- - `FastList:replaceGaps`: fill the gaps.
function FastListImpl.filterBetweenY(
	fastlist: FastList,
	mask: FastList,
	min: number | FastList,
	max: number | FastList
): FastList
	error("Implemented in native code")
end

--- Returns a copy of the fastlist where every gap is replaced by a vector.
---
--- @param replacement The vector put in place of each gap.
--- @return A new fastlist of the same length.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 1), fastlist.GAP })
--- local filled = list:replaceGaps(vec.V2(0, 0))
--- print(filled:countGaps()) -- 0
--- print(filled:get(2)) -- V2(0, 0)
--- ```
---
--- # See also
---
--- - `FastList:filterGaps`: remove the gaps instead.
--- - `FastList:otherwise`: fill from another list.
function FastListImpl.replaceGaps(self: FastList, replacement: Vec.Vec2): FastList
	error("Implemented in native code")
end

--- Returns a copy of the fastlist without its gaps.
---
--- The remaining elements keep their order but get new indices, so the result is shorter when there were gaps.
---
--- @return A new fastlist with no gaps.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 1), fastlist.GAP, vec.V2(2, 2) })
--- print(#list:filterGaps()) -- 2
--- ```
---
--- # See also
---
--- - `FastList:replaceGaps`: keep the length.
function FastListImpl.filterGaps(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the fastlist where each gap is replaced by the element of another list at the same index.
---
--- This is similar to how `or` works for values: `4 or 5` is `4`, but `nil or 5` is `5`. This method consumes `other`:
--- it cannot be used after this call. The result has the length of the shorter list.
---
--- @param other The fastlist that provides the values for the gaps.
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 1), fastlist.GAP })
--- local backup = fastlist.fromValue(vec.V2(9, 9), 2)
--- print(list:otherwise(backup):get(2)) -- V2(9, 9)
--- ```
function FastListImpl.otherwise(self: FastList, other: FastList): FastList
	error("Implemented in native code")
end

--- Returns the elements of another list, except where either list has a gap.
---
--- At each index, the result is the element of `other` if neither list has a gap there, and a gap otherwise. This is
--- similar to how `and` works for values: `4 and 5` is `5`, but `nil and 5` is `nil`. This method consumes `other`:
--- it cannot be used after this call. The result has the length of the shorter list.
---
--- @param other The fastlist that provides the values.
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 1), fastlist.GAP })
--- local other = fastlist.fromValue(vec.V2(9, 9), 2)
--- print(list:also(other):countGaps()) -- 1
--- ```
function FastListImpl.also(self: FastList, other: FastList): FastList
	error("Implemented in native code")
end

--- Returns the number of gaps in the fastlist.
---
--- @return The number of elements that are gaps.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(1, 1), fastlist.GAP })
--- print(list:countGaps()) -- 1
--- ```
function FastListImpl.countGaps(self: FastList): number
	error("Implemented in native code")
end

--MARK: Vec2-like
-- These functions are the same as Vec2 with the same API

--- Returns the element-wise sum of the fastlist and another fastlist or a vector.
---
--- With a fastlist, elements at the same index are added, and the result has the length of the shorter list. With a
--- vector, it is added to every element. The fastlist must be on the left of the `+`.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local positions = fastlist.fromValue(vec.V2(0, 0), 3)
--- local moved = positions + vec.V2(0.5, 0.25)
--- print(moved:get(3)) -- V2(0.5, 0.25)
--- ```
function FastListImpl.__add(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns the element-wise difference of the fastlist and another fastlist or a vector.
---
--- It follows the same rules as `+`.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local positions = fastlist.fromValue(vec.V2(1, 1), 2)
--- print((positions - vec.V2(0.5, 0.5)):get(1)) -- V2(0.5, 0.5)
--- ```
function FastListImpl.__sub(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns a fastlist where both components of every element are negated.
function FastListImpl.__unm(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the element-wise product of the fastlist and another fastlist or a vector.
---
--- It follows the same rules as `+`. Components are multiplied one by one. To multiply every element by a number,
--- use `FastList:scale`.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local sizes = fastlist.fromValue(vec.V2(2, 4), 2)
--- print((sizes * vec.V2(0.5, 0.25)):get(1)) -- V2(1, 1)
--- ```
function FastListImpl.__mul(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns the element-wise quotient of the fastlist and another fastlist or a vector.
---
--- It follows the same rules as `+`. A zero component in the divisor gives `inf` or `nan`, and a `nan` in both
--- components is a gap.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local sizes = fastlist.fromValue(vec.V2(2, 4), 2)
--- print((sizes / vec.V2(2, 2)):get(1)) -- V2(1, 2)
--- ```
function FastListImpl.__div(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns the fastlist with every element multiplied by a number.
---
--- This is `Vec2:scale` applied to each element.
---
--- @param k The factor. A negative value flips the direction of every element.
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local velocities = fastlist.fromValue(vec.V2(1, 2), 100)
--- local step = velocities:scale(0.5)
--- print(step:get(1)) -- V2(0.5, 1)
--- ```
function FastListImpl.scale(self: FastList, k: number): FastList
	error("Implemented in native code")
end

--- Returns the product of the fastlist and another fastlist or a vector, seen as complex numbers.
---
--- This is `Vec2:cmul` applied to each pair of elements. Multiplying by a vector of length 1 rotates each element by
--- the angle of that vector. It follows the same rules as `+` for the length of the result.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromValue(vec.V2(1, 2), 2)
--- print(list:cmul(vec.V2(3, 4)):get(1)) -- V2(-5, 10)
--- ```
function FastListImpl.cmul(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns the dot product of the elements of the fastlist with another fastlist or a vector.
---
--- This is `Vec2:dot` applied to each pair of elements. As a fastlist only contains vectors, each dot product is
--- stored in both components of the resulting element. It follows the same rules as `+` for the length of the
--- result.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist where the `x` and `y` of each element are both the dot product.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromValue(vec.V2(1, 2), 2)
--- print(list:dot(vec.V2(3, 4)):get(1)) -- V2(11, 11)
--- ```
function FastListImpl.dot(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns the fastlist with every element scaled to a length of 1.
---
--- This is `Vec2:normalized` applied to each element. A zero vector stays the zero vector.
---
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local directions = fastlist.fromTable({ vec.V2(0, 3), vec.V2(0, 0) })
--- local unit = directions:normalized()
--- print(unit:get(1)) -- V2(0, 1)
--- print(unit:get(2)) -- V2(0, 0)
--- ```
function FastListImpl.normalized(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the fastlist with every component rounded to a number of decimal digits.
---
--- This is `Vec2:round` applied to each element.
---
--- @param digits_of_precision The number of digits kept after the decimal point. Defaults to 0, which rounds to the
--- nearest integer.
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromValue(vec.V2(1.26, 2.5), 2)
--- print(list:round():get(1)) -- V2(1, 3)
--- print(list:round(1):get(1)) -- V2(1.3, 2.5)
--- ```
function FastListImpl.round(self: FastList, digits_of_precision: number?): FastList
	error("Implemented in native code")
end

--- Returns the fastlist with every component rounded down to the nearest integer.
---
--- This is `Vec2:floor` applied to each element.
---
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromValue(vec.V2(2.7, -1.5), 2)
--- print(list:floor():get(1)) -- V2(2, -2)
--- ```
function FastListImpl.floor(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the fastlist with every component rounded up to the nearest integer.
---
--- This is `Vec2:ceil` applied to each element.
---
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromValue(vec.V2(2.1, -1.5), 2)
--- print(list:ceil():get(1)) -- V2(3, -1)
--- ```
function FastListImpl.ceil(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the component-wise maximum of the fastlist and another fastlist or a vector.
---
--- This is `Vec2:max` applied to each pair of elements. It follows the same rules as `+` for the length of the
--- result. A gap paired with a regular vector gives that vector, so gaps are not kept.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local positions = fastlist.fromTable({ vec.V2(-2, 0.5), vec.V2(0.3, 3) })
--- print(positions:max(vec.V2(-1, -1)):get(1)) -- V2(-1, 0.5)
--- ```
function FastListImpl.max(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns the component-wise minimum of the fastlist and another fastlist or a vector.
---
--- This is `Vec2:min` applied to each pair of elements. It follows the same rules as `+` for the length of the
--- result. A gap paired with a regular vector gives that vector, so gaps are not kept.
---
--- @param other A fastlist or a `Vec2`.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local positions = fastlist.fromTable({ vec.V2(-2, 0.5), vec.V2(0.3, 3) })
--- print(positions:min(vec.V2(1, 1)):get(2)) -- V2(0.3, 1)
--- ```
function FastListImpl.min(self: FastList, other: FastListOrVec): FastList
	error("Implemented in native code")
end

--- Returns the points between the fastlist and another fastlist or a vector, at a given fraction of the way.
---
--- This is `Vec2:lerp` applied to each pair of elements. It follows the same rules as `+` for the length of the
--- result.
---
--- @param other A fastlist or a `Vec2`, reached when `k` is 1.
--- @param k The fraction of the way from the elements of the list to `other`. It is not limited to the range 0 to 1.
--- @return A new fastlist.
---
--- # Errors
---
--- Raises an error if `other` is neither a fastlist nor a vector.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local positions = fastlist.fromValue(vec.V2(0, 0), 3)
--- local nearer = positions:lerp(vec.V2(1, 2), 0.5)
--- print(nearer:get(1)) -- V2(0.5, 1)
--- ```
function FastListImpl.lerp(self: FastList, other: FastListOrVec, k: number): FastList
	error("Implemented in native code")
end

--- Returns the sign of every component of the fastlist.
---
--- This is `Vec2:sign` applied to each element: each component becomes 1, -1 or 0. A gap becomes `V2(0, 0)`, so gaps
--- are not kept.
---
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(-0.3, 4) })
--- print(list:sign():get(1)) -- V2(-1, 1)
--- ```
function FastListImpl.sign(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the simplex noise value at every element of the fastlist.
---
--- This is `Vec2:noise` applied to each element. As a fastlist only contains vectors, the noise value is stored in
--- both components of the resulting element. The noise is the same on every run.
---
--- @return A new fastlist where the `x` and `y` of each element are both the noise value.
---
--- # Example
---
--- ```luau
--- const fastlist = require("@vectarine/fastlist")
--- const vec = require("@vectarine/vec")
---
--- local grid = fastlist.newLinspace(vec.V2(0, 0), vec.V2(1, 1), vec.V2(0.25, 0.25))
--- local heights = grid:noise()
--- print(heights:get(1).x)
--- ```
function FastListImpl.noise(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the Worley noise value at every element of the fastlist.
---
--- This is `Vec2:worleyNoise` applied to each element. As a fastlist only contains vectors, the noise value is stored
--- in both components of the resulting element. The noise is the same on every run.
---
--- @return A new fastlist where the `x` and `y` of each element are both the noise value.
---
--- # Example
---
--- ```luau
--- const fastlist = require("@vectarine/fastlist")
--- const vec = require("@vectarine/vec")
---
--- local grid = fastlist.newLinspace(vec.V2(0, 0), vec.V2(1, 1), vec.V2(0.25, 0.25))
--- local cells = grid:worleyNoise()
--- print(cells:get(1).x)
--- ```
function FastListImpl.worleyNoise(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the elements of the fastlist converted to polar coordinates.
---
--- The `x` component of each result is the length of the original vector and the `y` component is its angle in
--- radians. This is `Vec2:length` and `Vec2:angle` applied to each element. See `FastList:toCartesian` for the
--- opposite conversion.
---
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local list = fastlist.fromTable({ vec.V2(3, 4) })
--- print(list:toPolar():get(1).x) -- 5
--- ```
function FastListImpl.toPolar(self: FastList): FastList
	error("Implemented in native code")
end

--- Returns the elements of the fastlist converted from polar coordinates to Cartesian ones.
---
--- The `x` component of each element is read as a length and the `y` component as an angle in radians. The result is
--- the vector of this length pointing in this direction, like `vec.fromAngle`. This is the opposite of
--- `FastList:toPolar`.
---
--- @return A new fastlist.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local polar = fastlist.fromTable({ vec.V2(2, 0) })
--- print(polar:toCartesian():get(1)) -- V2(2, 0)
--- ```
function FastListImpl.toCartesian(self: FastList): FastList
	error("Implemented in native code")
end

--MARK: Drawing

--- Draws a filled rectangle for each group of 4 elements of the fastlist.
---
--- Each group is: the position, the size, then the color as two vectors `(r, g)` and `(b, a)`, with every channel
--- between 0 and 1. Each rectangle is drawn like `graphics.drawRect` would. This is a lot faster than calling
--- `graphics.drawRect` in a loop. If the length of the list is not a multiple of 4, the last incomplete group is
--- ignored. Use `FastList:weave` to build the list from separate lists of positions, sizes and colors.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local rects = fastlist.fromTable({
--- 	vec.V2(-0.5, -0.5), vec.V2(0.4, 0.4), vec.V2(1, 0), vec.V2(0, 1), -- a red rectangle
--- })
---
--- function Update(deltaTime: number)
--- 	rects:drawRects()
--- end
--- ```
---
--- # See also
---
--- - `FastList:weave`: build the groups of 4.
--- - `FastList:drawQuads`: draw four-sided shapes.
function FastListImpl.drawRects(self: FastList)
	error("Implemented in native code")
end

--- Draws a filled four-sided shape for each group of 6 elements of the fastlist.
---
--- Each group is: the four corners `p1`, `p2`, `p3` and `p4`, then the color as two vectors `(r, g)` and `(b, a)`,
--- with every channel between 0 and 1. Each shape is drawn like `graphics.drawPolygon({p1, p2, p3, p4}, color)` would.
--- This is a lot faster than calling `graphics.drawPolygon` in a loop. If the length of the list is not a multiple of
--- 6, the last incomplete group is ignored.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
---
--- local quads = fastlist.fromTable({
--- 	vec.V2(-0.5, -0.5), vec.V2(0.5, -0.5), vec.V2(0.3, 0.5), vec.V2(-0.3, 0.5),
--- 	vec.V2(0, 1), vec.V2(0, 1), -- a green shape
--- })
---
--- function Update(deltaTime: number)
--- 	quads:drawQuads()
--- end
--- ```
function FastListImpl.drawQuads(self: FastList)
	error("Implemented in native code")
end

--- Draws an image for each group of 2 elements of the fastlist.
---
--- Each group is: the position, then the size. Each image is drawn like `ImageResource:draw` would. This is a lot
--- faster than calling `ImageResource:draw` in a loop, for example to draw many bullets or particles. If the length
--- of the list is not a multiple of 2, the last incomplete group is ignored. Nothing is drawn while the image is not
--- loaded yet. Use `FastList:weave` to build the list from separate lists of positions and sizes.
---
--- @param image The image to draw.
--- @param color The tint multiplied with the pixels of the image. Defaults to white, which leaves the image as it is.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
--- const loader = require("@vectarine/loader")
---
--- local star = loader.loadImage("images/star.png")
--- local positions = fastlist.fromTable({ vec.V2(-0.5, 0), vec.V2(0.5, 0) })
--- local sizes = fastlist.fromValue(vec.V2(0.1, 0.1), 2)
--- local stars = positions:weave({ sizes })
---
--- function Update(deltaTime: number)
--- 	stars:drawImages(star)
--- end
--- ```
---
--- # See also
---
--- - `FastList:drawImageParts`: draw part of an image.
--- - `FastList:weave`: build the groups.
function FastListImpl.drawImages(self: FastList, image: Image.ImageResource, color: Vec4.Vec4?)
	error("Implemented in native code")
end

--- Draws a part of an image for each group of 6 elements of the fastlist.
---
--- Each group is: the four destination points `p1`, `p2`, `p3` and `p4`, then the position `src_pos` and the size
--- `src_size` of the part of the image to draw. Each part is drawn like
--- `image:drawPart(p1, p2, p3, p4, src_pos, src_size)` would. See `ImageResource:drawPart`. This is a lot faster than
--- calling it in a loop. If the length of the list is not a multiple of 6, the last incomplete group is ignored.
--- Nothing is drawn while the image is not loaded yet.
---
--- @param image The image to draw from.
--- @param color The tint multiplied with the pixels of the image. Defaults to white, which leaves the image as it is.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
--- const loader = require("@vectarine/loader")
---
--- local sheet = loader.loadImage("images/sheet.png")
--- local parts = fastlist.fromTable({
--- 	vec.V2(-0.5, -0.5), vec.V2(0.5, -0.5), vec.V2(0.5, 0.5), vec.V2(-0.5, 0.5),
--- 	vec.V2(0, 0), vec.V2(0.5, 0.5), -- the bottom left quarter of the image
--- })
---
--- function Update(deltaTime: number)
--- 	parts:drawImageParts(sheet)
--- end
--- ```
function FastListImpl.drawImageParts(self: FastList, image: Image.ImageResource, color: Vec4.Vec4?)
	error("Implemented in native code")
end

--- Draws a tile of a tileset for each group of 3 elements of the fastlist.
---
--- Each group is: the tile id in its `x` component (the `y` component is ignored), then the position, then the size.
--- Each tile is drawn like `ImageWithTileset:drawTile` would, with the tile identified by its id. This is a lot faster
--- than calling `ImageWithTileset:drawTile` in a loop, for example to draw a whole map. If the length of the list is
--- not a multiple of 3, the last incomplete group is ignored.
---
--- @param imageWithTileset The image and tileset to draw the tiles from, made with `ImageResource:withTileset`.
--- @param color The tint multiplied with the pixels of the tiles. Defaults to white, which leaves them as they are.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
--- const loader = require("@vectarine/loader")
---
--- local tiles = loader.loadImage("images/tiles.png"):withTileset(loader.loadTileset("tiles.tsj"))
--- local map = fastlist.fromTable({
--- 	vec.V2(1, 0), vec.V2(-0.5, -0.5), vec.V2(0.2, 0.2), -- tile 1
--- 	vec.V2(2, 0), vec.V2(-0.3, -0.5), vec.V2(0.2, 0.2), -- tile 2
--- })
---
--- function Update(deltaTime: number)
--- 	map:drawTiles(tiles)
--- end
--- ```
---
--- # See also
---
--- - `FastList:drawTileQuads`: deform tiles to four points.
function FastListImpl.drawTiles(self: FastList, imageWithTileset: Image.ImageWithTileset, color: Vec4.Vec4?)
	error("Implemented in native code")
end

--- Draws a tile of a tileset for each group of 5 elements of the fastlist, deformed to fit four points.
---
--- Each group is: the tile id in its `x` component (the `y` component is ignored), then the four destination points
--- `p1`, `p2`, `p3` and `p4`. Each tile is drawn like `ImageWithTileset:drawTileQuad` would, with the tile
--- identified by its id. This is a lot faster than calling `ImageWithTileset:drawTileQuad` in a loop. If the length of
--- the list is not a multiple of 5, the last incomplete group is ignored.
---
--- @param imageWithTileset The image and tileset to draw the tiles from, made with `ImageResource:withTileset`.
--- @param color The tint multiplied with the pixels of the tiles. Defaults to white, which leaves them as they are.
---
--- # Example
---
--- ```luau
--- const vec = require("@vectarine/vec")
--- const fastlist = require("@vectarine/fastlist")
--- const loader = require("@vectarine/loader")
---
--- local tiles = loader.loadImage("images/tiles.png"):withTileset(loader.loadTileset("tiles.tsj"))
--- local roof = fastlist.fromTable({
--- 	vec.V2(1, 0), vec.V2(-0.5, -0.5), vec.V2(0.5, -0.5), vec.V2(0.3, 0.5), vec.V2(-0.3, 0.5),
--- })
---
--- function Update(deltaTime: number)
--- 	roof:drawTileQuads(tiles)
--- end
--- ```
function FastListImpl.drawTileQuads(self: FastList, imageWithTileset: Image.ImageWithTileset, color: Vec4.Vec4?)
	error("Implemented in native code")
end

return module
