local Vec = require('@vectarine/vec')

local module = {}

--- Checks if a key is pressed
--- For example, `IsKeyDown("space")`. If the name is invalid, always return false.
--- You can use 'GetKeysDown' to find the name of keys
function module.isKeyDown(keycode: string): boolean
    error("Not implemented")
end

--- Returns a list of key that are currently pressed
function module.getKeysDown(): {string}
    error("Not implemented")
end

--- Print something to the editor console.
---
--- Prints to the browser console on the web for when using debug builds.
--- If you are printing inside Update(), consider using fprint.
function module.print(...: any): ()
    error("Not implemented")
end

--- Print something to the editor console
----
--- Does nothing when used in the runtime.
--- The message is cleared on the next frame.
function module.fprint(...: any): ()
    error("Not implemented")
end

--- Get the current mouse position
function module.getMouse(): Vec.Vec2
    error("Not implemented")
end

--- Get the current mouse state
--- @return {is_left_down: boolean, is_right_down: boolean}
function module.getMouseState(): { is_left_down: boolean; is_right_down: boolean }
    error("Not implemented")
end

--- Get the current window size (in px)
function module.getWindowSize(): Vec.Vec2
    error("Not implemented")
end

--- Get the current screen size (in px)
function module.getScreenSize(): Vec.Vec2
    error("Not implemented")
end

--- Sets the window size
--- Does nothing on the web
function module.setWindowSize(width: number, height: number): ()
    error("Not implemented")
end

--- Set if the window is resizeable
function module.setResizeable(resizeable: boolean): ()
    error("Not implemented")
end

--- Set the window to fullscreen or windowed mode
--- Before calling this function,module. you can set the window size to the size of the screen to avoid black borders.
--- On the web, the game is usually embedded in an iframe / player which already provides a fullscreen button, so this
--- function module.does nothing.
function module.setFullscreen(fullscreen: boolean): ()
    error("Not implemented")
end

--- Return a friendly string representation of arg
function module.toString(arg: any): string
    error("Not implemented")
end

return module