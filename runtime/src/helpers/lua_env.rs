use std::{cell::RefCell, collections::VecDeque, path::Path, rc::Rc};

use mlua::Table;
use sdl2::keyboard::Keycode;

use crate::helpers::{draw_instruction, io::IoEnvState};

#[derive(Clone, Debug)]
pub struct LuaEnvironment {
    pub lua: Rc<mlua::Lua>,
    pub draw_instructions: Rc<RefCell<VecDeque<draw_instruction::DrawInstruction>>>,
    pub env_state: Rc<RefCell<IoEnvState>>,

    pub frame_messages: Rc<RefCell<Vec<String>>>,
    pub messages: Rc<RefCell<VecDeque<String>>>,
}

impl LuaEnvironment {
    pub fn new() -> Self {
        let lua_options = mlua::LuaOptions::default();
        let lua_libs = mlua::StdLib::MATH | mlua::StdLib::TABLE | mlua::StdLib::STRING;

        let lua =
            Rc::new(mlua::Lua::new_with(lua_libs, lua_options).expect("Failed to create Lua"));
        let _ = lua.sandbox(false);

        let draw_instructions = Rc::new(RefCell::new(VecDeque::new()));
        let queue_for_closure = draw_instructions.clone();

        lua.globals()
            .set(
                "drawRect",
                lua.create_function(move |_, (x, y, w, h, color): (f32, f32, f32, f32, Table)| {
                    let color = [
                        color.get::<f32>("r").unwrap_or(0.0),
                        color.get::<f32>("g").unwrap_or(0.0),
                        color.get::<f32>("b").unwrap_or(0.0),
                        color.get::<f32>("a").unwrap_or(0.0),
                    ];
                    queue_for_closure.borrow_mut().push_back(
                        draw_instruction::DrawInstruction::Rectangle { x, y, w, h, color },
                    );
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let queue_for_closure = draw_instructions.clone();
        lua.globals()
            .set(
                "clear",
                lua.create_function(move |_, (color,): (Table,)| {
                    let color = [
                        color.get::<f32>("r").unwrap_or(0.0),
                        color.get::<f32>("g").unwrap_or(0.0),
                        color.get::<f32>("b").unwrap_or(0.0),
                        color.get::<f32>("a").unwrap_or(0.0),
                    ];
                    queue_for_closure
                        .borrow_mut()
                        .push_back(draw_instruction::DrawInstruction::Clear { color });
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let env_state = Rc::new(RefCell::new(IoEnvState::default()));

        let env_state_for_closure = env_state.clone();
        lua.globals()
            .set(
                "isKeyDown",
                lua.create_function(move |_, keycode_name: String| {
                    let keycode = Keycode::from_name(&keycode_name);
                    let Some(keycode) = keycode else {
                        return Ok(false);
                    };
                    let is_pressed = *env_state_for_closure
                        .borrow()
                        .keyboard_state
                        .get(&keycode)
                        .unwrap_or(&false);
                    Ok(is_pressed)
                })
                .unwrap(),
            )
            .unwrap();

        let env_state_for_closure = env_state.clone();
        lua.globals()
            .set(
                "getKeysDown",
                lua.create_function(move |lua, ()| {
                    let table = lua.create_table().unwrap();
                    for (keycode, is_pressed) in
                        env_state_for_closure.borrow().keyboard_state.iter()
                    {
                        if *is_pressed {
                            let _ = table.set(table.len().unwrap() + 1, keycode.name());
                        }
                    }
                    Ok(table)
                })
                .unwrap(),
            )
            .unwrap();

        let frame_messages = Rc::new(RefCell::new(Vec::new()));
        let frame_messages_for_closure = frame_messages.clone();

        lua.globals()
            .set(
                "fprint",
                lua.create_function(move |_, msg: mlua::Value| {
                    let msg = msg.to_string().unwrap_or_default();
                    frame_messages_for_closure.borrow_mut().push(msg);
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let messages = Rc::new(RefCell::new(VecDeque::new()));
        let messages_for_closure = messages.clone();
        lua.globals()
            .set(
                "dprint",
                lua.create_function(move |_, msg: mlua::Value| {
                    let msg = msg.to_string().unwrap_or_default();
                    messages_for_closure.borrow_mut().push_front(msg);
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let env_state_for_closure = env_state.clone();

        lua.globals()
            .set(
                "mouse",
                lua.create_function(move |lua, ()| {
                    let mouse_state = env_state_for_closure.borrow().mouse_state.clone();
                    let table = lua.create_table().unwrap();
                    let _ = table.set("x", mouse_state.x);
                    let _ = table.set("y", mouse_state.y);
                    let _ = table.set("is_left_down", mouse_state.is_left_down);
                    let _ = table.set("is_right_down", mouse_state.is_right_down);
                    Ok(table)
                })
                .unwrap(),
            )
            .unwrap();

        let env_state_for_closure = env_state.clone();
        lua.globals()
            .set(
                "windowSize",
                lua.create_function(move |lua, ()| {
                    let state = env_state_for_closure.borrow();
                    let table = lua.create_table().unwrap();
                    let _ = table.set("x", state.window_width);
                    let _ = table.set("y", state.window_height);
                    Ok(table)
                })
                .unwrap(),
            )
            .unwrap();

        lua.globals()
            .set(
                "toString",
                lua.create_function(move |_, args: (mlua::Value,)| {
                    let string = stringify_lua_value(&args.0);
                    Ok(string)
                })
                .unwrap(),
            )
            .unwrap();

        LuaEnvironment {
            lua,
            draw_instructions,
            env_state,
            frame_messages,
            messages,
        }
    }
}

impl Default for LuaEnvironment {
    fn default() -> Self {
        Self::new()
    }
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
        mlua::Value::Table(table) => table
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
            .join(", "),
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
        mlua::Value::UserData(userdata) => {
            let ptr = userdata.to_pointer();
            format!("[userdata: {ptr:?}]")
        }
        mlua::Value::LightUserData(lightuserdata) => {
            let ptr = lightuserdata.0;
            format!("[lightuserdata: {ptr:?}]")
        }
        _ => "[unknown]".to_string(),
    }
}
