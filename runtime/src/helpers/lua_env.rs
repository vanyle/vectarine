use std::{
    collections::{HashMap, VecDeque},
    path::Path,
    sync::{Arc, Mutex},
};

use mlua::Table;
use sdl2::keyboard::Keycode;

use crate::helpers::draw_instruction;

#[derive(Clone, Debug)]
pub struct LuaEnvironment {
    pub lua: Arc<mlua::Lua>,
    pub draw_instructions: Arc<Mutex<VecDeque<draw_instruction::DrawInstruction>>>,
    pub keyboard_state: Arc<Mutex<HashMap<Keycode, bool>>>,

    pub frame_messages: Arc<Mutex<Vec<String>>>,
    pub messages: Arc<Mutex<VecDeque<String>>>,
}

impl LuaEnvironment {
    pub fn new() -> Self {
        let lua_options = mlua::LuaOptions::default();
        let lua_libs = mlua::StdLib::MATH | mlua::StdLib::TABLE | mlua::StdLib::STRING;

        let lua =
            Arc::new(mlua::Lua::new_with(lua_libs, lua_options).expect("Failed to create Lua"));
        let _ = lua.sandbox(false);

        let draw_instructions = Arc::new(Mutex::new(VecDeque::new()));
        let queue_for_closure = draw_instructions.clone();

        lua.globals()
            .set(
                "DrawRect",
                lua.create_function(move |_, (x, y, w, h, color): (f32, f32, f32, f32, Table)| {
                    let color = [
                        color.get::<f32>("r").unwrap_or(0.0),
                        color.get::<f32>("g").unwrap_or(0.0),
                        color.get::<f32>("b").unwrap_or(0.0),
                        color.get::<f32>("a").unwrap_or(0.0),
                    ];
                    queue_for_closure.lock().unwrap().push_back(
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
                "Clear",
                lua.create_function(move |_, (color,): (Table,)| {
                    let color = [
                        color.get::<f32>("r").unwrap_or(0.0),
                        color.get::<f32>("g").unwrap_or(0.0),
                        color.get::<f32>("b").unwrap_or(0.0),
                        color.get::<f32>("a").unwrap_or(0.0),
                    ];
                    queue_for_closure
                        .lock()
                        .unwrap()
                        .push_back(draw_instruction::DrawInstruction::Clear { color });
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let keyboard_state = Arc::new(Mutex::new(HashMap::<Keycode, bool>::new()));

        let keyboard_state_for_closure = keyboard_state.clone();
        lua.globals()
            .set(
                "IsKeyDown",
                lua.create_function(move |_, keycode_name: String| {
                    let keycode = Keycode::from_name(&keycode_name);
                    let Some(keycode) = keycode else {
                        return Ok(false);
                    };
                    let is_pressed = *keyboard_state_for_closure
                        .lock()
                        .unwrap()
                        .get(&keycode)
                        .unwrap_or(&false);
                    Ok(is_pressed)
                })
                .unwrap(),
            )
            .unwrap();

        let keyboard_state_for_closure = keyboard_state.clone();
        lua.globals()
            .set(
                "GetKeysDown",
                lua.create_function(move |lua, ()| {
                    let table = lua.create_table().unwrap();
                    for (keycode, is_pressed) in keyboard_state_for_closure.lock().unwrap().iter() {
                        if *is_pressed {
                            let _ = table.set(table.len().unwrap() + 1, keycode.name());
                        }
                    }
                    Ok(table)
                })
                .unwrap(),
            )
            .unwrap();

        let frame_messages = Arc::new(Mutex::new(Vec::new()));
        let frame_messages_for_closure = frame_messages.clone();

        lua.globals()
            .set(
                "fprint",
                lua.create_function(move |_, msg: mlua::Value| {
                    let msg = msg.to_string().unwrap_or_default();
                    frame_messages_for_closure.lock().unwrap().push(msg);
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let messages = Arc::new(Mutex::new(VecDeque::new()));
        let messages_for_closure = messages.clone();
        lua.globals()
            .set(
                "dprint",
                lua.create_function(move |_, msg: mlua::Value| {
                    let msg = msg.to_string().unwrap_or_default();
                    messages_for_closure.lock().unwrap().push_front(msg);
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        LuaEnvironment {
            lua,
            draw_instructions,
            keyboard_state,
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
