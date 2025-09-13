---@meta

--- Checks if a key is pressed
--- For example, `IsKeyDown("space")`. If the name is invalid, always return false.
--- You can use 'GetKeysDown' to find the name of keys
--- @param keycode string
--- @return boolean
function IsKeyDown(keycode) end

--- Returns a list of key that are currently pressed
--- @return string[]
function GetKeysDown() end

--- Print something to the editor console
--- If you are printing inside Update(), consider using fprint.
--- @param msg any The thing to print
function dprint(msg) end

--- Print something to the editor console
--- The message is cleared on the next frame
--- @param msg any The thing to print
function fprint(msg) end
