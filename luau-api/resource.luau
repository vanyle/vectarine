--- Functions common to all resources
---
--- A resource is an external piece of data.
--- Use the loader module to load resources.
--- You shouldn't need to import this module.
local module = {}

local ResourceId = {}
ResourceId.__index = ResourceId

--- Get the status of a resource.
---
--- When loadResource is called, the resource is created in an Unloaded state.
--- Once per frame, `"Unloaded"` resources are scheduled for loading, and their state becomes `"Loading"`.
--- When the loading is finished, the state becomes `"Loaded"` or `"Error: Description of the error"` if something went wrong.
--- When a resource which is not `"Loaded"` is used, nothing will happen and a warning will be printed in the console.
function ResourceId.getStatus(id: Resource): string | "NotLoaded" | "Loading" | "Loaded"
	error("Not implemented")
end

--- Check if a resource is ready to be used.
--- This is the same as checking if getResourceStatus returns `"Loaded"`.
function ResourceId.isReady(id: Resource): boolean
	error("Not implemented")
end

export type Resource = typeof(setmetatable({}, ResourceId))

local ScriptResourceImpl = { type = "script" }
ScriptResourceImpl.__index = ScriptResourceImpl
export type ScriptResource = typeof(setmetatable({}, ScriptResourceImpl)) & Resource

return module
