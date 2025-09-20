use std::{cell::RefCell, rc::Rc};

use sdl2::keyboard::Keycode;

use crate::{
    io::IoEnvState,
    lua_env::{add_fn_to_table, lua_vec2::Vec2, stringify_lua_value},
};

/// Adds to the Lua environment functions to interact with the outside environment
/// For example, the keyboard, the mouse, the window, etc...
/// This is called the IO API.
pub fn setup_io_api(
    lua: &Rc<mlua::Lua>,
    env_state: &Rc<RefCell<IoEnvState>>,
    messages: &Rc<RefCell<std::collections::VecDeque<String>>>,
    frame_messages: &Rc<RefCell<Vec<String>>>,
) -> mlua::Result<mlua::Table> {
    let io_module = lua.create_table()?;

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
        move |lua, ()| {
            let state = env_state.borrow();
            let table = lua.create_table().unwrap();
            let _ = table.set("x", state.window_width);
            let _ = table.set("y", state.window_height);
            Ok(table)
        }
    });

    add_fn_to_table(lua, &io_module, "getScreenSize", {
        let env_state = env_state.clone();
        move |lua, ()| {
            let state = env_state.borrow();
            let table = lua.create_table().unwrap();
            let _ = table.set("x", state.screen_width);
            let _ = table.set("y", state.screen_height);
            Ok(table)
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

    add_fn_to_table(lua, &io_module, "setFullscreen", {
        let env_state = env_state.clone();
        move |_, (fullscreen,): (bool,)| {
            env_state.borrow_mut().fullscreen_state_request = Some(fullscreen);
            Ok(())
        }
    });

    add_fn_to_table(lua, &io_module, "fprint", {
        let frame_messages = frame_messages.clone();
        move |_, args: mlua::Variadic<mlua::Value>| {
            let msg = args
                .iter()
                .map(stringify_lua_value)
                .collect::<Vec<_>>()
                .join(" ");
            frame_messages.borrow_mut().push(msg);
            Ok(())
        }
    });

    add_fn_to_table(lua, &io_module, "print", {
        let messages = messages.clone();
        move |_, args: mlua::Variadic<mlua::Value>| {
            let msg = args
                .iter()
                .map(stringify_lua_value)
                .collect::<Vec<_>>()
                .join(" ");
            messages.borrow_mut().push_front(msg);
            Ok(())
        }
    });

    Ok(io_module)
}
