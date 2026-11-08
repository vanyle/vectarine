local Vec = require("@vectarine/vec")

--[[
Reads the keyboard, the mouse, touch screens and controls the game window.

Call these functions from `Update` to react to the player. Input functions give the state of the current frame:
a key that is held stays down for many frames, while `io.isKeyJustPressed` is `true` for one frame only. Functions that
change the window, like `io.setFullscreen`, ask the engine for the change, which is applied after the current frame. For gamepads, see the `gamepad` module.

# Example

```luau
const io = require("@vectarine/io")

local x = 0
function Update(deltaTime: number)
	if io.isKeyDown("Right") then
		x += deltaTime
	end
end
```

# Common mistakes

- Using `io.isKeyDown` for a one-time action like jumping. It is `true` on every frame the key is held, so the action
  repeats. Use `io.isKeyJustPressed`.
- Using key functions to read typed text. Keys are physical positions, so they ignore the keyboard layout. Use
  `io.getTextInput`.
- Expecting a window change to apply immediately. `io.setWindowSize`, `io.setFullscreen` and `io.centerWindow` only
  make a request that the engine applies after the current frame.
]]
local module = {}

-- This list of scancodes comes from here: https://github.com/libsdl-org/SDL/blob/main/src/events/SDL_keymap.c#L749
--- The name of a physical key on the keyboard, like `"A"`, `"Space"`, `"Left Shift"` or `"F5"`.
---
--- A scancode is a location on the keyboard, independent of the keyboard layout. `"Q"` is always the key at the
--- position of Q on a QWERTY keyboard, even on an AZERTY keyboard where it types A. Scancodes are the names accepted
--- by `io.isKeyDown`, `io.isKeyJustPressed` and `io.getKeyName`, and returned by `io.getKeysDown`.
export type Scancode =
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

--- Returns `true` while a key is held down.
---
--- The key is a physical location, so it does not depend on the keyboard layout. Use it for actions like moving or
--- jumping. To read typed text, use `io.getTextInput`. To detect the frame where the key goes down, use
--- `io.isKeyJustPressed`.
---
--- @param keycode The name of the key, like `"Space"`, `"A"` or `"Left Shift"`. See `Scancode` for the list.
--- @return True if the key is down. False if it is up or if the name is not a valid key name.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- local x = 0
--- function Update(deltaTime: number)
--- 	if io.isKeyDown("Left") then
--- 		x -= 0.5 * deltaTime
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `io.isKeyJustPressed`: for actions that happen once per press.
--- - `io.getKeysDown`: to list every key held down.
--- - `gamepad`: for controllers.
function module.isKeyDown(keycode: Scancode | string): boolean
	error("Implemented in native code")
end

--- Returns the name of a key as the player sees it on their keyboard layout.
---
--- Use it to tell the player which key to press. For example, `io.getKeyName("Z")` returns `"W"` on an AZERTY keyboard
--- and `"Z"` on a QWERTY keyboard.
---
--- @param keycode The name of the key, as in `io.isKeyDown`.
--- @return The name of the key, or `nil` if the name is not a valid key name or the key has no name in the current layout.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- local label = io.getKeyName("W")
--- print("Press " .. (label or "?") .. " to jump")
--- ```
function module.getKeyName(keycode: Scancode | string): string?
	error("Implemented in native code")
end

--- Returns the text typed by the player since the last frame.
---
--- Use it for text fields, names and chat. It follows the keyboard layout and handles accents and upper case. To check
--- if a key is pressed for an action, use `io.isKeyDown` instead.
---
--- @return The characters typed since the last frame. It is an empty string when nothing was typed.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- local name = ""
--- function Update(deltaTime: number)
--- 	name ..= io.getTextInput()
--- 	if io.isKeyJustPressed("Backspace") then
--- 		name = string.sub(name, 1, -2)
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `io.isKeyJustPressed`: for key presses that trigger actions.
function module.getTextInput(): string
	error("Implemented in native code")
end

--- Returns `true` during the frame where a key goes down.
---
--- It stays `false` while the key is held, so use it for actions that must trigger once per press, like jumping or
--- opening a menu. Keys are physical locations and do not depend on the keyboard layout: `"Q"` is always the key at
--- the top-left letter position, even on AZERTY. Do not use it for typing text, see `io.getTextInput`.
---
--- @param keycode The name of the key, like `"Space"`. See `Scancode` for the list.
--- @return True if the key was pressed this frame. False if the name is not a valid key name.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- local jumps = 0
--- function Update(deltaTime: number)
--- 	if io.isKeyJustPressed("Space") then
--- 		jumps += 1
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `io.isKeyDown`: for actions that last while the key is held.
--- - `io.getTextInput`: for typing text.
function module.isKeyJustPressed(keycode: Scancode | string): boolean
	error("Implemented in native code")
end

--- Returns the names of all the keys that are currently held down.
---
--- Use it to find the name of a key, or to build a "press any key" screen. The order of the keys is not specified.
---
--- @return A new list of key names. It is empty when no key is down.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- function Update(deltaTime: number)
--- 	for _, key in io.getKeysDown() do
--- 		print("Down: " .. key)
--- 	end
--- end
--- ```
function module.getKeysDown(): { Scancode }
	error("Implemented in native code")
