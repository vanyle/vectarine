use std::{cell::RefCell, collections::HashMap, rc::Rc};

use vectarine_plugin_sdk::mlua::{self, AnyUserData, UserDataMethods, Value};
use vectarine_plugin_sdk::sdl2;

use crate::gamepad::{Gamepad, GamepadSystem};
use crate::lua_env::lua_vec4::Vec4;
use crate::{
    io::IoEnvState,
    lua_env::{add_fn_to_table, lua_vec2::Vec2},
};

struct LuaGamepad(Box<dyn Gamepad>);

fn string_to_gamepad_button(button_name: &str) -> Option<sdl2::controller::Button> {
    // We need to implement this because 'sdl2::controller::Button::from_string' is garbage.
    // Names follow the position of the button on an Xbox gamepad (A is the bottom face button), with the PlayStation
    // and Nintendo names as aliases. Keep in sync with `GamepadButton` in luau-api/gamepad.luau.
    use sdl2::controller::Button;
    match button_name.to_lowercase().as_str() {
        "a" => Some(Button::A),
        "b" => Some(Button::B),
        "x" => Some(Button::X),
        "y" => Some(Button::Y),
        "start" | "options" | "+" => Some(Button::Start),
        "select" | "back" | "share" | "-" => Some(Button::Back),
        "home" | "guide" => Some(Button::Guide),
        "leftshoulder" | "lb" | "l1" | "l" => Some(Button::LeftShoulder),
        "rightshoulder" | "rb" | "r1" | "r" => Some(Button::RightShoulder),
        "leftstick" | "ls" | "l3" => Some(Button::LeftStick),
        "rightstick" | "rs" | "r3" => Some(Button::RightStick),
        "up" => Some(Button::DPadUp),
        "down" => Some(Button::DPadDown),
        "left" => Some(Button::DPadLeft),
        "right" => Some(Button::DPadRight),
        "misc" | "capture" | "mute" => Some(Button::Misc1),
        "paddle1" => Some(Button::Paddle1),
        "paddle2" => Some(Button::Paddle2),
        "paddle3" => Some(Button::Paddle3),
        "paddle4" => Some(Button::Paddle4),
        "touchpad" => Some(Button::Touchpad),
        _ => None,
    }
}

pub fn setup_gamepad_api(
    lua: &vectarine_plugin_sdk::mlua::Lua,
    env_state: &Rc<RefCell<IoEnvState>>,
    gamepad_system: &Rc<dyn GamepadSystem>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let io_module = lua.create_table()?;

    let _ = lua.register_userdata_type::<LuaGamepad>(|registry| {
        registry.add_method(
            "isButtonDown",
            move |_, gamepad: &LuaGamepad, (button_name,): (String,)| {
                let button = string_to_gamepad_button(&button_name);
                Ok(button
                    .map(|btn| gamepad.0.is_pressed(btn))
                    .unwrap_or_default())
            },
        );
        registry.add_method("getJoystickTilt", move |_, gamepad: &LuaGamepad, (): ()| {
            Ok(gamepad.0.get_joystick_position())
        });
        registry.add_method_mut(
            "rumble",
            move |_,
                  gamepad: &mut LuaGamepad,
                  (low_freq, high_freq, duration_ms): (u16, u16, u32)| {
                let _: () = gamepad.0.rumble(low_freq, high_freq, duration_ms);
                Ok(())
            },
        );
        registry.add_method("hasLed", move |_, gamepad: &LuaGamepad, (): ()| {
            Ok(gamepad.0.has_led())
        });
        registry.add_method_mut(
            "setLed",
            move |_, gamepad: &mut LuaGamepad, (color,): (Vec4,)| {
                gamepad.0.set_led(
                    (color.x() * 256.0) as u8,
                    (color.y() * 256.0) as u8,
                    (color.z() * 256.0) as u8,
                );
                Ok(())
            },
        );
    });

    add_fn_to_table(lua, &io_module, "getGamepadCount", {
        let gamepad_system = gamepad_system.clone();
        move |_, (): ()| Ok(gamepad_system.num_gamepads())
    });

    add_fn_to_table(lua, &io_module, "getGamepad", {
        let gamepad_system = gamepad_system.clone();
        // A gamepad can only be opened once, so every call for the same index returns the same userdata.
        let opened: RefCell<HashMap<u32, AnyUserData>> = RefCell::new(HashMap::new());
        move |lua, (gamepad_index,): (Option<u32>,)| {
            let gamepad_index = gamepad_index.unwrap_or(0);
            if let Some(cached) = opened.borrow().get(&gamepad_index) {
                let connected = cached
                    .borrow::<LuaGamepad>()
                    .map(|gamepad| gamepad.0.is_connected())
                    .unwrap_or(false);
                if connected {
                    return Ok(Value::UserData(cached.clone()));
                }
            }
            // Unplugged (or never opened): drop the old handle before opening the index again.
            opened.borrow_mut().remove(&gamepad_index);
            match gamepad_system.get_by_index(gamepad_index) {
                None => Ok(mlua::Nil),
                Some(gamepad) => {
                    let userdata = lua.create_any_userdata(LuaGamepad(gamepad))?;
                    opened.borrow_mut().insert(gamepad_index, userdata.clone());
                    Ok(Value::UserData(userdata))
                }
            }
        }
    });

    add_fn_to_table(lua, &io_module, "getMouse", {
        let env_state = env_state.clone();
        move |_, ()| {
            let mouse_state = env_state.borrow().mouse_state.clone();
            Ok(Vec2::new(mouse_state.x, mouse_state.y))
        }
    });

    Ok(io_module)
}
