--- @meta _

--- @class Vec2
--- @operator add(Vec2): Vec2
--- @operator sub(Vec2): Vec2
--- @operator mul(number): Vec2
--- @field x number
--- @field y number
local Vec2 = {}

--- Create a new 2d vector
--- Represents a 2d vector
--- @param x number
--- @param y number
--- @return Vec2
function V2(x, y) end

--- Scale a vector by a number
--- @param v Vec2
--- @param k number
--- @return Vec2
--- @nodiscard
function Vec2.scale(v, k) end

--- Complex multiplication of two vectors
--- @param a Vec2
--- @param b Vec2
--- @return Vec2
--- @nodiscard
function Vec2.cmul(a, b) end

--- Get the length of the vector
--- @param v Vec2
--- @return number
--- @nodiscard
function Vec2.length(v) end
