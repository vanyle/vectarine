
export type ResourceId = number

local module = {}

--- Get the status of a resource.
---
--- When loadResource is called, the resource is created in an Unloaded state.
--- Once per frame, `"Unloaded"` resources are scheduled for loading, and their state becomes `"Loading"`.
--- When the loading is finished, the state becomes `"Loaded"` or `"Error: Description of the error"` if something went wrong.
--- When a resource which is not `"Loaded"` is used, nothing will happen and a warning will be printed in the console.
function module.getResourceStatus(id: ResourceId): string | "NotLoaded" | "Loading" | "Loaded"
    error("Not implemented")
end

--- Check if a resource is ready to be used.
--- This is the same as checking if getResourceStatus returns `"Loaded"`.
function module.isResourceReady(id: ResourceId): boolean
    error("Not implemented")
end

---MARK: Script

export type ScriptResource = ResourceId

--- Loads and runs a Lua file at the given path
--- @param path string For example scripts/monster.lua
function module.loadScript(path: string): ScriptResource
    error("Not implemented")
end

---MARK: Image

export type ImageResource = ResourceId

--- Load an image from a path inside assets
--- @param path string
--- @return ImageResource
function module.loadImage(path: string): ImageResource
    error("Not implemented")
end

---MARK: Font

export type FontResource = ResourceId

--- Load a font from a path inside assets
--- @param path string
--- @return FontResource
--- @nodiscard
function module.loadFont(path: string): FontResource
    error("Not implemented")
end

return module
