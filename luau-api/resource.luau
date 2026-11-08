--[[
Follows the loading of resources, like images, sounds, fonts and texts.

A resource is an external piece of data, stored in a file of your game. You get one by calling a function of the
`loader` module, like `loader.loadImage`. Loading is asynchronous: the function returns right away and the file is read
in the background. Use `Resource:isReady` or `Resource:getStatus` to know when it can be used. You don't need to
require this module: what it describes is available on every resource.

# Example

```luau
const loader = require("@vectarine/loader")

local image = loader.loadImage("images/hero.png")
function Update(deltaTime: number)
	if image:isReady() then
		print("Hero loaded")
	end
end
```

# Common mistakes

- Using a resource right after loading it. It is not ready yet, so drawing or playing it does nothing and a warning is
  printed. Check `Resource:isReady` first.
- Never checking the status. If the file is missing, the resource stays unusable and the only hint is the
  `"Error: ..."` text of `Resource:getStatus`.
]]
local module = {}

local ResourceId = {}
ResourceId.__index = ResourceId

--- Returns the loading status of a resource as a string.
---
--- When a `loader` function is called, the resource starts as not loaded. Once per frame, resources that are not loaded
--- are scheduled for loading and become `"Loading"`. When loading ends, the status becomes `"Loaded"`, or
--- `"Error: <description>"` if something went wrong, for example if the file does not exist.
---
--- Using a resource that is not `"Loaded"` does nothing and prints a warning in the console.
---
--- @return The status: `"Loaded"`, `"Loading"`, `"Error: <description>"`, or `"Not yet loaded"` before loading starts.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local music = loader.loadAudio("audio/theme.ogg")
--- function Update(deltaTime: number)
--- 	if music:getStatus() ~= "Loaded" then
--- 		print(music:getStatus())
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `Resource:isReady`: a simple `true` or `false` check.
--- - `loader`: the functions that create resources.
function ResourceId:getStatus(): string | "Not yet loaded" | "Loading" | "Loaded"
	error("Implemented in native code")
end

--- Returns `true` if the resource is loaded and can be used.
---
--- This is the same as checking that `Resource:getStatus` returns `"Loaded"`. It is `false` while the resource is
--- loading and when loading failed.
---
--- @return True if the status is `"Loaded"`.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local text = loader.loadText("texts/intro.txt")
--- function Update(deltaTime: number)
--- 	if text:isReady() then
--- 		print(text:getText())
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `Resource:getStatus`: to see why a resource is not ready.
function ResourceId:isReady(): boolean
	error("Implemented in native code")
end

--- Returns the identifier of a resource.
---
--- The identifier is a number that is unique to the resource. It stays the same when the game is hot-reloaded, but not
--- when it is restarted.
---
--- @return The identifier.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local image = loader.loadImage("images/hero.png")
--- print(image:getId())
--- ```
function ResourceId:getId(): number
	error("Implemented in native code")
end

--- Any resource returned by a `loader` function.
---
--- Use it to follow the loading of a file, for example with `Resource:isReady`.
export type Resource = typeof(setmetatable({}, ResourceId))

local ScriptResourceImpl = { type = "script" }
ScriptResourceImpl.__index = ScriptResourceImpl
--- A Luau script loaded with `loader.loadScript`.
---
--- Like every `Resource`, it tells you when the script is loaded, with `Resource:isReady`.
export type ScriptResource = typeof(setmetatable({}, ScriptResourceImpl)) & Resource

local TextResourceImpl = { type = "text" }
TextResourceImpl.__index = TextResourceImpl
--- A text file loaded with `loader.loadText`.
---
--- Once it is ready, read its content with `TextResource:getText`.
export type TextResource = typeof(setmetatable({}, TextResourceImpl)) & Resource

--- Returns the content of a text resource.
---
--- The file is read as UTF-8 text. Invalid bytes are replaced by the character U+FFFD.
---
--- @return The content of the file, or `nil` if the resource is not loaded yet or failed to load.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local dialog = loader.loadText("texts/dialog.txt")
--- function Update(deltaTime: number)
--- 	local content = dialog:getText()
--- 	if content then
--- 		print(content)
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `loader.loadText`: to load a text file.
function TextResourceImpl:getText(): string?
	error("Implemented in native code")
end

return module
