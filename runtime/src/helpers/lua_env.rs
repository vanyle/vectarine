use std::{cell::RefCell, collections::VecDeque, path::Path, rc::Rc};

use mlua::ObjectLike;
use sdl2::keyboard::Keycode;

pub mod graphics;
pub mod vec2;

use crate::helpers::{
    draw_instruction::{self},
    game_resource::{ResourceManager, font_resource::FontResource, image_resource::ImageResource},
    io::IoEnvState,
    lua_env::vec2::Vec2,
};

#[derive(Debug, Clone)]
pub struct LuaEnvironment {
    pub lua: Rc<mlua::Lua>,
    pub draw_instructions: Rc<RefCell<VecDeque<draw_instruction::DrawInstruction>>>,
    pub env_state: Rc<RefCell<IoEnvState>>,

    pub frame_messages: Rc<RefCell<Vec<String>>>,
    pub messages: Rc<RefCell<VecDeque<String>>>,

    pub resources: Rc<ResourceManager>,
}

impl LuaEnvironment {
    pub fn new() -> Self {
        let lua_options = mlua::LuaOptions::default();
        let lua_libs = mlua::StdLib::MATH | mlua::StdLib::TABLE | mlua::StdLib::STRING;

        let lua =
            Rc::new(mlua::Lua::new_with(lua_libs, lua_options).expect("Failed to create Lua"));
        let _ = lua.sandbox(false);

        let draw_instructions = Rc::new(RefCell::new(VecDeque::new()));
        let resources = Rc::new(ResourceManager::default());
        let env_state = Rc::new(RefCell::new(IoEnvState::default()));
        let frame_messages = Rc::new(RefCell::new(Vec::new()));
        let messages = Rc::new(RefCell::new(VecDeque::new()));

        lua.globals()
            .set("Global", lua.create_table().unwrap())
            .unwrap();

        let _ = vec2::setup_vec2_api(&lua);
        let _ = graphics::setup_graphics_api(&lua, &draw_instructions);

        let env_state_for_closure = env_state.clone();
        add_global_fn(&lua, "measureText", {
            let resources = resources.clone();
            move |lua, (text, font_resource_id, font_size): (String, usize, f32)| {
                let font_resource = resources.get_by_id::<FontResource>(font_resource_id);
                let result = lua.create_table().unwrap();
                let Ok(font_resource) = font_resource else {
                    let _ = result.set("width", 0.0);
                    let _ = result.set("height", 0.0);
                    let _ = result.set("bearingY", 0.0);
                    return Ok(result);
                };
                let env_state = env_state_for_closure.borrow();
                let ratio = env_state.window_width as f32 / env_state.window_height as f32;
                let (width, height, max_ascent) =
                    font_resource.measure_text(&text, font_size, ratio);
                let _ = result.set("width", width);
                let _ = result.set("height", height);
                let _ = result.set("bearingY", max_ascent);
                Ok(result)
            }
        });

        add_global_fn(&lua, "isKeyDown", {
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

        add_global_fn(&lua, "getKeysDown", {
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

        add_global_fn(&lua, "fprint", {
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

        add_global_fn(&lua, "dprint", {
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

        add_global_fn(&lua, "getMouse", {
            let env_state = env_state.clone();
            move |_, ()| {
                let mouse_state = env_state.borrow().mouse_state.clone();
                Ok(Vec2 {
                    x: mouse_state.x,
                    y: mouse_state.y,
                })
            }
        });

        add_global_fn(&lua, "getMouseState", {
            let env_state = env_state.clone();
            move |lua, ()| {
                let mouse_state = env_state.borrow().mouse_state.clone();
                let table = lua.create_table().unwrap();
                let _ = table.set("is_left_down", mouse_state.is_left_down);
                let _ = table.set("is_right_down", mouse_state.is_right_down);
                Ok(table)
            }
        });

        add_global_fn(&lua, "getWindowSize", {
            let env_state = env_state.clone();
            move |lua, ()| {
                let state = env_state.borrow();
                let table = lua.create_table().unwrap();
                let _ = table.set("x", state.window_width);
                let _ = table.set("y", state.window_height);
                Ok(table)
            }
        });

        add_global_fn(&lua, "getScreenSize", {
            let env_state = env_state.clone();
            move |lua, ()| {
                let state = env_state.borrow();
                let table = lua.create_table().unwrap();
                let _ = table.set("x", state.screen_width);
                let _ = table.set("y", state.screen_height);
                Ok(table)
            }
        });

        add_global_fn(&lua, "setResizeable", {
            let env_state = env_state.clone();
            move |_, (resizeable,): (bool,)| {
                env_state.borrow_mut().is_window_resizeable = resizeable;
                Ok(())
            }
        });

        add_global_fn(&lua, "setWindowSize", {
            let env_state = env_state.clone();
            move |_, (width, height): (u32, u32)| {
                env_state.borrow_mut().window_target_size = Some((width, height));
                Ok(())
            }
        });

        add_global_fn(&lua, "setFullscreen", {
            let env_state = env_state.clone();
            move |_, (fullscreen,): (bool,)| {
                env_state.borrow_mut().fullscreen_state_request = Some(fullscreen);
                Ok(())
            }
        });

        add_global_fn(&lua, "loadImage", {
            let resources = resources.clone();
            move |_, path: String| {
                let id = resources.load_resource::<ImageResource>(Path::new(&path));
                Ok(id)
            }
        });

        add_global_fn(&lua, "loadFont", {
            let resources = resources.clone();
            move |_, path: String| {
                let id = resources.load_resource::<FontResource>(Path::new(&path));
                Ok(id)
            }
        });

        add_global_fn(&lua, "toString", move |_, (arg,): (mlua::Value,)| {
            let string = stringify_lua_value(&arg);
            Ok(string)
        });

        LuaEnvironment {
            lua,
            draw_instructions,
            env_state,
            frame_messages,
            messages,
            resources,
        }
    }
}

impl Default for LuaEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

pub fn add_global_fn<F, A, R>(lua: &Rc<mlua::Lua>, name: &str, func: F)
where
    F: Fn(&mlua::Lua, A) -> mlua::Result<R> + 'static,
    A: mlua::FromLuaMulti,
    R: mlua::IntoLuaMulti,
{
    lua.globals()
        .set(name, lua.create_function(func).unwrap())
        .unwrap()
}

pub fn run_file_and_display_error(lua: &LuaEnvironment, file_content: &[u8], file_path: &Path) {
    let lua_chunk = lua.lua.load(file_content);
    let result = lua_chunk
        .set_name("@".to_owned() + file_path.to_str().unwrap())
        .exec();
    if result.is_err() {
        let error = result.err().unwrap();
        let error_msg = error.to_string();
        println!("Error: {error_msg} in {}", file_path.to_string_lossy());
    }
}

pub fn stringify_lua_value(value: &mlua::Value) -> String {
    let seen = Vec::new();
    stringify_lua_value_helper(value, seen)
}
fn stringify_lua_value_helper(value: &mlua::Value, mut seen: Vec<mlua::Value>) -> String {
    if seen.contains(value) {
        return "[circular]".to_string();
    }
    seen.push(value.clone());

    match value {
        mlua::Value::Nil => "nil".to_string(),
        mlua::Value::Boolean(b) => b.to_string(),
        mlua::Value::Integer(i) => i.to_string(),
        mlua::Value::Number(n) => n.to_string(),
        mlua::Value::String(s) => s.to_string_lossy(),
        mlua::Value::Table(table) => format!(
            "{{{}}}",
            table
                .pairs::<mlua::Value, mlua::Value>()
                .map(|pair| {
                    if let Ok((key, value)) = pair {
                        let key_str = stringify_lua_value(&key);
                        let value_str = stringify_lua_value(&value);
                        format!("[{key_str}] = {value_str}")
                    } else {
                        "[error]".to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        mlua::Value::Function(func) => {
            let fninfo = func.info();
            format!(
                "[function: {}:{}]",
                fninfo.name.unwrap_or("anonymous".to_string()),
                fninfo.line_defined.unwrap_or(0)
            )
        }
        mlua::Value::Thread(thread) => {
            let ptr = thread.to_pointer();
            format!("[thread: {ptr:?}]")
        }
        mlua::Value::UserData(userdata) => userdata.to_string().unwrap_or_else(|_| {
            let ptr = userdata.to_pointer();
            format!("[userdata: {ptr:?}]")
        }),
        mlua::Value::LightUserData(lightuserdata) => {
            let ptr = lightuserdata.0;
            format!("[lightuserdata: {ptr:?}]")
        }
        _ => "[unknown]".to_string(),
    }
}
