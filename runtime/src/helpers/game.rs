use std::{cell::RefCell, rc::Rc, sync::Arc};

use glow::HasContext;
use sdl2::{EventPump, video::FullscreenType};

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
    pub gl: Arc<glow::Context>,
    pub batch: BatchDraw2d,
    pub event_pump: EventPump,
    pub lua_env: LuaEnvironment,
}

impl Game {
    pub fn new(
        gl: &Arc<glow::Context>,
        batch: BatchDraw2d,
        event_pump: EventPump,
        lua_env: LuaEnvironment,
    ) -> Self {
        Game {
            gl: gl.clone(),
            batch,
            event_pump,
            lua_env,
        }
    }

    /// Initializes the game and then calls the Load function in Lua, if it exists.
    pub fn load(
        &mut self,
        video: &Rc<RefCell<sdl2::VideoSubsystem>>,
        window: &Rc<RefCell<sdl2::video::Window>>,
    ) {
        // Make screen and window size accessible inside Load.
        if let Ok(display_size) = video.borrow().display_bounds(0) {
            self.lua_env.env_state.borrow_mut().screen_width = display_size.width();
            self.lua_env.env_state.borrow_mut().screen_height = display_size.height();

            let size = window.borrow().size();
            let drawable_size = window.borrow().drawable_size();
            let (px_ratio_x, px_ratio_y) = (
                drawable_size.0 as f32 / size.0 as f32,
                drawable_size.1 as f32 / size.1 as f32,
            );

            self.lua_env.env_state.borrow_mut().px_ratio_x = px_ratio_x;
            self.lua_env.env_state.borrow_mut().px_ratio_y = px_ratio_y;
        }

        {
            let (width, height) = window.borrow().size();
            self.lua_env.env_state.borrow_mut().window_width = width;
            self.lua_env.env_state.borrow_mut().window_height = height;
        }

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
        _in_editor: bool,
    ) {
        let framebuffer_width;
        let framebuffer_height;
        {
            let mut env_state = self.lua_env.env_state.borrow_mut();
            let (width, height) = screen_size(&window.borrow());
            env_state.window_width = width;
            env_state.window_height = height;
            let aspect_ratio = width as f32 / height as f32;
            // This works in the editor, but not the runtime.
            // On the web, this is different, the aspect ratio needs to be squared??
            //self.batch.set_aspect_ratio(aspect_ratio * aspect_ratio);
            self.batch.set_aspect_ratio(aspect_ratio);

            framebuffer_width = width;
            framebuffer_height = height;
        }

        {
            // This is incorrect on the web.
            let gl = &self.gl;
            unsafe {
                gl.viewport(0, 0, framebuffer_width as i32, framebuffer_height as i32);
            }
        }

        {
            let env_state = self.lua_env.env_state.borrow_mut();
            if env_state.is_window_resizeable {
                window.borrow_mut().set_resizable(true);
            } else {
                window.borrow_mut().set_resizable(false);
            }
        }
        {
            let mut env_state = self.lua_env.env_state.borrow_mut();
            if let Some(target_size) = env_state.window_target_size {
                let (target_width, target_height) = target_size;
                let _ = window.borrow_mut().set_size(target_width, target_height);
                env_state.window_target_size = None;
            }
            if let Some(fullscreen_request) = env_state.fullscreen_state_request {
                if fullscreen_request {
                    let _ = window.borrow_mut().set_fullscreen(FullscreenType::True);
                } else {
                    let _ = window.borrow_mut().set_fullscreen(FullscreenType::Off);
                }
                env_state.fullscreen_state_request = None;
            }
        }

        process_events(
            self,
            events,
            framebuffer_width as f32,
            framebuffer_height as f32,
        );

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
                    draw_instruction::DrawInstruction::Rectangle { pos, size, color } => {
                        self.batch.draw_rect(pos.x, pos.y, size.x, size.y, color);
                    }
                    draw_instruction::DrawInstruction::Circle { pos, radius, color } => {
                        self.batch.draw_circle(pos.x, pos.y, radius, color);
                    }
                    draw_instruction::DrawInstruction::Image {
                        pos,
                        size,
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

                        self.batch.draw_image(pos.x, pos.y, size.x, size.y, texture);
                    }
                    draw_instruction::DrawInstruction::Text {
                        pos,
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
                        let (x, y) = (pos.x, pos.y);
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

#[cfg(not(target_os = "emscripten"))]
pub fn screen_size(window: &sdl2::video::Window) -> (u32, u32) {
    window.drawable_size()
}

#[cfg(target_os = "emscripten")]
pub fn screen_size(_window: &sdl2::video::Window) -> (u32, u32) {
    use emscripten_val::Val;
    let size = Val::global("vectarine").call("getCanvasSizeInPx", &[]);
    let width = size.get(&Val::from_str("width")).as_i32();
    let height = size.get(&Val::from_str("height")).as_i32();
    (width as u32, height as u32)
}
