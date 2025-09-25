use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Arc, time::Instant};

use egui_extras::{Column, StripBuilder};
use egui_sdl2_platform::sdl2;
use runtime::{
    game::Game,
    game_resource::{ResourceId, get_absolute_path},
    io::file,
};
use serde::{Deserialize, Serialize};

use crate::{
    editorconsole::draw_editor_console, editormenu::draw_editor_menu,
    editorresources::draw_editor_resources,
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    pub is_console_shown: bool,
    pub is_resources_window_shown: bool,
    pub is_watcher_window_shown: bool,
    pub is_always_on_top: bool,
    pub debug_resource_shown: Option<ResourceId>,
}

pub struct EditorState {
    pub config: Rc<RefCell<EditorConfig>>,
    pub text_command: String,

    pub start_time: std::time::Instant,
    pub video: Rc<RefCell<sdl2::VideoSubsystem>>,
    pub window: Rc<RefCell<sdl2::video::Window>>,
    pub gl: Arc<glow::Context>,
}

impl EditorState {
    pub fn save_config(&self) {
        let config = &self.config.borrow();
        let data = toml::to_string(config.deref()).unwrap_or_default();

        file::write_file("vectarine_config.toml", data.as_bytes());
    }

    pub fn load_config(&self) {
        let config_store = self.config.clone();
        file::read_file(
            "vectarine_config.toml",
            Box::new(move |data: Option<Vec<u8>>| {
                let Some(data) = data else {
                    return; // no config file
                };
                if let Ok(config) = toml::from_slice::<EditorConfig>(data.as_slice()) {
                    *config_store.borrow_mut() = config;
                }
            }),
        );
    }

    pub fn new(
        video: Rc<RefCell<sdl2::VideoSubsystem>>,
        window: Rc<RefCell<sdl2::video::Window>>,
        gl: Arc<glow::Context>,
    ) -> Self {
        Self {
            config: Rc::new(RefCell::new(EditorConfig::default())),
            text_command: String::new(),
            start_time: Instant::now(),
            video,
            window,
            gl,
        }
    }

    pub fn draw_editor_interface(
        &mut self,
        platform: &mut egui_sdl2_platform::Platform,
        sdl: &sdl2::Sdl,
        game: &mut Game,
        latest_events: &Vec<sdl2::event::Event>,
        painter: &mut egui_glow::Painter,
    ) {
        // Update the time
        platform.update_time(self.start_time.elapsed().as_secs_f64());
        // Get the egui context and begin drawing the frame
        let ctx = platform.context();

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num1)) {
            let mut config = self.config.borrow_mut();
            config.is_console_shown = !config.is_console_shown;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num2)) {
            let mut config = self.config.borrow_mut();
            config.is_resources_window_shown = !config.is_resources_window_shown;
        }

        draw_editor_menu(self, &ctx);
        draw_editor_console(self, game, &ctx);
        draw_editor_resources(self, game, &ctx);

        if let Some(id) = self.config.borrow().debug_resource_shown {
            let res = game.lua_env.resources.get_holder_by_id(id);
            egui::Window::new(format!(
                "Resource debug - {}",
                res.get_path().to_string_lossy()
            ))
            .resizable(true)
            .show(&ctx, |ui| {
                res.draw_debug_gui(ui);
            });
        }

        // Stop drawing the egui frame and get the full output
        let full_output = platform.end_frame(&mut self.video.borrow_mut()).unwrap();
        // Get the paint jobs
        let paint_jobs = platform.tessellate(&full_output);
        let pj = paint_jobs.as_slice();

        // Render the editor interface on top of the game.
        let size = self.window.borrow().drawable_size();

        let pixel_per_point = size.0 as f32 / self.window.borrow().size().0 as f32;

        painter.paint_and_update_textures(
            [size.0, size.1],
            pixel_per_point,
            pj,
            &full_output.textures_delta,
        );
        for event in latest_events {
            // Convert mouse position.
            platform.handle_event(event, sdl, &self.video.borrow());
        }
    }
}
