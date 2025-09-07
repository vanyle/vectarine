use std::{
    collections::{HashMap, VecDeque},
    path::Path,
    sync::{Arc, Mutex},
};

use sdl2::keyboard::Keycode;

use crate::draw_instruction;

pub struct LuaEnvironment {
    pub lua: Arc<mlua::Lua>,
    pub draw_instructions: Arc<Mutex<VecDeque<draw_instruction::DrawInstruction>>>,
    pub keyboard_state: Arc<Mutex<HashMap<Keycode, bool>>>,
}

impl LuaEnvironment {
    pub fn new() -> Self {
        let lua = Arc::new(mlua::Lua::new());
        let _ = lua.sandbox(false);

        let draw_instructions = Arc::new(Mutex::new(VecDeque::new()));
        let queue_for_closure = draw_instructions.clone();

        lua.globals()
            .set(
                "DrawRect",
                lua.create_function(move |_, (x, y, w, h): (f32, f32, f32, f32)| {
                    queue_for_closure
                        .lock()
                        .unwrap()
                        .push_back(draw_instruction::DrawInstruction::Rectangle { x, y, w, h });
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let queue_for_closure = draw_instructions.clone();
        lua.globals()
            .set(
                "SetColor",
                lua.create_function(move |_, (r, g, b): (u8, u8, u8)| {
                    queue_for_closure
                        .lock()
                        .unwrap()
                        .push_back(draw_instruction::DrawInstruction::SetColor { r, g, b });
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let queue_for_closure = draw_instructions.clone();
        lua.globals()
            .set(
                "Clear",
                lua.create_function(move |_, ()| {
                    queue_for_closure
                        .lock()
                        .unwrap()
                        .push_back(draw_instruction::DrawInstruction::Clear);
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        let keyboard_state = Arc::new(Mutex::new(HashMap::new()));

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

        LuaEnvironment {
            lua,
            draw_instructions,
            keyboard_state,
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
