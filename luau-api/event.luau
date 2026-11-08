const io = require("@vectarine/io")

--- Provides events: named channels that let one part of your game notify others without knowing who listens.
---
--- Create an event with `event.newEvent`, subscribe to it with `Event:on` and fire it with `Event:dispatch`. This
--- keeps code separate: the code that detects that the player was hit does not need to know about the sound, the
--- screen shake or the score. The module also gives access to events sent by the engine, like key presses
--- (`event.getKeyDownEvent`) or text typing (`event.getTextInputEvent`). For polling the keyboard or mouse every frame
--- instead, see `io`.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- local coinEvent: event.Event<number> = event.newEvent("coin")
--- coinEvent:on("score", function(value: number)
--- 	print("Picked a coin worth " .. value)
--- end)
--- coinEvent:dispatch(5) -- prints "Picked a coin worth 5"
--- ```
---
--- # Common mistakes
---
--- - Subscribing with a new subscription name each time the code runs (for example after a reload) adds a new
---   subscriber every time. Reuse the same name to replace the previous callback.
--- - If a callback raises an error, the remaining subscribers of that dispatch are not called.
--- - The order in which subscribers are called is not defined, so do not rely on it.
local module = {}

local EventTypeImpl = {}
EventTypeImpl.__index = EventTypeImpl
--- A named channel that carries values of type `T` from `Event:dispatch` to every subscriber added with `Event:on`.
---
--- You get one from `event.newEvent` or from the functions that return engine events, like `event.getKeyDownEvent`.
--- Events with the same name are the same event, so you can create it in several files.
export type Event<T> = typeof(setmetatable({}, EventTypeImpl)) & { __data_type: T } -- The fake `__data_type` field only carries `T`, so the type checker knows what data the event dispatches.

--- Creates an event with the given name, or returns the existing one if the name is already used.
---
--- The name identifies the event: calling `event.newEvent` twice with the same name gives the same event, with the
--- same subscribers. The type of the data it carries is chosen by you, by annotating the variable. Events are not
--- removed: they live as long as the game runs.
---
--- @param name The name of the event, for example `"hit"`. Use a name that is not shared with an unrelated event.
--- @return The event with that name.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- local hitEvent: event.Event<number> = event.newEvent("hit")
--- hitEvent:on("print", function(damage: number)
--- 	print("Hit with damage: " .. damage)
--- end)
--- hitEvent:dispatch(10) -- prints "Hit with damage: 10"
--- ```
function newEvent<T>(name: string): Event<T>
	error("Implemented in native code")
end
module.newEvent = newEvent

--- Calls every subscriber of the event with the given data.
---
--- Subscribers run one after the other before `Event:dispatch` returns, in no particular order. Dispatching an event
--- without subscribers does nothing. If a subscriber raises an error, the error is propagated to the caller of
--- `Event:dispatch` and the remaining subscribers are not called.
---
--- The list of subscribers is read when the dispatch starts. A subscriber removed during the dispatch (with
--- `SubID:unsubscribe` or `Event:clear`) is still called in that dispatch. A subscriber added during the dispatch is
--- not called until the next one. A subscriber can dispatch an event itself, including the same one.
---
--- @param data The value passed to every subscriber.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- local gameOver: event.Event<string> = event.newEvent("gameOver")
--- gameOver:on("ui", function(reason: string)
--- 	print("Game over: " .. reason)
--- end)
--- gameOver:dispatch("out of lives") -- prints "Game over: out of lives"
--- ```
function EventTypeImpl.dispatch<T>(event: Event<T>, data: T)
	error("Implemented in native code")
end

local SubIDImpl = {}
SubIDImpl.__index = SubIDImpl
--- A subscription identifier, used to stop listening to an event.
---
--- You get one from `Event:on`. Call `SubID:unsubscribe` on it to remove the subscription.
export type SubID = typeof(setmetatable({}, SubIDImpl))

--- Subscribes a function to the event, to be called each time the event is dispatched.
---
--- Each subscription has a name. Subscribing again to the same event with the same name replaces the previous
--- callback instead of adding a second one, so it is safe to run the same setup code twice (for example after a
--- reload). To have several subscribers on one event, use different names. The order in which subscribers are called
--- is not defined.
---
--- @param subscriptionName The name of this subscription, unique for this event.
--- @param callback The function called with the data of each dispatch.
--- @return A `SubID` that can be used to unsubscribe. Subscribing again with the same name returns an identifier
--- for the same subscription.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- local jumped: event.Event<number> = event.newEvent("jumped")
--- jumped:on("sound", function(height: number)
--- 	print("Play the jump sound")
--- end)
--- jumped:on("sound", function(height: number)
--- 	print("Replaced: only this one runs")
--- end)
--- jumped:dispatch(1) -- prints "Replaced: only this one runs"
--- ```
function EventTypeImpl.on<T>(event: Event<T>, subscriptionName: string, callback: (data: T) -> ()): SubID
	error("Implemented in native code")