end

--- Returns the position of the mouse cursor.
---
--- The position is in screen coordinates: (-1, -1) is the bottom left, (1, 1) the top right and (0, 0) the center of the
--- screen.
---
--- @return The position of the mouse as a `Vec2`.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- function Update(deltaTime: number)
--- 	local mouse = io.getMouse()
--- 	print(mouse.x, mouse.y) -- 0 0 when the cursor is at the center
--- end
--- ```
---
--- # See also
---
--- - `io.getMouseState`: to know which buttons are down.
--- - `io.getCurrentTouches`: for touch screens.
function module.getMouse(): Vec.Vec2
	error("Implemented in native code")
end

--- Returns which mouse buttons are down or were just pressed.
---
--- The `isLeftDown` and `isRightDown` fields are `true` while the button is held. The `isLeftJustPressed` and
--- `isRightJustPressed` fields are `true` only during the frame where the button goes down. Use `io.getMouse` for the
--- position.
---
--- @return A table with the fields `isLeftDown`, `isRightDown`, `isLeftJustPressed` and `isRightJustPressed`.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- function Update(deltaTime: number)
--- 	if io.getMouseState().isLeftJustPressed then
--- 		print("Clicked at", io.getMouse())
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `io.getMouse`: the position of the cursor.
function module.getMouseState(): {
	isLeftDown: boolean,
	isRightDown: boolean,
	isLeftJustPressed: boolean,
	isRightJustPressed: boolean,
}
	error("Implemented in native code")
end

--- Returns how much the mouse wheel was scrolled since the last frame.
---
--- The `x` component is the horizontal scroll and the `y` component is the vertical scroll. Most mice have no
--- horizontal scroll, so `x` is usually 0. Both are 0 when the wheel did not move.
---
--- @return The scroll amount as a `Vec2`.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- local zoom = 1
--- function Update(deltaTime: number)
--- 	zoom += io.getMouseWheel().y * 0.1
--- end
--- ```
function module.getMouseWheel(): Vec.Vec2
	error("Implemented in native code")
end

--- A finger touching the screen, as returned by `io.getCurrentTouches`.
export type Touch = {
	--- A stable identifier for this finger while it remains on the screen.
	id: number,
	--- The current position in screen coordinates.
	position: Vec.Vec2,
	--- The normalized pressure reported by the device.
	pressure: number,
}

