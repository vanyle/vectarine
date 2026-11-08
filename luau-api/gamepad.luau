const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

--[[
Reads gamepad buttons and joysticks, and controls rumble and lights.

Call `gamepad.getGamepad` to get a `Gamepad`, or `nil` if none is connected. Then read it every frame from `Update`
with `Gamepad:isButtonDown` and `Gamepad:getJoystickTilt`. Always handle the `nil` case, because players may not have
a gamepad. For keyboard and mouse input, see the `io` module.

# Example

```luau
const gamepad = require("@vectarine/gamepad")

function Update(deltaTime: number)
	local pad = gamepad.getGamepad()
	if pad and pad:isButtonDown("a") then
		print("Jump!")
	end
end
```

# Common mistakes

- `gamepad.getGamepad` returns `nil` when no gamepad is connected, or after it was unplugged. Check for `nil` every frame.
- `Gamepad:getJoystickTilt` returns raw values from `-32768` to `32767`, not numbers between `-1` and `1`. Divide them yourself
  and ignore small values, because there is no dead zone.
- `Gamepad:isButtonDown` is `true` on every frame the button is held, not only when it is pressed.
]]
const module = {}

--- The name of a gamepad button, following the layout of an Xbox gamepad.
---
--- Names are not case sensitive. Several names can designate the same button, for example `"start"` and `"options"`.
--- The `"a"` button is the bottom button of the four buttons on the right of the gamepad, whatever the label printed on
--- it. Pass a name of this type to `Gamepad:isButtonDown`.
export type GamepadButton =
	"a"
	| "b"
	| "x"
	| "y"
	| "start"
	| "options"
	| "+"
	| "select"
	| "back"
	| "share"
	| "-"
	| "home"
	| "guide"
	| "leftShoulder"
	| "rightShoulder"
	| "lb"
	| "rb"
	| "l1"
	| "r1"
	| "l"
	| "r"
	| "leftStick"
	| "rightStick"
	| "ls"
	| "rs"
	| "l3"
	| "r3"
	| "up"
	| "down"
	| "left"
	| "right"
	| "misc"
	| "capture"
	| "mute"
	| "paddle1"
	| "paddle2"
	| "paddle3"
	| "paddle4"
	| "touchpad"

local GamepadImpl = { type = "gamepad" }
GamepadImpl.__index = GamepadImpl

--- A gamepad is an input device for games, also called a "controller".
---
--- You get one from `gamepad.getGamepad`. Use it to read buttons with `Gamepad:isButtonDown` and joysticks with
--- `Gamepad:getJoystickTilt`, and to make it vibrate with `Gamepad:rumble`.
export type Gamepad = typeof(setmetatable({}, GamepadImpl))

--- Returns the number of connected gamepads.
---
--- @return The number of gamepads. It is `0` if none is connected.
---
--- # Example
---
--- ```luau
--- const gamepad = require("@vectarine/gamepad")
---
--- print(gamepad.getGamepadCount() > 0) -- true if a gamepad is plugged in
--- ```
function module.getGamepadCount(): number
	error("Implemented in native code")
end

--- Returns the gamepad at an index, or `nil` if there is none.
---
--- Calling it again with the same index returns the same gamepad, so you can call it every frame. If the gamepad was
--- unplugged, the index is opened again, and another gamepad may have taken it.
---
--- @param gamepadIndex The index of the gamepad, starting at `0`. Defaults to `0`, the first gamepad.
--- @return The gamepad, or `nil` if no gamepad is available at this index.
---
--- # Example
---
--- ```luau
--- const gamepad = require("@vectarine/gamepad")
---
--- function Update(deltaTime: number)
--- 	local pad = gamepad.getGamepad(0)
--- 	if pad == nil then
--- 		return -- play with the keyboard instead
--- 	end
--- 	print(pad:isButtonDown("start"))
--- end
--- ```
function module.getGamepad(gamepadIndex: number?): Gamepad?
	error("Implemented in native code")
end

