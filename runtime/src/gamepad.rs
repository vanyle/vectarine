use vectarine_plugin_sdk::sdl2::{self};

use crate::lua_env::lua_vec2::Vec2;

/// A simplified interface representing a gamepad.
pub trait Gamepad {
    /// A human-friendly name for this gamepad, like "PS4 Controller".
    fn name(&self) -> String;

    /// Returns false if the button does not exist.
    fn is_pressed(&self, button: sdl2::controller::Button) -> bool;

    /// Returns the position of the joysticks. Potentially 0 if there is only one or no joystick on the controller.
    fn get_joystick_position(&self) -> (Vec2, Vec2, Vec2);

    /// Makes the joystick shake for the duration provided.
    fn rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32);

    fn has_led(&self) -> bool;
    fn set_led(&mut self, r: u8, g: u8, b: u8);

    /// False once the gamepad has been unplugged.
    fn is_connected(&self) -> bool;
}

pub struct DummyJoystick;

/// An abstract representation of a gamepad system.
pub trait GamepadSystem {
    fn num_gamepads(&self) -> u32;
    fn get_by_index(&self, joystick_index: u32) -> Option<Box<dyn Gamepad>>;
}

/// The gamepad used if the sdl subsystem failed to initialize.
pub struct DummyGamepadSystem;

impl GamepadSystem for DummyGamepadSystem {
    fn num_gamepads(&self) -> u32 {
        0
    }
    fn get_by_index(&self, _joystick_index: u32) -> Option<Box<dyn Gamepad>> {
        None
    }
}

impl GamepadSystem for sdl2::GameControllerSubsystem {
    fn num_gamepads(&self) -> u32 {
        self.num_joysticks().unwrap_or_default()
    }

    fn get_by_index(&self, gamepad_index: u32) -> Option<Box<dyn Gamepad>> {
        match self.open(gamepad_index) {
            Err(_) => None,
            Ok(joystick) => Some(Box::new(joystick)),
        }
    }
}

impl Gamepad for sdl2::controller::GameController {
    fn name(&self) -> String {
        self.name()
    }

    fn is_pressed(&self, button: sdl2::controller::Button) -> bool {
        self.button(button)
    }

    fn get_joystick_position(&self) -> (Vec2, Vec2, Vec2) {
        // https://wiki.libsdl.org/SDL2/SDL_JoystickGetAxis
        let lx = self.axis(sdl2::controller::Axis::LeftX) as f32;
        let ly = self.axis(sdl2::controller::Axis::LeftY) as f32;
        let rx = self.axis(sdl2::controller::Axis::RightX) as f32;
        let ry = self.axis(sdl2::controller::Axis::RightY) as f32;
        let tx = self.axis(sdl2::controller::Axis::TriggerLeft) as f32;
        let ty = self.axis(sdl2::controller::Axis::TriggerRight) as f32;
        (Vec2::new(lx, ly), Vec2::new(rx, ry), Vec2::new(tx, ty))
    }

    fn rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) {
        let _ = self.set_rumble(low_freq, high_freq, duration_ms);
    }

    fn has_led(&self) -> bool {
        self.has_led()
    }

    fn set_led(&mut self, r: u8, g: u8, b: u8) {
        let _ = self.set_led(r, g, b);
    }

    fn is_connected(&self) -> bool {
        self.attached()
    }
}
