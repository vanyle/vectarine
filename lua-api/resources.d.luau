
--- A global handle which is shared across a Lua context.
declare Global: {[string]: any}

declare class ResourceId
end

--- Get the status of a resource.
---
--- When loadResource is called, the resource is created in an Unloaded state.
--- Once per frame, `"Unloaded"` resources are scheduled for loading, and their state becomes `"Loading"`.
--- When the loading is finished, the state becomes `"Loaded"` or `"Error: Description of the error"` if something went wrong.
--- When a resource which is not `"Loaded"` is used, nothing will happen and a warning will be printed in the console.
declare function getResourceStatus(id: ResourceId): string | "NotLoaded" | "Loading" | "Loaded"

--- Check if a resource is ready to be used.
--- This is the same as checking if getResourceStatus returns `"Loaded"`.
declare function isResourceReady(id: ResourceId): boolean

---MARK: Script

declare class ScriptResource extends ResourceId
end

--- Loads and runs a Lua file at the given path
--- @param path string For example scripts/monster.lua
declare function loadScript(path: string): ScriptResource

---MARK: Image

declare class ImageResource extends ResourceId
end

--- Load an image from a path inside assets
--- @param path string
--- @return ImageResource
--- @nodiscard
declare function loadImage(path: string): ImageResource

---MARK: Font

declare class FontResource extends ResourceId
end

--- Load a font from a path inside assets
--- @param path string
--- @return FontResource
--- @nodiscard
declare function loadFont(path: string): FontResource
