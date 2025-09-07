use sdl2::{
    EventPump, event::Event, keyboard::Keycode, pixels::Color, rect::FRect, render::Canvas,
    video::Window,
};

use crate::{draw_instruction, lua_env::LuaEnvironment};

pub struct Game {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    lua_env: LuaEnvironment,
}

impl Game {
    pub fn new(canvas: Canvas<Window>, event_pump: EventPump, lua_env: LuaEnvironment) -> Self {
        Game {
            canvas,
            event_pump,
            lua_env,
        }
    }

    pub fn main_loop(&mut self) {
        {
            let mut keyboard_state = self.lua_env.keyboard_state.lock().unwrap();

            for event in self.event_pump.poll_iter() {
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
                        keyboard_state.insert(keycode, false);
                    }
                    Event::KeyDown { keycode, .. } => {
                        let Some(keycode) = keycode else {
                            return;
                        };
                        keyboard_state.insert(keycode, true);
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
                    draw_instruction::DrawInstruction::Rectangle { x, y, w, h } => {
                        self.canvas.fill_frect(FRect::new(x, y, w, h)).unwrap();
                    }
                    // draw_instruction::DrawInstruction::Circle { x, y, radius } => {
                    //     // ...
                    // }
                    draw_instruction::DrawInstruction::SetColor { r, g, b } => {
                        self.canvas.set_draw_color(Color::RGB(r, g, b));
                    }
                    draw_instruction::DrawInstruction::Clear => {
                        self.canvas.clear();
                    }
                    _ => (),
                }
            }
        }

        self.canvas.present();
    }
}
