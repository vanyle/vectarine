use std::{cell::RefCell, rc::Rc};

use sdl2::EventPump;

use crate::{
    graphics::batchdraw::BatchDraw2d,
    helpers::{draw_instruction, io::process_events, lua_env::LuaEnvironment},
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

    pub fn main_loop(
        &mut self,
        events: &[sdl2::event::Event],
        window: &Rc<RefCell<sdl2::video::Window>>,
    ) {
        process_events(self, events);

        {
            let mut env_state = self.lua_env.env_state.borrow_mut();
            let (width, height) = window.borrow().size();
            env_state.window_width = width;
            env_state.window_height = height;
        }

        {
            let update_fn = self.lua_env.lua.globals().get::<mlua::Function>("Update");
            if let Ok(update_fn) = update_fn {
                let err = update_fn.call::<()>(());
                if let Err(err) = err {
                    self.lua_env
                        .messages
                        .borrow_mut()
                        .push_back(format!("Lua error while calling Update():\n{err}"));
                }
            }
        }

        {
            let mut instructions = self.lua_env.draw_instructions.borrow_mut();
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
