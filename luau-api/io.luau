local Vec = require("@vectarine/vec")

local module = {}

-- This list of scancodes comes from here: https://github.com/libsdl-org/SDL/blob/main/src/events/SDL_keymap.c#L749
--- List of all available scancodes
type Scancode =
	"A"
	| "B"
	| "C"
	| "D"
	| "E"
	| "F"
	| "G"
	| "H"
	| "I"
	| "J"
	| "K"
	| "L"
	| "M"
	| "N"
	| "O"
	| "P"
	| "Q"
	| "R"
	| "S"
	| "T"
	| "U"
	| "V"
	| "W"
	| "X"
	| "Y"
	| "Z"
	| "1"
	| "2"
	| "3"
	| "4"
	| "5"
	| "6"
	| "7"
	| "8"
	| "9"
	| "0"
	| "Return"
	| "Escape"
	| "Backspace"
	| "Tab"
	| "Space"
	| "-"
	| "="
	| "["
	| "]"
	| "\\"
	| "#"
	| ";"
	| "'"
	| "`"
	| ","
	| "."
	| "/"
	| "CapsLock"
	| "F1"
	| "F2"
	| "F3"
	| "F4"
	| "F5"
	| "F6"
	| "F7"
	| "F8"
	| "F9"
	| "F10"
	| "F11"
	| "F12"
	| "PrintScreen"
	| "ScrollLock"
	| "Pause"
	| "Insert"
	| "Home"
	| "PageUp"
	| "Delete"
	| "End"
	| "PageDown"
	| "Right"
	| "Left"
	| "Down"
	| "Up"
	| "Numlock"
	| "Keypad /"
	| "Keypad *"
	| "Keypad -"
	| "Keypad +"
	| "Keypad Enter"
	| "Keypad 1"
	| "Keypad 2"
	| "Keypad 3"
	| "Keypad 4"
	| "Keypad 5"
	| "Keypad 6"
	| "Keypad 7"
	| "Keypad 8"
	| "Keypad 9"
	| "Keypad 0"
	| "Keypad ."
	| "Application"
	| "Power"
	| "Keypad ="
	| "F13"
	| "F14"
	| "F15"
	| "F16"
	| "F17"
	| "F18"
	| "F19"
	| "F20"
	| "F21"
	| "F22"
	| "F23"
	| "F24"
	| "Execute"
	| "Help"
	| "Menu"
	| "Select"
	| "Stop"
	| "Again"
	| "Undo"
	| "Cut"
	| "Copy"
	| "Paste"
	| "Find"
	| "Mute"
	| "VolumeUp"
	| "VolumeDown"
	| "Keypad ,"
	| "Keypad = (AS400)"
	| "AltErase"
	| "SysReq"
	| "Cancel"
	| "Clear"
	| "Prior"
	| "Return"
	| "Separator"
	| "Out"
	| "Oper"
	| "Clear / Again"
	| "CrSel"
	| "ExSel"
	| "Keypad 00"
	| "Keypad 000"
	| "ThousandsSeparator"
	| "DecimalSeparator"
	| "CurrencyUnit"
	| "CurrencySubUnit"
	| "Keypad ("
	| "Keypad )"
	| "Keypad {"
	| "Keypad }"
	| "Keypad Tab"
	| "Keypad Backspace"
	| "Keypad A"
	| "Keypad B"
	| "Keypad C"
	| "Keypad D"
	| "Keypad E"
	| "Keypad F"
	| "Keypad XOR"
	| "Keypad ^"
	| "Keypad %"
	| "Keypad <"
	| "Keypad >"
	| "Keypad &"
	| "Keypad &&"
	| "Keypad |"
	| "Keypad ||"
	| "Keypad :"
	| "Keypad #"
	| "Keypad Space"
	| "Keypad @"
	| "Keypad !"
	| "Keypad MemStore"
	| "Keypad MemRecall"
	| "Keypad MemClear"
	| "Keypad MemAdd"
	| "Keypad MemSubtract"
	| "Keypad MemMultiply"
	| "Keypad MemDivide"
	| "Keypad +/-"
	| "Keypad Clear"
	| "Keypad ClearEntry"
	| "Keypad Binary"
	| "Keypad Octal"
	| "Keypad Decimal"
	| "Keypad Hexadecimal"
	| "Left Ctrl"
	| "Left Shift"
	| "Left Alt"
	| "Left GUI"
	| "Right Ctrl"
	| "Right Shift"
	| "Right Alt"
	| "Right GUI"
	| "ModeSwitch"
	| "AudioNext"
	| "AudioPrev"
	| "AudioStop"
	| "AudioPlay"
	| "AudioMute"
	| "MediaSelect"
	| "WWW"
	| "Mail"
	| "Calculator"
	| "Computer"
	| "AC Search"
	| "AC Home"
	| "AC Back"
	| "AC Forward"
	| "AC Stop"
	| "AC Refresh"
	| "AC Bookmarks"
	| "BrightnessDown"
	| "BrightnessUp"
	| "DisplaySwitch"
	| "KBDIllumToggle"
	| "KBDIllumDown"
	| "KBDIllumUp"
	| "Eject"
	| "Sleep"

--- Checks if a key is pressed
--- For example, `IsKeyDown("Space")`. If the name is invalid, always return false.
--- You can use 'GetKeysDown' to find the name of keys
function module.isKeyDown(keycode: Scancode): boolean
	error("Not implemented")
end

--- Checks if a key was just pressed (this frame)
--- The keys are independent from the keyboard layout, so "Q" is always the top-left letter, even on AZERTY layouts.
--- This is the function you need to map button presses to actions, but not for typing text.
--- For example, `IsKeyJustPressed("space")`. If the name is invalid, always return false.
--- You can use 'GetKeysDown' to find the name of keys
function module.isKeyJustPressed(keycode: Scancode): boolean
	error("Not implemented")
end

--- Returns a list of key that are currently pressed
function module.getKeysDown(): { Scancode }
	error("Not implemented")
end

--- Get the current mouse position in OpenGL coordinates
function module.getMouse(): Vec.Vec2
	error("Not implemented")
end

--- Get the current mouse state
--- @return {is_left_down: boolean, is_right_down: boolean}
function module.getMouseState(): { is_left_down: boolean, is_right_down: boolean }
	error("Not implemented")
end

--- Get the current window size (in px)
function module.getWindowSize(): Vec.Vec2
	error("Not implemented")
end

--- Center the window on the screen
--- Does nothing on the web
function module.centerWindow(): ()
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

--- Set the window title
function module.setWindowTitle(title: string): ()
	error("Not implemented")
end

--- Returns true if the window is currently minimized
--- When this is the case, Vectarine will run a lot slower to save battery life and avoid draining the CPU.
function module.isWindowMinimized(): boolean
	error("Not implemented")
end

--- Set the window to fullscreen, windowed, or "desktop fullscreen" (also called fake fullscreen) mode
--- true means "fullscreen", false means "windowed".
--- Before calling this function, you can set the window size to the size of the screen to avoid black borders.
--- On the web, the game is usually embedded in an iframe / player which already provides a fullscreen button, so this
--- function does nothing.
function module.setFullscreen(fullscreen: true | false | "fullscreen" | "windowed" | "desktop"): ()
	error("Not implemented")
end

--- Return a friendly string representation of arg
function module.toString(arg: any): string
	error("Not implemented")
end

return module
