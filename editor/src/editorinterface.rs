use std::{
    cell::RefCell,
    ops::Deref,
    rc::Rc,
    sync::{Arc, LazyLock, Mutex},
    time::Instant,
};

use egui::{RichText, Widget};
use egui_extras::Column;
use egui_sdl2_platform::sdl2;
use runtime::{
    console::Verbosity,
    game::Game,
    game_resource::{ResourceId, get_absolute_path},
    io::file,
    lua_env::to_lua,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    is_console_shown: bool,
    is_resources_window_shown: bool,
    debug_resource_shown: Option<ResourceId>,
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
                    static ALWAYS_ON_TOP: LazyLock<Mutex<bool>> =
                        LazyLock::new(|| Mutex::new(false));

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
                        if ui
                            .checkbox(&mut ALWAYS_ON_TOP.lock().unwrap(), "Always on top")
                            .clicked()
                        {
                            let always_on_top = *ALWAYS_ON_TOP.lock().unwrap();
                            self.window.borrow_mut().set_always_on_top(always_on_top);
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

        static ARE_LOGS_ERROR_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));
        static ARE_LOGS_WARN_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));
        static ARE_LOGS_INFO_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));

        if self.config.borrow().is_console_shown {
            egui::Window::new("Console")
                .max_height(200.0)
                .resizable([true, false])
                .vscroll(false)
                .show(&ctx, |ui| {
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.checkbox(&mut ARE_LOGS_INFO_SHOWN.lock().unwrap(), "Infos");
                        ui.checkbox(&mut ARE_LOGS_WARN_SHOWN.lock().unwrap(), "Warnings");
                        ui.checkbox(&mut ARE_LOGS_ERROR_SHOWN.lock().unwrap(), "Errors");
                    });
                    egui::ScrollArea::vertical()
                        .id_salt("console")
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            let show_errors = *ARE_LOGS_ERROR_SHOWN.lock().unwrap();
                            let show_warnings = *ARE_LOGS_WARN_SHOWN.lock().unwrap();
                            let show_infos = *ARE_LOGS_INFO_SHOWN.lock().unwrap();

                            let messages = &mut game.lua_env.messages.borrow_mut();
                            for line in messages.iter().rev() {
                                let msg = &line.msg;
                                let is_error = line.verbosity == Verbosity::Error;
                                let is_warning = line.verbosity == Verbosity::Warn;
                                let is_info = line.verbosity == Verbosity::Info;
                                if (show_errors && is_error)
                                    || (show_warnings && is_warning)
                                    || (show_infos && is_info)
                                {
                                    let text = if is_error {
                                        RichText::new(msg).color(egui::Color32::RED)
                                    } else if is_warning {
                                        RichText::new(msg).color(egui::Color32::YELLOW)
                                    } else {
                                        RichText::new(msg).color(egui::Color32::WHITE)
                                    };
                                    ui.label(text.monospace());
                                }
                            }
                            messages.truncate(500);
                        });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("frame console")
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            let messages = &mut game.lua_env.frame_messages.borrow_mut();
                            for line in messages.iter() {
                                let msg = &line.msg;
                                ui.label(
                                    RichText::new(msg).color(egui::Color32::WHITE).monospace(),
                                );
                            }
                            messages.clear();
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        let response = egui::TextEdit::singleline(&mut self.text_command).ui(ui);
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let _ = game.lua_env.default_events.console_command_event.trigger(
                                game.lua_env.lua.as_ref(),
                                to_lua(game.lua_env.lua.as_ref(), self.text_command.clone())
                                    .unwrap(),
                            );
                            self.text_command.clear();
                            response.request_focus();
                        }
                        if egui::Button::new("Clear").ui(ui).clicked() {
                            game.lua_env.messages.borrow_mut().clear();
                        }
                    });
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
                    .column(Column::auto())
                    .max_scroll_height(available_height);
                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("ID");
                        });
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
                        for (id, res) in game.lua_env.resources.enumerate() {
                            let resources = game.lua_env.resources.clone();
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(id.to_string());
                                });
                                row.col(|ui| {
                                    if ui
                                        .link(res.get_path().to_string_lossy().to_string())
                                        .clicked()
                                    {
                                        // Open the file
                                        let absolute_path = get_absolute_path(res.get_path());
                                        open::that(absolute_path).ok();
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(res.get_type_name().to_string());
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", res.get_status()));
                                });
                                row.col(|ui| {
                                    if ui.button("Reload").clicked() {
                                        let gl: Arc<glow::Context> = self.gl.clone();
                                        resources.reload(
                                            id,
                                            gl,
                                            game.lua_env.lua.clone(),
                                            game.lua_env.default_events.resource_loaded_event,
                                        );
                                    }
                                    let mut config = self.config.borrow_mut();
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
