
local ResourceId = {}
export type ResourceId = typeof(setmetatable({}, ResourceId))

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

export type ScriptResource = {type: "script"} & ResourceId

--- Loads and runs a Lua file at the given path.
--- If the script returns a table, it will be merged with the `results` table if provided.
--- The return value is a ScriptResource
--- @param path string For example scripts/monster.lua
function module.loadScript<T>(path: string, type_hint: T): (ScriptResource, T)
    error("Not implemented")
end

---MARK: Image

export type ImageResource = {type: "image"} & ResourceId

--- Load an image from a path inside assets
--- @param path string
--- @return ImageResource
function module.loadImage(path: string): ImageResource
    error("Not implemented")
end

---MARK: Font

export type FontResource = {type: "font"} & ResourceId

--- Load a font from a path inside assets
--- @param path string
--- @return FontResource
--- @nodiscard
function module.loadFont(path: string): FontResource
    error("Not implemented")
end

---MARK: Shader

export type ShaderResource = {type: "shader"} & ResourceId

--- Load a shader from a path inside assets
--- @param path string
--- @return ShaderResource
--- @nodiscard
function module.loadShader(path: string): ShaderResource
    error("Not implemented")
end

return module
