--- Debugging utilities
---
--- This module contains functions that help with debugging your game.
--- Most of these functions do nothing in a production build.
local module = {}

--- Add a value to the watcher in the editor
--- Watched values can be inspected and edited live.
function module.watch(value: any, name: string?): any
	error("Not implemented")
end

--- Print something to the editor console.
---
--- Prints to the browser console on the web for when using debug builds.
--- If you are printing inside Update(), consider using fprint.
function module.print(...: any): ()
	error("Not implemented")
end

--- Print something to the editor frame message console
----
--- Does nothing when used in the runtime.
--- The message is cleared on the next frame.
function module.fprint(...: any): ()
	error("Not implemented")
end

return module
