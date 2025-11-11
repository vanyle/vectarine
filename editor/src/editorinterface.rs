use std::{
    cell::RefCell,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

use egui::{Align, Frame, Layout, RichText, Sense, Stroke, UiBuilder};
use egui_extras::{Size, StripBuilder};
use egui_sdl2_platform::sdl2;
use glow::HasContext;
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache, new_debouncer, notify,
};
use runtime::{
    anyhow::{self},
    game::drawable_screen_size,
    game_resource::ResourceId,
    io::{
        fs::{FileSystem, ReadOnlyFileSystem},
        localfs::LocalFileSystem,
    },
    projectinfo::{ProjectInfo, get_project_info},
    toml,
};
use serde::{Deserialize, Serialize};

use crate::{
    editorconsole::draw_editor_console, editormenu::draw_editor_menu,
    editorresources::draw_editor_resources, editorwatcher::draw_editor_watcher, egui_sdl2_platform,
    exportinterface::draw_editor_export, projectstate::ProjectState,
};

const EDITOR_CONFIG_FILE: &str = "vectarine-config.toml";

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    pub is_console_shown: bool,
    pub is_resources_window_shown: bool,
    pub is_watcher_window_shown: bool,
    pub is_export_window_shown: bool,
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

    debouncer: Rc<RefCell<Debouncer<notify::RecommendedWatcher, RecommendedCache>>>,
}

impl EditorState {
    pub fn save_config(&self) {
        let config = &self.config.borrow();
        let data = toml::to_string(config.deref()).unwrap_or_default();
        LocalFileSystem.write_file(EDITOR_CONFIG_FILE, data.as_bytes(), Box::new(|_| {}));
    }

    /// Load the editor config from file.
    /// If `auto_start_project` is true, and there was a project opened previously, it is loaded automatically overwriting any current project.
    pub fn load_config(&self, auto_start_project: bool) {
        let config_store = self.config.clone();
        let project = self.project.clone();
        let gl = self.gl.clone();
        let video = self.video.clone();
        let window = self.window.clone();
        let debouncer = self.debouncer.clone();

        LocalFileSystem.read_file(
            EDITOR_CONFIG_FILE,
            Box::new(move |data: Option<Vec<u8>>| {
                let Some(data) = data else {
                    return; // no config file
                };
                if let Ok(config) = toml::from_slice::<EditorConfig>(data.as_slice()) {
                    let previous_project_path = config.opened_project_path.clone();
                    if let Some(project_path_str) = &previous_project_path {
                        let previous_project_path = PathBuf::from(project_path_str);
                        let parent = previous_project_path.parent();
                        if let Some(parent) = parent {
                            let _ = debouncer.borrow_mut().unwatch(parent);
                        }
                    }

                    *config_store.borrow_mut() = config;
                    if auto_start_project
                        && let Some(project_path_str) = &config_store.borrow().opened_project_path
                    {
                        let project_path = PathBuf::from(project_path_str);
                        let parent = project_path.parent();
                        if let Some(parent) = parent {
                            let _ = debouncer
                                .borrow_mut()
                                .watch(parent, notify::RecursiveMode::Recursive);
                        }

                        ProjectState::new(
                            &project_path,
                            Box::new(LocalFileSystem),
                            gl,
                            video,
                            window,
                            |loaded_project| {
                                if let Ok(loaded_project) = loaded_project {
                                    project.replace(Some(loaded_project));
                                }
                            },
                        );
                    }
                }
            }),
        );
    }

    pub fn new(
        video: Rc<RefCell<sdl2::VideoSubsystem>>,
        window: Rc<RefCell<sdl2::video::Window>>,
        gl: Arc<glow::Context>,
        debounce_event_sender: mpsc::Sender<DebouncedEvent>,
    ) -> Self {
        Self {
            config: Rc::new(RefCell::new(EditorConfig::default())),
            text_command: String::new(),
            start_time: Instant::now(),
            project: Rc::new(RefCell::new(None)),
            video,
            window,
            gl,
            debouncer: Rc::new(RefCell::new(
                new_debouncer(
                    Duration::from_millis(10),
                    None,
                    move |result: DebounceEventResult| match result {
                        Ok(events) => events.iter().for_each(|event| {
                            let _ = debounce_event_sender.send(event.clone());
                        }),
                        Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
                    },
                )
                .unwrap(),
            )),
        }
    }

