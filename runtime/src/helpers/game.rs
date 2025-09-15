use std::{cell::RefCell, rc::Rc, sync::Arc};

use sdl2::EventPump;

use crate::{
    graphics::batchdraw::BatchDraw2d,
    helpers::{
        draw_instruction,
        game_resource::{
            ResourceStatus, font_resource::FontResource, image_resource::ImageResource,
        },
        io::process_events,
        lua_env::LuaEnvironment,
    },
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

    pub fn load(&mut self) {
        let load_fn = self.lua_env.lua.globals().get::<mlua::Function>("Load");
        if let Ok(load_fn) = load_fn {
            let err = load_fn.call::<()>(());
            if let Err(err) = err {
                self.lua_env
                    .messages
                    .borrow_mut()
                    .push_back(format!("Lua error while calling Load():\n{err}"));
            }
        }
    }

    pub fn print_to_editor_console(&self, message: &str) {
        let mut messages = self.lua_env.messages.borrow_mut();
        messages.push_back(message.to_string());
    }

    pub fn main_loop(
        &mut self,
        events: &[sdl2::event::Event],
        window: &Rc<RefCell<sdl2::video::Window>>,
        delta_time: std::time::Duration,
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
                let err = update_fn.call::<()>((delta_time.as_secs_f64(),));
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
                    draw_instruction::DrawInstruction::Circle {
                        x,
                        y,
                        radius,
                        color,
                    } => {
                        self.batch.draw_circle(x, y, radius, color);
                    }
                    draw_instruction::DrawInstruction::Image {
                        x,
                        y,
                        w,
                        h,
                        resource_id,
                    } => {
                        let resource_manager = self.lua_env.resources.borrow();
                        let resource = resource_manager.get_by_id::<ImageResource>(resource_id);
                        let image_resource = match resource {
                            Ok(res) => res,
                            Err(cause) => {
                                self.print_to_editor_console(
                                &format!(
                                    "Warning: Failed to draw image with id '{resource_id}': {cause}",
                                ),
                            );
                                continue;
                            }
                        };

                        let texture = image_resource.texture.borrow();
                        let texture = texture.as_ref();
                        let Some(texture) = texture else {
                            debug_assert!(
                                false,
                                "Resource said it was loaded but the texture is None"
                            );
                            continue; // texture is not loaded. This probably breaks an invariant.
                        };

                        self.batch.draw_image(x, y, w, h, texture);
                    }
                    draw_instruction::DrawInstruction::Text {
                        x,
                        y,
                        text,
                        color,
                        font_size,
                        font_resource_id,
                    } => {
                        let resource_manager = self.lua_env.resources.borrow();
                        let resource = resource_manager.get_by_id::<FontResource>(font_resource_id);
                        let res = match resource {
                            Ok(res) => res,
                            Err(cause) => {
                                self.print_to_editor_console(
                                &format!(
                                    "Warning: Failed to draw text with font id '{font_resource_id}': {cause}",
                                ),
                            );
                                continue;
                            }
                        };

                        let font_rendering_data = res.font_rendering.borrow();
                        let font_rendering_data = font_rendering_data.as_ref();
                        let Some(font_rendering_data) = font_rendering_data else {
                            debug_assert!(
                                false,
                                "Resource said it was loaded but the texture is None"
                            );
                            continue; // texture is not loaded. This probably breaks an invariant.
                        };
                        self.batch
                            .draw_text(x, y, &text, color, font_size, font_rendering_data);
                    }
                    draw_instruction::DrawInstruction::Clear { color } => {
                        self.batch.clear(color[0], color[1], color[2], color[3]);
                    }
                }
            }
        }

        self.batch.draw(true);
    }

    /// Calls reload on all unloaded resource inside the manager.
    pub fn load_resource_as_needed(&mut self, gl: Arc<glow::Context>) {
        let mut to_reload = Vec::new();
        {
            let resource_manager = self.lua_env.resources.borrow();
            for resource in resource_manager.resources.iter() {
                if resource.get_loading_status() != ResourceStatus::Unloaded {
                    continue;
                }
                to_reload.push(resource.clone());
            }
        }
        for resource in to_reload {
            resource.reload(gl.clone());
        }
    }
}