--- Returns all the fingers that are currently touching the screen.
---
--- Use `Touch.id` to follow the same finger from one frame to the next. The list is empty when nothing touches the
--- screen, and on devices without a touch screen.
---
--- @return A new list of `Touch`.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- function Update(deltaTime: number)
--- 	for _, touch in io.getCurrentTouches() do
--- 		print(touch.id, touch.position.x, touch.position.y)
--- 	end
--- end
--- ```
function module.getCurrentTouches(): { Touch }
	error("Implemented in native code")
end

--- Returns the current size of the window.
---
--- @return The width and height of the window, in pixels, as a `Vec2`.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- local size = io.getWindowSize()
--- print(size.x .. "x" .. size.y)
--- ```
function module.getWindowSize(): Vec.Vec2
	error("Implemented in native code")
end

--- Asks to move the window to the center of the screen.
---
--- The window moves after the current frame. Does nothing on the web.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- io.setWindowSize(800, 600)
--- io.centerWindow()
--- ```
function module.centerWindow(): ()
	error("Implemented in native code")
end

--- Returns the size of the screen the game is displayed on.
---
--- @return The width and height of the screen, in pixels, as a `Vec2`.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- local screen = io.getScreenSize()
--- io.setWindowSize(screen.x / 2, screen.y / 2)
--- ```
function module.getScreenSize(): Vec.Vec2
	error("Implemented in native code")
end

--- Asks to change the size of the window.
---
--- On desktop, the window is resized to the requested size. On the web and in the editor, the game keeps the
--- requested aspect ratio and black borders are added where needed.
---
--- @param width The requested width, in pixels. Use a whole number that is not negative.
--- @param height The requested height, in pixels. Use a whole number that is not negative.
---
--- # Errors
---
--- Raises an error if a size is negative or is not a number that fits in a whole number.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- io.setWindowSize(960, 540)
--- ```
---
--- # See also
---
--- - `io.getScreenSize`: the size of the screen.
--- - `io.setResizeable`: to lock the window size.
function module.setWindowSize(width: number, height: number): ()
	error("Implemented in native code")
end

--- Sets whether the player can resize the window.
---
--- @param resizeable True to let the player resize the window, `false` to lock its size.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- io.setResizeable(false)
--- ```
function module.setResizeable(resizeable: boolean): ()
	error("Implemented in native code")
end

--- Sets the text shown in the title bar of the window.
---
--- @param title The new title.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- io.setWindowTitle("My game - level 1")
--- ```
function module.setWindowTitle(title: string): ()
	error("Implemented in native code")
end

--- Returns `true` if the window is currently minimized.
---
--- While the window is minimized, Vectarine runs much slower to save battery life and avoid using the CPU.
---
--- @return True if the window is minimized.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- function Update(deltaTime: number)
--- 	if io.isWindowMinimized() then
--- 		return
--- 	end
--- end
--- ```
function module.isWindowMinimized(): boolean
	error("Implemented in native code")
end

--- Asks to switch the window between fullscreen and windowed mode.
---
--- `true` and `"fullscreen"` switch to fullscreen. `false` and `"windowed"` go back to a window. `"desktop"` is
--- "desktop fullscreen" (also called fake fullscreen): the window covers the screen without changing the video mode.
--- Before calling it, you can set the window size to the size of the screen to avoid black borders.
---
--- Does nothing on the web: the game is usually embedded in a page or player which already has a fullscreen button.
---
--- @param fullscreen The requested mode. Any other value is ignored.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- function Update(deltaTime: number)
--- 	if io.isKeyJustPressed("F11") then
--- 		io.setFullscreen("desktop")
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `io.setWindowSize`: to set the size of the window.
function module.setFullscreen(fullscreen: true | false | "fullscreen" | "windowed" | "desktop"): ()
	error("Implemented in native code")
end

--- Returns the platform the game is running on.
---
--- Use it to adapt controls or skip features that do not exist everywhere, like `io.setFullscreen` on the web.
---
--- @return One of `"windows"`, `"macos"`, `"linux"` or `"web"`.
---
--- # Example
---
--- ```luau
--- const io = require("@vectarine/io")
---
--- if io.getPlatform() == "web" then
--- 	print("Running in a browser")
--- end
--- ```
function module.getPlatform(): "windows" | "macos" | "linux" | "web"
	error("Implemented in native code")
end

return module
