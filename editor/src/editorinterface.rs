use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Arc, time::Instant};

use egui::RichText;
use egui_extras::Column;
use egui_sdl2_platform::sdl2;
use runtime::helpers::{file, game::Game, game_resource::get_absolute_path};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    is_console_shown: bool,
    is_resources_window_shown: bool,
    debug_resource_shown: Option<u32>,
}

pub struct EditorState {
    pub config: Rc<RefCell<EditorConfig>>,
    text_command: String,

    start_time: std::time::Instant,
    video: Rc<RefCell<sdl2::VideoSubsystem>>,
    window: Rc<RefCell<sdl2::video::Window>>,
    gl: Arc<glow::Context>,
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
            Box::new(move |data: Vec<u8>| {
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

        if ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::NONE,
                egui::Key::I,
            )
        }) {
            let mut config = self.config.borrow_mut();
            config.is_console_shown = !config.is_console_shown;
        }
        egui::TopBottomPanel::top("toppanel").show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Vectarine Editor").size(18.0));
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Toggle console (Ctrl+Shift+I)").clicked() {
                            let mut config = self.config.borrow_mut();
                            config.is_console_shown = !config.is_console_shown;
                        }
                        if ui.button("Resources").clicked() {
                            let mut config = self.config.borrow_mut();
                            config.is_resources_window_shown = !config.is_resources_window_shown;
                        }
                        if ui.button("Save config").clicked() {
                            self.save_config();
                        }
                        if ui.button("Exit (Alt+F4)").clicked() {
                            std::process::exit(0);
                        }
                    });
                });
            });
            // let window_handle = self.window.borrow().raw();
            // sdl2_sys::SDL_SetWindowHitTest(window_handle, callback, callback_data)
        });

        if self.config.borrow().is_console_shown {
            egui::Window::new("Console").show(&ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("console")
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let messages = &mut game.lua_env.messages.borrow_mut();
                        for line in messages.iter().rev() {
                            ui.label(RichText::new(line).monospace());
                        }
                        messages.truncate(100);
                    });
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("frame console")
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let messages = &mut game.lua_env.frame_messages.borrow_mut();
                        for line in messages.iter() {
                            ui.label(RichText::new(line).monospace());
                        }
                        messages.clear();
                    });
                ui.separator();
                let response = ui.text_edit_singleline(&mut self.text_command);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    println!("Running command: {}", self.text_command);
                    self.text_command.clear();
                    response.request_focus();
                }
            });
        }

        if self.config.borrow().is_resources_window_shown {
            egui::Window::new("Resources").show(&ctx, |ui| {
                let available_height = ui.available_height();
                let table = egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .max_scroll_height(available_height);
                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("Path");
                        });
                        header.col(|ui| {
                            ui.label("Type");
                        });
                        header.col(|ui| {
                            ui.label("Status");
                        });
                        header.col(|ui| {
                            ui.label("Actions");
                        });
                    })
                    .body(|mut body| {
                        for (id, res) in
                            game.lua_env.resources.borrow().resources.iter().enumerate()
                        {
                            let description = res.get_resource_info();
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui
                                        .link(description.path.to_string_lossy().to_string())
                                        .clicked()
                                    {
                                        // Open the file
                                        let absolute_path = get_absolute_path(&description.path);
                                        open::that(absolute_path).ok();
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(res.get_type_name().to_string());
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", res.get_loading_status()));
                                });
                                row.col(|ui| {
                                    if ui.button("Reload").clicked() {
                                        let res = res.clone();
                                        let gl = self.gl.clone();
                                        res.reload(gl);
                                    }
                                    let mut config = self.config.borrow_mut();
                                    let id = id as u32;
                                    let shown = config.debug_resource_shown == Some(id);
                                    let text = if shown { "Hide" } else { "Show" };
                                    ui.button(text).clicked().then(|| {
                                        if shown {
                                            config.debug_resource_shown = None;
                                        } else {
                                            config.debug_resource_shown = Some(id);
                                        }
                                    });
                                });
                            });
                        }
                    })
            });
        }

        if let Some(id) = self.config.borrow().debug_resource_shown {
            if let Some(res) = game
                .lua_env
                .resources
                .borrow_mut()
                .resources
                .get_mut(id as usize)
            {
                egui::Window::new(format!(
                    "Resource debug - {}",
                    res.get_resource_info().path.to_string_lossy()
                ))
                .resizable(true)
                .show(&ctx, |ui| {
                    res.draw_debug_gui(ui);
                });
            }
        }

        // Stop drawing the egui frame and get the full output
        let full_output = platform.end_frame(&mut self.video.borrow_mut()).unwrap();
        // Get the paint jobs
        let paint_jobs = platform.tessellate(&full_output);
        let pj = paint_jobs.as_slice();

        // Render the editor interface on top of the game.
        let size = self.window.borrow().size();
        painter.paint_and_update_textures([size.0, size.1], 1.0, pj, &full_output.textures_delta);
        for event in latest_events {
            platform.handle_event(event, sdl, &self.video.borrow());
        }
    }
}
