use sdl2::{EventPump, event::Event, keyboard::Keycode};

use crate::{
    graphics::batchdraw::BatchDraw2d,
    helpers::{draw_instruction, lua_env::LuaEnvironment},
};

pub struct Game {
    pub batch: BatchDraw2d,
    pub event_pump: EventPump,
    pub lua_env: LuaEnvironment,
}

impl Game {
    pub fn new(batch: BatchDraw2d, event_pump: EventPump, lua_env: LuaEnvironment) -> Self {
        Game {
            batch,
            event_pump,
            lua_env,
        }
    }

    pub fn main_loop(&mut self, events: &[sdl2::event::Event]) {
        {
            let mut keyboard_state = self.lua_env.keyboard_state.lock().unwrap();

            for event in events.iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        std::process::exit(0);
                    }
                    Event::KeyUp { keycode, .. } => {
                        let Some(keycode) = keycode else {
                            return;
                        };
                        keyboard_state.insert(*keycode, false);
                    }
                    Event::KeyDown { keycode, .. } => {
                        let Some(keycode) = keycode else {
                            return;
                        };
                        keyboard_state.insert(*keycode, true);
                    }
                    _ => {}
                }
            }
        }

        {
            let update_fn = self.lua_env.lua.globals().get::<mlua::Function>("Update");
            if let Ok(update_fn) = update_fn {
                update_fn.call::<()>(()).unwrap();
            }
        }

        {
            let mut instructions = self.lua_env.draw_instructions.lock().unwrap();
            while let Some(instruction) = instructions.pop_front() {
                match instruction {
                    draw_instruction::DrawInstruction::Rectangle { x, y, w, h, color } => {
                        self.batch.draw_rect(x, y, w, h, color);
                    }
                    // draw_instruction::DrawInstruction::Circle { x, y, radius } => {
                    //     // ...
                    // }
                    draw_instruction::DrawInstruction::Clear { color } => {
                        self.batch.clear(color[0], color[1], color[2], color[3]);
                    }
                    _ => (),
                }
            }
        }

        self.batch.draw(true);
    }
}