    pub fn load_project<F>(
        &self,
        file_system: Box<dyn ReadOnlyFileSystem>,
        project_path: &Path,
        callback: F,
    ) where
        F: FnOnce(anyhow::Result<()>),
    {
        ProjectState::new(
            project_path,
            file_system,
            self.gl.clone(),
            self.video.clone(),
            self.window.clone(),
            |project| {
                match project {
                    Ok(p) => self.project.borrow_mut().replace(p),
                    Err(e) => {
                        callback(Err(e));
                        return;
                    }
                };
                self.config.borrow_mut().opened_project_path =
                    Some(project_path.to_string_lossy().to_string());

                let parent = project_path.parent();
                if let Some(parent) = parent {
                    // This only makes sense for a local file system.
                    let _ = self
                        .debouncer
                        .borrow_mut()
                        .watch(parent, notify::RecursiveMode::Recursive);
                }
                self.save_config();
                callback(Ok(()));
            },
        );
    }

    pub fn reload_project(&mut self) {
        if let Some(proj) = &mut *self.project.borrow_mut() {
            proj.reload();
        }
    }

    pub fn close_project(&mut self) {
        if let Some(proj) = &*self.project.borrow() {
            let project_path = &proj.project_path;
            let parent = project_path.parent();
            if let Some(parent) = parent {
                let _ = self.debouncer.borrow_mut().unwatch(parent);
            }
        }

        self.project.borrow_mut().take();
        self.config.borrow_mut().opened_project_path = None;
        self.save_config();
    }

    pub fn draw_editor_interface(
        &mut self,
        platform: &mut egui_sdl2_platform::Platform,
        sdl: &sdl2::Sdl,
        latest_events: &[sdl2::event::Event],
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
        draw_editor_export(self, &ctx);

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
    state.window.borrow_mut().set_always_on_top(false); // prevent editor from being over the file picker.
    let path = rfd::FileDialog::new()
        .add_filter("Vectarine Project", &["vecta", "toml"])
        .set_title("Open Vectarine Project")
        .pick_file();
    state
        .window
        .borrow_mut()
        .set_always_on_top(state.config.borrow().is_always_on_top);

    let Some(path) = path else {
        return;
    };
    state.load_project(Box::new(LocalFileSystem), &path, |result| {
        if let Err(e) = result {
            // TO-DO: show error in GUI
            println!("Failed to load project: {e}");
        }
    });
}

pub fn get_gallery_path() -> PathBuf {
    let executable_path = std::env::current_exe().unwrap_or_default();
    let executable_parent = executable_path.parent().unwrap_or(Path::new("."));
    let gallery_path = executable_parent.join("gallery");
    if gallery_path.is_dir() {
        return gallery_path;
    }
    PathBuf::from("gallery")
}

pub fn trim_string_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut trimmed = s[..max_len].to_string();
        trimmed.push_str("...");
        trimmed
    }
}

