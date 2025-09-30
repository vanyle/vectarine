local module = {}

local Vec2Impl = {}
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
export type Vec2Data = {x: number, y: number}

--- Implementation of a 2D vector
export type Vec2 = typeof(setmetatable({} :: Vec2Data, Vec2Impl))

--- Create a new 2d vector
function module.V2(x: number, y: number): Vec2
    return nil :: any
end

function Vec2Impl.scale(a: Vec2, k: number): Vec2
    error("Not implemented")
end

--- Complex multiplication
function Vec2Impl.cmul(a: Vec2, b: Vec2): Vec2
    error("Not implemented")
end

function Vec2Impl.length(v: Vec2): number
    error("Not implemented")
end


return module