
--- Create a new 2d vector
declare function V2(x: number, y: number): Vec2

--- A 2D vector with x and y components.
declare class Vec2
    x: number
    y: number

    --- Scale a vector by a number
    function scale(self, k: number): Vec2

    --- Complex multiplication of two vectors
    function cmul(self, b: Vec2): Vec2

    --- Get the length of the vector
    function length(self): number
end

--- A 2D vector with x and y components.
export type Vec2 = Vec2