pub fn draw_gallery(state: &mut EditorState, ui: &mut egui::Ui) {
    thread_local! {
        static GALLERY_PROJECTS: RefCell<Vec<(PathBuf, ProjectInfo)>> = const { RefCell::new(vec![]) };
        static INITIALIZED: RefCell<bool> = const { RefCell::new(false) };
    }

    // Initialize the gallery if needed
    GALLERY_PROJECTS.with_borrow_mut(|gallery_projects| {
        if !INITIALIZED.with_borrow(|id| *id) {
            let gallery_path = get_gallery_path();
            let Ok(entries) = std::fs::read_dir(&gallery_path) else {
                println!("Failed to read gallery directory at {:?}.", gallery_path);
                INITIALIZED.with_borrow_mut(|id| *id = true);
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let project_file = path.join("game.vecta");

                if !project_file.is_file() {
                    println!(
                        "Gallery project at {:?} is missing game.vecta file, skipping.",
                        path
                    );
                    continue;
                }
                let project_manifest_content =
                    std::fs::read_to_string(&project_file).unwrap_or_default();
                let project_info = get_project_info(&project_manifest_content);
                let Ok(project_info) = project_info else {
                    println!(
                        "Failed to parse project info for gallery project at {:?}, skipping.",
                        path
                    );
                    continue;
                };
                gallery_projects.push((project_file, project_info));
            }
            INITIALIZED.with_borrow_mut(|id| *id = true);
        }
    });

    // Draw the gallery projects
    GALLERY_PROJECTS.with_borrow(|gallery_projects| {
        StripBuilder::new(ui)
            .size(Size::initial(20.0))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    for (project_file, project_info) in gallery_projects.iter().cloned() {
                        ui.scope_builder(
                            UiBuilder::new()
                                .id_salt("interactive_container")
                                .sense(Sense::click()),
                            |ui| {
                                let response = ui.response();
                                let visuals = ui.style().interact(&response);
                                let rect = response.rect;
                                let layer_id = response.layer_id;
                                let is_hovering = {
                                    rect.is_positive() && {
                                        let pointer_pos =
                                            ui.ctx().input(|i| i.pointer.interact_pos());
                                        if let Some(pointer_pos) = pointer_pos {
                                            rect.contains(pointer_pos)
                                                && ui.ctx().layer_id_at(pointer_pos)
                                                    == Some(layer_id)
                                        } else {
                                            false
                                        }
                                    }
                                };
                                let stroke = if is_hovering {
                                    Stroke::new(2.0, egui::Color32::WHITE)
                                } else {
                                    Stroke::new(2.0, egui::Color32::TRANSPARENT)
                                };
                                let mut is_clicked = false;

                                Frame::canvas(ui.style())
                                    .fill(visuals.bg_fill.gamma_multiply(0.3))
                                    .stroke(stroke)
                                    .show(ui, |ui| {
                                        ui.with_layout(
                                            Layout::left_to_right(Align::Center),
                                            |ui| {
                                                ui.vertical(|ui| {
                                                    let label_response = ui.label(
                                                        RichText::new(project_info.title)
                                                            .strong()
                                                            .size(18.0),
                                                    );
                                                    is_clicked |= label_response.clicked();
                                                    let description = trim_string_with_ellipsis(
                                                        &project_info.description,
                                                        80,
                                                    );
                                                    let label_response = ui.label(
                                                        RichText::new(description).size(12.0),
                                                    );
                                                    is_clicked |= label_response.clicked();
                                                });
                                                let label_response = ui.label(
                                                    RichText::new(format!(
                                                        "{}",
                                                        project_file.display()
                                                    ))
                                                    .size(12.0),
                                                );
                                                is_clicked |= label_response.clicked();
                                            },
                                        );
                                    });
                                if response.clicked() || is_clicked {
                                    state.load_project(
                                        Box::new(LocalFileSystem),
                                        &project_file,
                                        |result| {
                                            if let Err(e) = result {
                                                // TO-DO: show error in GUI
                                                println!("Failed to load project: {e}");
                                            }
                                        },
                                    );
                                }
                            },
                        );
                    }
                });
            });
    });
}

pub fn open_folder_dialog_and_create_project(state: &mut EditorState) {
    state.window.borrow_mut().set_always_on_top(false); // prevent editor from being over the file picker.
    let path = rfd::FileDialog::new()
        .set_title("Select a location where the Vectarine project folder will be created")
        .pick_folder();
    state
        .window
        .borrow_mut()
        .set_always_on_top(state.config.borrow().is_always_on_top);
    let Some(path) = path else {
        return;
    };
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
                        draw_empty_screen_window_content(state, ui);
                    });
                });
        });
}

pub fn draw_empty_screen_window_content(state: &mut EditorState, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.label(RichText::new("Welcome to Vectarine").size(24.0));
        });
        ui.add_space(8.0);
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);
            if ui
                .button(RichText::new("Create new Project").size(18.0))
                .clicked()
            {
                open_folder_dialog_and_create_project(state);
            }
            ui.add_space(8.0);
            if ui
                .button(RichText::new("Open Existing Project").size(18.0))
                .on_hover_text_at_pointer(
                "Vectarine projects are stored as files with the .vecta extension, they are usually called game.vecta"
            )
                .clicked()
            {
                open_file_dialog_and_load_project(state);
            }
            ui.style_mut().spacing.button_padding =
                egui::Spacing::default().button_padding;
        });
        if false {
            ui.add_space(8.0);
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.label(RichText::new("Recent projects").size(18.0));
                ui.add_space(4.0);
                ui.label(RichText::new("No recent projects found").size(14.0));
            });
        }
        ui.add_space(8.0);

        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.label(RichText::new("Gallery").size(24.0)).on_hover_text_at_pointer(
                "The gallery contains example projects and template to get started quickly and learn features of Vectarine."
            );
            ui.add_space(4.0);
            draw_gallery(state, ui);
        });
    });
}

pub fn process_events_when_no_game(latest_events: &[sdl2::event::Event], gl: &glow::Context) {
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
