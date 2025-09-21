local module = {} -- This is where all our exports will go
local my_value = 4

-- add_things is inside module, it gets exported
function module.add_things(a: number, b: number): number
    -- Notice that despite my_value not being exported, it can be used inside exported functions!
    return a + b + my_value
end

export type myType = "abc" | "def"

Global.helper = module -- export the module table to Global
return module -- return for the module for typing
