--- Checks if a key is pressed
--- For example, `IsKeyDown("space")`. If the name is invalid, always return false.
--- You can use 'GetKeysDown' to find the name of keys
declare function isKeyDown(keycode: string): boolean

--- Returns a list of key that are currently pressed
declare function getKeysDown(): {string}

--- Print something to the editor console
--- Prints to the browser console on the web for when using debug builds.
--- If you are printing inside Update(), consider using fprint.
declare function dprint(...: any): ()

--- Print something to the editor console
--- Does nothing when used in the runtime
--- The message is cleared on the next frame
declare function fprint(...: any): ()

--- Get the current mouse position
declare function getMouse(): Vec2

--- Get the current mouse state
--- @return {is_left_down: boolean, is_right_down: boolean}
declare function getMouseState(): { is_left_down: boolean; is_right_down: boolean }

--- Get the current window size (in px)
declare function getWindowSize(): Vec2

--- Get the current screen size (in px)
declare function getScreenSize(): Vec2

--- Sets the window size
--- Does nothing on the web
declare function setWindowSize(width: number, height: number): ()

--- Set if the window is resizeable
declare function setResizeable(resizeable: boolean): ()

--- Set the window to fullscreen or windowed mode
--- Before calling this function, you can set the window size to the size of the screen to avoid black borders.
--- On the web, the game is usually embedded in an iframe / player which already provides a fullscreen button, so this
--- function does nothing.
declare function setFullscreen(fullscreen: boolean): ()

--- Return a friendly string representation of arg
declare function toString(arg: any): string