--- Returns `true` if a button of the gamepad is currently held down.
---
--- It is `true` on every frame while the button is held, not only on the frame it was pressed. If you need to react once
--- per press, remember the previous value yourself. If the name is not a valid button name, it always returns `false`.
---
--- @param buttonName The name of the button. See `GamepadButton` for the valid names.
--- @return True if the button is held down.
---
--- # Example
---
--- ```luau
--- const gamepad = require("@vectarine/gamepad")
---
--- local wasDown = false
---
--- function Update(deltaTime: number)
--- 	local pad = gamepad.getGamepad()
--- 	if pad == nil then
--- 		return
--- 	end
--- 	local isDown = pad:isButtonDown("a")
--- 	if isDown and not wasDown then
--- 		print("A was just pressed")
--- 	end
--- 	wasDown = isDown
--- end
--- ```
---
--- # See also
---
--- - `io.isKeyDown`: reads the keyboard instead.
--- - `Gamepad:getJoystickTilt`: reads the joysticks.
function GamepadImpl:isButtonDown(buttonName: GamepadButton | string): boolean
	error("Implemented in native code")
end

--- Returns the tilt of the two joysticks and of the two triggers of the gamepad.
---
--- It returns three vectors: the left joystick, the right joystick and the triggers (`x` is the left trigger, `y` is
--- the right trigger). The values are the raw values of the gamepad, not numbers between `-1` and `1`: joystick axes go from
--- `-32768` to `32767`, and triggers from `0` to `32767`. For the vertical axis of a joystick, negative values mean up. There
--- is no dead zone, so a joystick at rest may report small values: ignore values that are close to `0`. Gamepads
--- that have no triggers or joysticks report `0` for them.
---
--- @return The tilt of the left joystick.
--- @return The tilt of the right joystick.
--- @return The pressure of the left (`x`) and right (`y`) triggers.
---
--- # Example
---
--- ```luau
--- const gamepad = require("@vectarine/gamepad")
--- const vec = require("@vectarine/vec")
---
--- local position = vec.V2(0, 0)
---
--- function Update(deltaTime: number)
--- 	local pad = gamepad.getGamepad()
--- 	if pad == nil then
--- 		return
--- 	end
--- 	local left = pad:getJoystickTilt()
--- 	if math.abs(left.x) > 4000 then
--- 		position.x += left.x / 32768 * deltaTime
--- 	end
--- end
--- ```
function GamepadImpl:getJoystickTilt(): (vec.Vec2, vec.Vec2, vec.Vec2)
	error("Implemented in native code")
end

--- Makes the gamepad vibrate for a short time.
---
--- The gamepad has two motors: a heavy one for low frequency vibrations and a light one for high frequency ones. Calling
--- it again replaces the previous vibration. Nothing happens if the gamepad does not support rumble.
---
--- @param lowFrequency The strength of the low frequency motor, from `0` (off) to `65535` (full strength).
--- @param highFrequency The strength of the high frequency motor, from `0` (off) to `65535` (full strength).
--- @param duration How long the gamepad vibrates, in milliseconds.
---
--- # Example
---
--- ```luau
--- const gamepad = require("@vectarine/gamepad")
---
--- local pad = gamepad.getGamepad()
--- if pad then
--- 	-- Half strength on both motors for a quarter of a second.
--- 	pad:rumble(32000, 32000, 250)
--- end
--- ```
---
--- # See also
---
--- - `Gamepad:setLed`: sets the color of the gamepad light.
function GamepadImpl:rumble(lowFrequency: number, highFrequency: number, duration: number)
	error("Implemented in native code")
end

--- Returns `true` if the gamepad has a light that can change color.
---
--- Call it before `Gamepad:setLed`. Some gamepads, like the PlayStation ones, have a color light.
---
--- @return True if the gamepad has a light whose color can be set.
---
--- # Example
---
--- ```luau
--- const gamepad = require("@vectarine/gamepad")
---
--- local pad = gamepad.getGamepad()
--- if pad then
--- 	print(pad:hasLed())
--- end
--- ```
function GamepadImpl:hasLed(): boolean
	error("Implemented in native code")
end

--- Sets the color of the light of the gamepad.
---
--- Only the red, green and blue components of the color are used, and the alpha is ignored. Nothing happens if the
--- gamepad has no light: check it with `Gamepad:hasLed`.
---
--- @param color The color of the light. Each component goes from `0` to `1`.
---
--- # Example
---
--- ```luau
--- const gamepad = require("@vectarine/gamepad")
--- const vec4 = require("@vectarine/vec4")
---
--- local pad = gamepad.getGamepad()
--- if pad and pad:hasLed() then
--- 	pad:setLed(vec4.RED)
--- end
--- ```
function GamepadImpl:setLed(color: vec4.Vec4)
	error("Implemented in native code")
end

return module
