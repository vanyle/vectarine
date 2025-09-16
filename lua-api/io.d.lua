---@meta

--- Checks if a key is pressed
--- For example, `IsKeyDown("space")`. If the name is invalid, always return false.
--- You can use 'GetKeysDown' to find the name of keys
--- @param keycode string
--- @return boolean
function isKeyDown(keycode) end

--- Returns a list of key that are currently pressed
--- @return string[]
function getKeysDown() end

--- Print something to the editor console
--- Prints to the browser console on the web for when using debug builds.
--- If you are printing inside Update(), consider using fprint.
--- @param msg any The thing to print
function dprint(msg) end

--- Print something to the editor console
--- Does nothing when used in the runtime
--- The message is cleared on the next frame
--- @param msg any The thing to print
function fprint(msg) end

--- Get the current mouse position
--- @return { x: number, y: number }
function mouse() end

--- Get the current window size (in px)
--- @return { x: number, y: number }
function windowSize() end

--- Get the current screen size (in px)
--- @return { x: number, y: number }
function screenSize() end

--- Sets the window size
--- Does nothing on the web
--- @param width number
--- @param height number
function setWindowSize(width, height) end

--- Set if the window is resizeable
--- @param resizeable boolean
function setResizeable(resizeable) end

--- Return a friendly string representation of arg
--- @param arg any
--- @return string
function toString(arg) end
