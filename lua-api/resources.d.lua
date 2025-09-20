---@meta


--- A global handle which is shared across a Lua context.
Global = {}


-- --- Loads and runs a Lua file at the given path
-- --- @param path string For example assets/scripts/monster.lua
-- function loadCode(path) end

--- @class ResourceId
local ResourceId = {}

--- @class ImageResource: ResourceId
local ImageResource = {}


--- Load an image from a path inside assets
--- @param path string
--- @return ImageResource
--- @nodiscard
function loadImage(path) end

--- @class FontResource: ResourceId
local FontResource = {}

--- Load a font from a path inside assets
--- @param path string
--- @return FontResource
--- @nodiscard
function loadFont(path) end

--- Get the status of a resource.
---
--- When loadResource is called, the resource is created in an Unloaded state.
--- Once per frame, `"Unloaded"` resources are scheduled for loading, and their state becomes `"Loading"`.
--- When the loading is finished, the state becomes `"Loaded"` or `"Error: Description of the error"` if something went wrong.
---
--- When a resource which is not `"Loaded"` is used, nothing will happen and a warning will be printed in the console.
--- @param id ResourceId
--- @return string | "NotLoaded" | "Loading" | "Loaded"
function getResourceStatus(id) end

--- Check if a resource is ready to be used.
--- This is the same as checking if getResourceStatus returns `"Loaded"`.
--- @param id ResourceId
--- @return boolean
function isResourceReady(id) end
