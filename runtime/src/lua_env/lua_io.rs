use std::{cell::RefCell, rc::Rc};

use sdl2::keyboard::Keycode;

use crate::{
    io::IoEnvState,
    lua_env::{add_fn_to_table, get_internals, lua_vec2::Vec2},
};

pub struct RcEnvState(pub Rc<RefCell<IoEnvState>>);
impl mlua::UserData for RcEnvState {}

const ENV_STATE_KEY: &str = "__env_state";

/// Adds to the Lua environment functions to interact with the outside environment
/// For example, the keyboard, the mouse, the window, etc...
/// This is called the IO API.
pub fn setup_io_api(
    lua: &Rc<mlua::Lua>,
    env_state: &Rc<RefCell<IoEnvState>>,
) -> mlua::Result<mlua::Table> {
    let io_module = lua.create_table()?;

    get_internals(lua)
        .raw_set(ENV_STATE_KEY, RcEnvState(env_state.clone()))
        .unwrap();

    add_fn_to_table(lua, &io_module, "isKeyDown", {
        let env_state = env_state.clone();
        move |_, keycode_name: String| {
            let keycode = Keycode::from_name(&keycode_name);
            let Some(keycode) = keycode else {
                return Ok(false);
            };
            let is_pressed = *env_state
                .borrow()
                .keyboard_state
                .get(&keycode)
                .unwrap_or(&false);
            Ok(is_pressed)
        }
    });

    add_fn_to_table(lua, &io_module, "isKeyJustPressed", {
        let env_state = env_state.clone();
        move |_, keycode_name: String| {
            let keycode = Keycode::from_name(&keycode_name);
            let Some(keycode) = keycode else {
                return Ok(false);
            };
            let is_pressed = *env_state
                .borrow()
                .keyboard_just_pressed_state
                .get(&keycode)
                .unwrap_or(&false);
            Ok(is_pressed)
        }
    });

    add_fn_to_table(lua, &io_module, "getKeysDown", {
        let env_state = env_state.clone();
        move |lua, ()| {
            let table = lua.create_table().unwrap();
            for (keycode, is_pressed) in env_state.borrow().keyboard_state.iter() {
                if *is_pressed {
                    let _ = table.set(table.len().unwrap() + 1, keycode.name());
                }
            }
            Ok(table)
        }
    });

    add_fn_to_table(lua, &io_module, "getMouse", {
        let env_state = env_state.clone();
        move |_, ()| {
            let mouse_state = env_state.borrow().mouse_state.clone();
            Ok(Vec2 {
                x: mouse_state.x,
                y: mouse_state.y,
            })
        }
    });

    add_fn_to_table(lua, &io_module, "getMouseState", {
        let env_state = env_state.clone();
        move |lua, ()| {
            let mouse_state = env_state.borrow().mouse_state.clone();
            let table = lua.create_table().unwrap();
            let _ = table.set("is_left_down", mouse_state.is_left_down);
            let _ = table.set("is_right_down", mouse_state.is_right_down);
            Ok(table)
        }
    });

    add_fn_to_table(lua, &io_module, "getWindowSize", {
        let env_state = env_state.clone();
        move |_lua, ()| {
            let state = env_state.borrow();
            Ok(Vec2 {
                x: (state.window_width as f32 / state.px_ratio_x),
                y: (state.window_height as f32 / state.px_ratio_y),
            })
        }
    });

    add_fn_to_table(lua, &io_module, "getScreenSize", {
        let env_state = env_state.clone();
        move |_lua, ()| {
            let state = env_state.borrow();
            Ok(Vec2 {
                x: state.screen_width as f32,
                y: state.screen_height as f32,
            })
        }
    });

    add_fn_to_table(lua, &io_module, "setResizeable", {
        let env_state = env_state.clone();
        move |_, (resizeable,): (bool,)| {
            env_state.borrow_mut().is_window_resizeable = resizeable;
            Ok(())
        }
    });

    add_fn_to_table(lua, &io_module, "setWindowSize", {
        let env_state = env_state.clone();
        move |_, (width, height): (u32, u32)| {
            env_state.borrow_mut().window_target_size = Some((width, height));
            Ok(())
        }
    });

    add_fn_to_table(lua, &io_module, "setWindowTitle", {
        let env_state = env_state.clone();
        move |_, (title,): (String,)| {
            env_state.borrow_mut().window_title = Some(title);
            Ok(())
        }
    });

    add_fn_to_table(lua, &io_module, "centerWindow", {
        let env_state = env_state.clone();
        move |_, ()| {
            env_state.borrow_mut().center_window_request = true;
            Ok(())
        }
    });

    add_fn_to_table(lua, &io_module, "setFullscreen", {
        let env_state = env_state.clone();
        move |_, (fullscreen,): (mlua::Value,)| {
            if let Some(fullscreen_bool) = fullscreen.as_boolean() {
                let fullscreen_mode = if fullscreen_bool {
                    sdl2::video::FullscreenType::True
                } else {
                    sdl2::video::FullscreenType::Off
                };
                env_state.borrow_mut().fullscreen_state_request = Some(fullscreen_mode);
            }
            if let Some(fullscreen_str) = fullscreen.as_string() {
                let fullscreen_mode = match fullscreen_str.to_string_lossy().as_str() {
                    "fullscreen" => Some(sdl2::video::FullscreenType::True),
                    "windowed" => Some(sdl2::video::FullscreenType::Off),
                    "desktop" => Some(sdl2::video::FullscreenType::Desktop),
                    _ => None,
                };
                env_state.borrow_mut().fullscreen_state_request = fullscreen_mode;
            }

            Ok(())
        }
    });

    Ok(io_module)
}

pub fn get_env_state(lua: &mlua::Lua) -> Rc<RefCell<IoEnvState>> {
    let internals = get_internals(lua);
    let value: mlua::Value = internals.raw_get(ENV_STATE_KEY).unwrap();
    let rc_env_state = value.as_userdata().unwrap();
    let rc_env_state = rc_env_state.borrow::<RcEnvState>().unwrap();
    rc_env_state.0.clone()
}
