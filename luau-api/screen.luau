--[[
Screens are deprecated, use the new ui module instead.
]]
--
local module = {}

local ScreenImpl = { screen = true }
ScreenImpl.__index = ScreenImpl
export type Screen = typeof(setmetatable({}, ScreenImpl))

--- Makes a new screen with the given name.
@deprecated
function module.newScreen(screen_name: string, draw_function: () -> ()): Screen
	error("Implemented in native code")
end

--- Represents a transition between two screens.
--- You can use a predefined transition style, or provide your own function.
--- The function will be called every frame during the transition with the old screen, the new screen, and a progress value between 0 and 1.
---
--- `duration` is a number in seconds.
export type ScreenTransition = {
	duration: number,
	transition_style: "fade"
		| "slide_up"
		| "slide_down"
		| "toon"
		| (old_screen: Screen, new_screen: Screen, progress: number) -> (),
}

--- Sets the current screen, with an optional transition.
--- If this is the first screen, it will be set immediately.
@deprecated
function module.setCurrentScreen(screen: Screen, transition: ScreenTransition?)
	error("Implemented in native code")
end

@deprecated
function module.getCurrentScreen(): Screen?
	error("Implemented in native code")
end

@deprecated
function module.drawCurrentScreen()
	error("Implemented in native code")
end

@deprecated
function ScreenImpl.name(self: Screen): string
	error("Implemented in native code")
end

@deprecated
function ScreenImpl.draw(self: Screen)
	error("Implemented in native code")
end

return module
