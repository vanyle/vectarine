--[[
Prints messages and measures time to help you debug your game.

Use `debug.print` for messages you want to keep in the console, and `debug.fprint` for values that change every frame,
like the position of the player. Use `debug.timed` to find which part of your game is slow. For general output, the
standard `print` also works.

# Example

```luau
const debug = require("@vectarine/debug")

function Update(deltaTime: number)
	debug.fprint("FPS: ", 1 / deltaTime)
end
```

# Common mistakes

- Calling `debug.print` inside `Update`. It adds a line to the console every frame. Use `debug.fprint`.
- Expecting spaces between the arguments. They are joined with nothing between them.
]]
local module = {}

--- Prints a message to the editor console.
---
--- All the arguments are converted to text and joined together with nothing between them. When the game runs in a
--- browser with a debug build, the message goes to the browser console. Inside `Update`, the console would fill up
--- with one line per frame: use `debug.fprint` instead.
---
--- @param ... The values to print. Add spaces yourself if you need them.
---
--- # Example
---
--- ```luau
--- const debug = require("@vectarine/debug")
---
--- debug.print("Level loaded: ", 3)
--- ```
---
--- # See also
---
--- - `debug.fprint`: for values that change every frame.
function module.print(...: any): ()
	error("Implemented in native code")
end

--- Prints a message in the frame message area of the editor.
---
--- The message is cleared on the next frame, so call it every frame to keep it visible. It does not fill the console.
--- It does nothing when the game runs outside the editor. All the arguments are converted to text and joined together
--- with nothing between them.
---
--- @param ... The values to print. Add spaces yourself if you need them.
---
--- # Example
---
--- ```luau
--- const debug = require("@vectarine/debug")
---
--- local x = 0
--- function Update(deltaTime: number)
--- 	x += deltaTime
--- 	debug.fprint("x = ", x)
--- end
--- ```
---
--- # See also
---
--- - `debug.print`: for messages that stay in the console.
function module.fprint(...: any): ()
	error("Implemented in native code")
end

--- Measures how long a function takes to run and records it in the editor metrics.
---
--- The function `callback` is called right away, once. The time it takes is recorded under `name`. If you call it
--- several times with the same name during one frame, the durations are added together.
---
--- @param name The name of the measure, as shown in the metrics.
--- @param callback The function to run and measure.
---
--- # Errors
---
--- Raises an error if `callback` raises one. The duration is not recorded in that case.
---
--- # Example
---
--- ```luau
--- const debug = require("@vectarine/debug")
---
--- function Update(deltaTime: number)
--- 	debug.timed("physics", function()
--- 		-- slow code here
--- 	end)
--- end
--- ```
function module.timed(name: string, callback: () -> ()): ()
	error("Implemented in native code")
end

return module
