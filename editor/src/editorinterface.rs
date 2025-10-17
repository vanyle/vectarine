use std::{
    cell::RefCell,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Instant,
};

use egui::RichText;
use egui_extras::{Size, StripBuilder};
use egui_sdl2_platform::sdl2;
use glow::HasContext;
use runtime::{
    anyhow::{self},
    game::drawable_screen_size,
    game_resource::ResourceId,
    io::file,
};
use serde::{Deserialize, Serialize};

use crate::{
    editorconsole::draw_editor_console, editormenu::draw_editor_menu,
    editorresources::draw_editor_resources, editorwatcher::draw_editor_watcher,
    projectstate::ProjectState,
};

const EDITOR_CONFIG_FILE: &str = "vectarine-config.toml";

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    pub is_console_shown: bool,
    pub is_resources_window_shown: bool,
    pub is_watcher_window_shown: bool,
    pub is_always_on_top: bool,
    pub debug_resource_shown: Option<ResourceId>,

    pub opened_project_path: Option<String>,
}

pub struct EditorState {
    pub config: Rc<RefCell<EditorConfig>>,
    pub text_command: String,

    pub project: Rc<RefCell<Option<ProjectState>>>,

    pub start_time: std::time::Instant,
    pub video: Rc<RefCell<sdl2::VideoSubsystem>>,
    pub window: Rc<RefCell<sdl2::video::Window>>,
    pub gl: Arc<glow::Context>,
}

impl EditorState {
    pub fn save_config(&self) {
        let config = &self.config.borrow();
        let data = toml::to_string(config.deref()).unwrap_or_default();
        file::write_file(EDITOR_CONFIG_FILE, data.as_bytes());
    }

    /// Load the editor config from file.
    /// If `auto_start_project` is true, and there was a project opened previously, it is loaded automatically overwriting any current project.
    pub fn load_config(&self, auto_start_project: bool) {
        let config_store = self.config.clone();
        let project = self.project.clone();
        let gl = self.gl.clone();
        let video = self.video.clone();
        let window = self.window.clone();

        file::read_file(
            EDITOR_CONFIG_FILE,
            Box::new(move |data: Option<Vec<u8>>| {
                let Some(data) = data else {
                    return; // no config file
                };
                if let Ok(config) = toml::from_slice::<EditorConfig>(data.as_slice()) {
                    *config_store.borrow_mut() = config;
                    if auto_start_project
                        && let Some(project_path_str) = &config_store.borrow().opened_project_path
                    {
                        let project_path = PathBuf::from(project_path_str);
                        let loaded_project = ProjectState::new(&project_path, gl, video, window);
                        if let Ok(loaded_project) = loaded_project {
                            project.replace(Some(loaded_project));
                        }
                    }
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
            project: Rc::new(RefCell::new(None)),
            video,
            window,
            gl,
        }
    }

    pub fn load_project(&self, project_path: &Path) -> anyhow::Result<()> {
        let project = ProjectState::new(
            project_path,
            self.gl.clone(),
            self.video.clone(),
            self.window.clone(),
        );
        match project {
            Ok(p) => self.project.borrow_mut().replace(p),
            Err(e) => {
                return Err(e);
            }
        };
        self.config.borrow_mut().opened_project_path =
            Some(project_path.to_string_lossy().to_string());
        self.save_config();
        Ok(())
    }

    pub fn close_project(&mut self) {
        self.project.borrow_mut().take();
        self.config.borrow_mut().opened_project_path = None;
        self.save_config();
    }

    pub fn draw_editor_interface(
        &mut self,
        platform: &mut egui_sdl2_platform::Platform,
        sdl: &sdl2::Sdl,
        latest_events: &Vec<sdl2::event::Event>,
        painter: &mut egui_glow::Painter,
    ) {
        // Update the time
        platform.update_time(self.start_time.elapsed().as_secs_f64());
        // Get the egui context and begin drawing the frame
        let ctx = platform.context();

        draw_editor_menu(self, &ctx);
        draw_editor_console(self, &ctx);
        draw_editor_resources(self, &ctx);
        draw_editor_watcher(self, &ctx);

        if self.project.borrow().is_none() {
            draw_empty_screen(self, &ctx);
        }

        // Stop drawing the egui frame and get the full output
        let full_output = platform.end_frame(&mut self.video.borrow_mut()).unwrap();
        // Get the paint jobs
        let paint_jobs = platform.tessellate(&full_output);
        let pj = paint_jobs.as_slice();

        // Render the editor interface on top of the game.
        let size = drawable_screen_size(&self.window.borrow());

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

pub fn open_file_dialog_and_load_project(state: &mut EditorState) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Vectarine Project", &["toml"])
        .set_title("Open Vectarine Project")
        .pick_file()
    else {
        return;
    };
    let result = state.load_project(&path);
    if let Err(e) = result {
        // TO-DO: show error in GUI
        println!("Failed to load project: {e}");
    }
}

pub fn draw_empty_screen(state: &mut EditorState, ctx: &egui::Context) {
    egui::Window::new("No project loaded")
        .default_width(384.0)
        .default_height(256.0)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder().at_most(384.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No project loaded").size(24.0));
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("A project is loaded from a game.toml file. See the gallery for examples.")
                                    .size(18.0),
                            );
                            ui.add_space(16.0);
                            ui.centered_and_justified(|ui| {
                                if ui
                                    .button(RichText::new("Open Project").size(24.0))
                                    .clicked()
                                {
                                    open_file_dialog_and_load_project(state);
                                }
                            });
                        });
                    });
                });
        });
}

pub fn process_events_when_no_game(latest_events: &Vec<sdl2::event::Event>, gl: &glow::Context) {
    unsafe {
        gl.clear_color(0.1, 0.1, 0.1, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    }

    for event in latest_events {
        if let sdl2::event::Event::Quit { .. } = event {
            std::process::exit(0);
        }
    }
}