end

--- Removes every subscriber of the event.
---
--- Dispatching the event afterwards calls nothing until someone subscribes again. Subscribers removed while a
--- dispatch is running are still called in that dispatch.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- local tick: event.Event<number> = event.newEvent("tick")
--- tick:on("log", function(n: number)
--- 	print(n)
--- end)
--- tick:clear()
--- tick:dispatch(1) -- prints nothing
--- ```
function EventTypeImpl.clear<T>(event: Event<T>)
	error("Implemented in native code")
end

--- Removes the subscription, so its callback is not called by later dispatches.
---
--- Does nothing if the subscription was already removed. The name of the subscription can be used again afterwards.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- local tick: event.Event<number> = event.newEvent("tick")
--- local id = tick:on("log", function(n: number)
--- 	print(n)
--- end)
--- tick:dispatch(1) -- prints 1
--- id:unsubscribe()
--- tick:dispatch(2) -- prints nothing
--- ```
function SubIDImpl.unsubscribe(id: SubID)
	error("Implemented in native code")
end

-- #MARK: Default events

--- Returns the event dispatched when a command is sent from the editor console.
---
--- The string is the text of the command. The event is only dispatched while the game runs in the editor.
---
--- @return The console command event.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- event.getConsoleCommandEvent():on("cheats", function(command: string)
--- 	if command == "god" then
--- 		print("God mode enabled")
--- 	end
--- end)
--- ```
function module.getConsoleCommandEvent(): Event<string>
	error("Implemented in native code")
end

--- Returns the event dispatched when a resource finishes loading.
---
--- It is dispatched when a resource is ready to be used, and also when its loading failed. The number is the ID of the
--- resource that was loaded.
---
--- @return The resource loaded event.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- event.getResourceLoadedEvent():on("loading", function(resourceId: number)
--- 	print("Resource loaded: " .. resourceId)
--- end)
--- ```
function module.getResourceLoadedEvent(): Event<number>
	error("Implemented in native code")
end

--- Returns the event dispatched when the player types text.
---
--- The string is the typed text, after modifiers like shift or option are applied (typing `a` with shift gives `"A"`).
--- Keys that do not produce text, like the arrow keys, do not trigger it. Use it to implement text input, and use
--- `event.getKeyDownEvent` for game controls.
---
--- @return The text input event.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- local name = ""
--- event.getTextInputEvent():on("nameField", function(text: string)
--- 	name ..= text
--- 	print(name)
--- end)
--- ```
function module.getTextInputEvent(): Event<string>
	error("Implemented in native code")
end

--- Returns the event dispatched when a keyboard key is released.
---
--- The data is the scancode of the key, as a string like `"Space"` or `"A"` (see `io.Scancode`).
---
--- @return The key up event.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- event.getKeyUpEvent():on("release", function(key)
--- 	print(key .. " released")
--- end)
--- ```
function module.getKeyUpEvent(): Event<io.Scancode>
	error("Implemented in native code")
end

--- Returns the event dispatched when a keyboard key is pressed.
---
--- The data is the scancode of the key, as a string like `"Space"` or `"A"` (see `io.Scancode`). Scancodes are
--- positions on the keyboard, independent of the layout.
---
--- @return The key down event.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- event.getKeyDownEvent():on("jump", function(key)
--- 	if key == "Space" then
--- 		print("Jump!")
--- 	end
--- end)
--- ```
function module.getKeyDownEvent(): Event<io.Scancode>
	error("Implemented in native code")
end

--- Returns the event dispatched when a mouse button is pressed.
---
--- The string is `"left"`, `"right"` or `"middle"`. Any other button gives `"???"`.
---
--- @return The mouse down event.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- event.getMouseDownEvent():on("shoot", function(button: string)
--- 	if button == "left" then
--- 		print("Bang!")
--- 	end
--- end)
--- ```
function module.getMouseDownEvent(): Event<string>
	error("Implemented in native code")
end

--- Returns the event dispatched when a mouse button is released.
---
--- The string is `"left"`, `"right"` or `"middle"`. Any other button gives `"???"`.
---
--- @return The mouse up event.
---
--- # Example
---
--- ```luau
--- const event = require("@vectarine/event")
---
--- event.getMouseUpEvent():on("release", function(button: string)
--- 	print(button .. " released")
--- end)
--- ```
function module.getMouseUpEvent(): Event<string>
	error("Implemented in native code")
end

return module
