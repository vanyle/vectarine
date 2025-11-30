local Vec = require("@vectarine/vec")

--- A module for writing high performance code by storing data in a compact way
local module = {}

local FastListImpl = { fast_list = true }
FastListImpl.__index = FastListImpl
export type FastList = typeof(setmetatable({}, FastListImpl))

--- Return a list of Vec2 that are linearly spaced between min and max in a 2D grid
function module.newLinspace(min: Vec.Vec2, max: Vec.Vec2, step: Vec.Vec2): FastList
	error("Not implemented")
end

--- Returns the number of Vec2 in the fast list
function FastListImpl.__len(self: FastList): number
	error("Not implemented")
end

--- Apply a function to each Vec2 in the fast list. You can modify the vector in-place
--- in the function to modify elements of the fast list efficiently
function FastListImpl.forEach(self: FastList, fn: (Vec.Vec2, number) -> ())
	error("Not implemented")
end

--- Process the fast list in groups of `groupSize` Vec2 and return a new fast list (with potentially a different size)
--- groupSize should be a multiple of the length of the fast list
function FastListImpl.group(self: FastList, groupSize: number, fn: ({ Vec.Vec2 }) -> { Vec.Vec2 }): FastList
	error("Not implemented")
end

--- Apply a linear transformation to each Vec2 in the fast list, in-place.
--- Namely, for every Vec2 v in the fast list, v is transformed to (v * scale + translation):cmul(cmul)
--- The `cmul` parameter can be used for rotations or can be kept at (k, 0) to scale by k
function FastListImpl.linearMap(self: FastList, translation: Vec.Vec2, scale: Vec.Vec2, cmul: Vec.Vec2)
	error("Not implemented")
end

--- Apply a function to each pair of Vec2 from the fast list and another fast list
--- The function is called with the Vec2 from the fast list, the Vec2 from the other fast list, and the index of the Vec2 in the fast list
function FastListImpl.mapZip(self: FastList, other: FastList, fn: (Vec.Vec2, Vec.Vec2, number) -> Vec.Vec2)
	error("Not implemented")
end

--- Add other to self, in-place
function FastListImpl.add(self: FastList, other: FastList)
	error("Not implemented")
end

--- Substract other from self, in-place
function FastListImpl.sub(self: FastList, other: FastList)
	error("Not implemented")
end

--- Compute simplex noise for each Vec2 in the fast list and return a new fast list
--- where the Vec2 both contain the noise value.
function FastListImpl.noise(self: FastList): FastList
	error("Not implemented")
end

--- Compute worley noise for each Vec2 in the fast list and return a new fast list
function FastListImpl.worleyNoise(self: FastList): FastList
	error("Not implemented")
end

--- Filter the fast list, keeping only the Vec2 for which the corresponding Vec2 in the mask has a x component greater than the threshold
function FastListImpl.filterGtX(self: FastList, mask: FastList, threshold: number): FastList
	error("Not implemented")
end

--- Return a copy of the fast list
function FastListImpl.copy(self: FastList): FastList
	error("Not implemented")
end

return module
