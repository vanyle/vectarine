---@meta


--- A global handle which is shared across a Lua context.
Global = {}


-- --- Loads and runs a Lua file at the given path
-- --- @param path string For example assets/scripts/monster.lua
-- function loadCode(path) end

--- @alias ImageResource `ImageResource`

--- Load an image from a path inside assets
--- @param path string
--- @return ImageResource
--- @nodiscard
function loadImage(path) end

--- @alias FontResource `FontResource`

--- Load a font from a path inside assets
--- @param path string
--- @return FontResource
--- @nodiscard
function loadFont(path) end
