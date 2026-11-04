const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

--- Reading gamepad input, rumble and leds.
const module = {}

--- Gamepad button names based on the XBOX convention.
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
export type Gamepad = typeof(setmetatable({}, GamepadImpl))

--- Return the number of connected gamepads.
function module.getGamepadCount(): number
	error("Implemented in native code")
end

--- Returns a gamepad from its index (starting at 0). If no index is provided, the first gamepad is returned.
--- Calling it again with the same index returns the same gamepad, so you can call it every frame.
--- If the gamepad was unplugged, the index is opened again (another gamepad may have taken it).
function module.getGamepad(gamepadIndex: number?): Gamepad?
	error("Implemented in native code")
end

--- Checks if a gamepad button is currently pressed.
--- For example, `gamepad:isButtonDown("a")`. If the name is invalid, always return false.
--- See `GamepadButton` for the names of the buttons.
function GamepadImpl:isButtonDown(buttonName: GamepadButton | string): boolean
	error("Implemented in native code")
end

--- Return the tilt of the joysticks of the gamepad. The vector returned represent the right, left and "trigger" tilts.
--- Depending on your joystick, not all of these are supported. Most controller support the right and left one.
function GamepadImpl:getJoystickTilt(): (vec.Vec2, vec.Vec2, vec.Vec2)
	error("Implemented in native code")
end

--- Make the gamepad provided rumble.
function GamepadImpl:rumble(lowFrequency: number, highFrequency: number, duration: number)
	error("Implemented in native code")
end

--- Make the gamepad provided rumble.
function GamepadImpl:hasLed(): boolean
	error("Implemented in native code")
end

function GamepadImpl:setLed(color: vec4.Vec4)
	error("Implemented in native code")
end

return module
