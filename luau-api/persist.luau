--[[
Keeps values across hot-reloads and across game restarts.

Use `persist.save` and `persist.load` for data the player expects to find again, like a high score or settings.
Use `persist.onReload` and `persist.onReloadWithProvider` while developing, to keep the state of your game when you edit
a script and the editor reloads it. See the `loader` module for another way to keep script state across reloads.

# Example

```luau
const persist = require("@vectarine/persist")

local best = persist.load("best_score") or 0
persist.save("best_score", best + 1)
```

# Common mistakes

- Reading a value right after `persist.save`. Saving can take some time, so the new value may not be there yet.
- Saving something that is not plain data, like a function, a `Vec2` or a resource. Values go through JSON, so
  `persist.save` raises an error for such a value. Save the numbers inside instead, for example `{ x = v.x, y = v.y }`.
- Calling `persist.save` every frame. Save at meaningful moments, like the end of a level.
- Using `persist.onReload` to keep data between restarts. It only survives hot-reloads. Use `persist.save`.
]]
local module = {}

--- Returns a value that survives hot-reloads, creating it from a default value the first time.
---
--- The value is stored in a global variable named `globalName`, so it can be inspected in the watcher. The first time
--- this function is called, the global is set to `defaultValue` and `defaultValue` is returned. After a hot-reload, the
--- global already holds a value, and that value is returned instead of `defaultValue`. Changes to the value itself are
--- kept only if it is a table: assigning a new number or string to your local variable does not update the global.
---
--- If `defaultValue` is expensive to create, use `persist.onReloadWithProvider`.
---
--- @param defaultValue The value to use when there is no value yet.
--- @param globalName The name of the global variable that holds the value. Use a different name for each value.
--- @return The stored value, or `defaultValue` the first time.
---
--- # Example
---
--- ```luau
--- const persist = require("@vectarine/persist")
---
--- local state = persist.onReload({ score = 0 }, "gameState")
--- function Update(deltaTime: number)
--- 	state.score += deltaTime
--- end
--- ```
---
--- # See also
---
--- - `persist.onReloadWithProvider`: when the default value is expensive to create.
--- - `loader.init`: to keep a whole script table.
function module.onReload<T>(defaultValue: T, globalName: string): T
	error("Implemented in native code")
end

--- Returns a value that survives hot-reloads, creating it with a function the first time.
---
--- It works like `persist.onReload`, but `provider` is called only when there is no stored value yet. Use it when the
--- default value is expensive to create, so that it is not rebuilt on every hot-reload.
---
--- @param provider A function that returns the initial value. It is not called after a hot-reload.
--- @param globalName The name of the global variable that holds the value. Use a different name for each value.
--- @return The stored value, or the value returned by `provider` the first time.
---
--- # Errors
---
--- Raises an error if `provider` raises one.
---
--- # Example
---
--- ```luau
--- const persist = require("@vectarine/persist")
---
--- local grid = persist.onReloadWithProvider(function()
--- 	local cells = {}
--- 	for i = 1, 10000 do
--- 		cells[i] = 0
--- 	end
--- 	return cells
--- end, "gridCells")
--- ```
function module.onReloadWithProvider<T>(provider: () -> T, globalName: string): T
	error("Implemented in native code")
end

--- Saves a value so that it is still there after the game restarts.
---
--- The value is stored under `name` and read back with `persist.load`. Saving again with the same name replaces the
--- previous value. The value is converted to JSON, so use numbers, strings, booleans and tables of those. Functions and
--- other objects, like vectors and resources, cannot be saved.
---
--- Saving can take some time, so calling `persist.save` followed by `persist.load` might not give the expected result.
--- Don't call it every frame.
---
--- @param name The name of the saved value. Use only letters, numbers, underscores and dots.
--- @param value The value to save.
---
--- # Errors
---
--- Raises an error if the value, or something inside it, cannot be converted to JSON. Nothing is saved in that case,
--- and the previous value saved under `name` is kept.
---
--- # Example
---
--- ```luau
--- const persist = require("@vectarine/persist")
---
--- persist.save("settings", { volume = 0.8, fullscreen = false })
--- ```
---
--- # See also
---
--- - `persist.load`: to read the value back.
function module.save(name: string, value: any)
	error("Implemented in native code")
end

--- Loads a value saved with `persist.save`.
---
--- @param name The name used when saving.
--- @return The saved value, or `nil` if nothing was saved under this name.
---
--- # Errors
---
--- Raises an error if the saved data cannot be read back, for example if the file was edited by hand.
---
--- # Example
---
--- ```luau
--- const persist = require("@vectarine/persist")
---
--- local settings = persist.load("settings")
--- local volume = if settings then settings.volume else 1
--- print(volume)
--- ```
---
--- # See also
---
--- - `persist.save`: to save a value.
function module.load(name: string): any?
	error("Implemented in native code")
end

return module
