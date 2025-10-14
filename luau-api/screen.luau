--[[
Screens are parts of your game like a menu, a pause, an inventory or the actual game
You can transition between screens, and each screen can have its own logic and rendering.

You can use screens to better organize your game code.

If you do, you need to call 'drawCurrentScreen' in your Update function to draw the current screen.

Tip: you can draw a screen on a canvas for debugging if needed.
]]--
local module = {}

local ScreenImpl = {screen=true}
ScreenImpl.__index = ScreenImpl
export type Screen = typeof(setmetatable({}, ScreenImpl))

--- Makes a new screen with the given name.
function module.newScreen(screen_name: string, draw_function: () -> ()): Screen
    error("Not implemented")
end

--- Represents a transition between two screens.
--- You can use a predefined transition style, or provide your own function.
--- The function will be called every frame during the transition with the old screen, the new screen, and a progress value between 0 and 1.
---
--- `duration` is a number in seconds.
export type ScreenTransition = {
    duration: number,
    transition_style: "fade" | "slide_up" | "slide_down" | "toon" | (old_screen: Screen, new_screen: Screen, progress: number) -> ()
}

--- Sets the current screen, with an optional transition.
--- If this is the first screen, it will be set immediately.
function module.setCurrentScreen(screen: Screen, transition: ScreenTransition?)
    error("Not implemented")
end

function module.getCurrentScreen(): Screen?
    error("Not implemented")
end

function module.drawCurrentScreen()
    error("Not implemented")
end

function ScreenImpl.name(self: Screen): string
    error("Not implemented")
end

function ScreenImpl.draw(self: Screen)
    error("Not implemented")
end

return module
