use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use egui_extras::{Size, StripBuilder};
use runtime::egui;
use runtime::egui::{Align, Frame, Layout, RichText, Sense, Stroke, UiBuilder};
use runtime::{
    io::localfs::LocalFileSystem,
    projectinfo::{ProjectInfo, get_project_info},
};
use vectarine_cli::{project::createproject::create_game_and_get_path, regex::Regex};

use crate::editorinterface::EditorState;
use vectarine_cli::project::geteditorpaths::{get_end_of_path, get_gallery_path};

pub fn draw_empty_screen(state: &mut EditorState, ui: &mut egui::Ui) {
    thread_local! {
        static NEW_GAME_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    }

    egui::Window::new("No project loaded")
        .default_width(384.0)
        .default_height(512.0)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder().at_most(512.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        NEW_GAME_PATH.with_borrow_mut(|new_game_path| {
                            let mut reset_path = false;
                            if let Some(new_game_path) = new_game_path.as_ref() {
                                reset_path = draw_new_game_window_content(state, ui, new_game_path);
                            } else {
                                draw_empty_screen_window_content(state, ui, new_game_path);
                            }
                            if reset_path {
                                new_game_path.take();
                            }
                        });
                    });
                });
        });
}

pub fn draw_empty_screen_window_content(
    state: &mut EditorState,
    ui: &mut egui::Ui,
    new_game_path: &mut Option<PathBuf>,
) {
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
                *new_game_path = open_folder_dialog_and_create_project(state);
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

        thread_local! {
            static IS_GALLERY_SHOWN: RefCell<bool> = const {RefCell::new(true)};
        }

        ui.add_space(8.0);
        
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                let gallery_label = {
                    let text = RichText::new("Gallery").size(24.0);
                    let label = ui.selectable_label(IS_GALLERY_SHOWN.with_borrow(|b| *b), text);
                    if IS_GALLERY_SHOWN.with_borrow(|b| *b) {
                        label.on_hover_cursor(egui::CursorIcon::Default)
                    } else {
                        label.on_hover_cursor(egui::CursorIcon::PointingHand)
                    }
                };
                let recent_projects_label = {
                    let text = RichText::new("Recent projects").size(24.0);
                    let label = ui.selectable_label(!IS_GALLERY_SHOWN.with_borrow(|b| *b), text);
                    if !IS_GALLERY_SHOWN.with_borrow(|b| *b) {
                        label.on_hover_cursor(egui::CursorIcon::Default)
                    } else {
                        label.on_hover_cursor(egui::CursorIcon::PointingHand)
                    }
                };
                if gallery_label.on_hover_text_at_pointer(
                "The gallery contains example projects and template to get started quickly and learn features of Vectarine."
                ).clicked() {
                    IS_GALLERY_SHOWN.replace(true);
                }
                ui.add_space(16.0);
                if recent_projects_label.on_hover_text_at_pointer(
                "The recent projects section shows projects you have worked on recently."
                ).clicked() {
                    IS_GALLERY_SHOWN.replace(false);
                }
            });
            ui.add_space(4.0);
            if IS_GALLERY_SHOWN.with_borrow(|b| *b) {
                draw_gallery(state, ui);
            }else {
                draw_recent_projects(state, ui);
            }
        });
    });
}

pub fn draw_new_game_window_content(
    state: &mut EditorState,
    ui: &mut egui::Ui,
    new_game_path: &Path,
) -> bool {
    ui.label(RichText::new("Create a new project").size(24.0));
    ui.add_space(8.0);
    {
        let end_of_path = get_end_of_path(new_game_path);
        let label =
            egui::Label::new(RichText::new(end_of_path)).wrap_mode(egui::TextWrapMode::Truncate);
        ui.label(RichText::new("Game folder created in").strong());
        ui.add(label);
    }

    thread_local! {
        static GAME_NAME: RefCell<String> = const {RefCell::new(String::new())}
    }

    const ERRORS: [&str; 2] = [
        "The name cannot be empty",
        "The name must only contain spaces, letters, numbers, dashes and underscores",
    ];
    let mut error_idx: Option<usize> = None;

    ui.label(RichText::new("Name of the game").strong());
    GAME_NAME.with_borrow_mut(|game_name| {
        ui.text_edit_singleline(game_name);
        if game_name.is_empty() {
            error_idx = Some(0);
        } else {
            let regex = Regex::new(r"^[A-Za-z0-9_\- ]+$").expect("Unable to create regex");
            if !regex.is_match(game_name) {
                error_idx = Some(1);
            }
        }
    });
    if let Some(error_idx) = error_idx {
        ui.label(
            RichText::new(ERRORS[error_idx])
                .color(egui::Color32::DARK_RED)
                .size(12.0),
        );
    }
    let mut exit_new_game_window = false;
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        if ui.button("Create the game and open it!").clicked() {
            GAME_NAME.with_borrow(|game_name| {
                let result_path = create_game_and_get_path(game_name, new_game_path);
                match result_path {
                    Ok(project_file_path) => {
                        state.load_project(
                            Box::new(LocalFileSystem),
                            &project_file_path,
                            |result| {
                                if let Err(e) = result {
                                    // TODO: show error in GUI
                                    println!("Failed to load project: {e}");
                                }
                            },
                        );
                    }
                    Err(e) => {
                        eprintln!("Error creating game: {:?}", e);
                    }
                }
            });
            exit_new_game_window = true;
        }
        if ui.button(RichText::new("Cancel")).clicked() {
            exit_new_game_window = true;
        }
    });
    exit_new_game_window
}

pub fn open_folder_dialog_and_create_project(state: &mut EditorState) -> Option<PathBuf> {
    state.window.borrow_mut().window.set_always_on_top(false); // prevent editor from being over the file picker.
    let path = rfd::FileDialog::new()
        .set_title("Select a location where the Vectarine project folder will be created")
        .pick_folder();
    state
        .window
        .borrow_mut()
        .window
        .set_always_on_top(state.config.borrow().is_always_on_top);
    path
}

pub fn open_file_dialog_and_load_project(state: &mut EditorState) {
    state.window.borrow_mut().window.set_always_on_top(false); // prevent editor from being over the file picker.
    let path = rfd::FileDialog::new()
        .add_filter("Vectarine Project", &["vecta", "toml"])
        .set_title("Open Vectarine Project")
        .pick_file();
    state
        .window
        .borrow_mut()
        .window
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
        static INITIALIZED: RefCell<bool> = const { RefCell::new(false) };
        static GALLERY_PROJECTS: RefCell<Vec<(PathBuf, ProjectInfo)>> = const { RefCell::new(vec![]) };
    }
    let is_initialized = INITIALIZED.with_borrow(|id| *id);
    if !is_initialized {
        let gallery_path = get_gallery_path();
        let Ok(entries) = std::fs::read_dir(&gallery_path) else {
            println!("Failed to read gallery directory at {:?}.", gallery_path);
            INITIALIZED.with_borrow_mut(|id| *id = true);
            return;
        };
        let gallery_projects = make_project_list_from_paths(entries.flatten().map(|e| e.path()));
        GALLERY_PROJECTS.replace(gallery_projects);
        INITIALIZED.with_borrow_mut(|id| *id = true);
    }

    GALLERY_PROJECTS.with_borrow(|gallery_paths|{
        if gallery_paths.is_empty() {
            ui.label("No gallery projects found. The gallery folder might be missing or empty.");
            ui.label(format!("The gallery folder is located at {}", get_gallery_path().display()));
        } else {
            draw_project_list(state, ui, gallery_paths);
        }
    });
}

pub fn draw_recent_projects(state: &mut EditorState, ui: &mut egui::Ui) {
    thread_local! {
        static INITIALIZED: RefCell<bool> = const { RefCell::new(false) };
        static RECENT_PROJECTS: RefCell<Vec<(PathBuf, ProjectInfo)>> = const { RefCell::new(vec![]) };
    }

    let is_initialized = INITIALIZED.with_borrow(|id| *id);
    if !is_initialized {
        let config = state.config.borrow_mut();
        let recent_projects_as_string = &config.recent_project_paths;
        
        let recent_projects = make_project_list_from_paths(recent_projects_as_string.iter().map(PathBuf::from));
        RECENT_PROJECTS.replace(recent_projects);
        INITIALIZED.with_borrow_mut(|id| *id = true);
    }
    
        RECENT_PROJECTS.with_borrow(|recent_project_paths|{
        if recent_project_paths.is_empty() {
            ui.label("No recent projects found.");
        } else {
            draw_project_list(state, ui, recent_project_paths);
        }
    });
}

pub fn draw_project_list(state: &mut EditorState, ui: &mut egui::Ui, project_infos: &[(PathBuf, ProjectInfo)]) {
    // Draw the list of projects
    egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("project_list")
                .num_columns(1)
                .spacing([0.0, 8.0])
                .show(ui, |ui| {
                    for (project_folder, project_info) in project_infos.iter().cloned() {
                        ui.scope_builder(
                            UiBuilder::new()
                                .id_salt("interactive_container")
                                .sense(Sense::click()),
                            |ui| {
                                let response = ui.response();
                                let bg_fill = ui.style().interact(&response).bg_fill;
                                let rect = response.rect;
                                let layer_id = response.layer_id;
                                let is_hovering = {
                                    rect.is_positive() && {
                                        let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                                        if let Some(pointer_pos) = pointer_pos {
                                            rect.contains(pointer_pos)
                                                && ui.layer_id_at(pointer_pos) == Some(layer_id)
                                        } else {
                                            false
                                        }
                                    }
                                };
                                let stroke = if is_hovering {
                                    Stroke::new(2.0_f32, egui::Color32::WHITE)
                                } else {
                                    Stroke::new(2.0_f32, egui::Color32::TRANSPARENT)
                                };
                                let mut is_clicked = false;

                                Frame::canvas(ui.style())
                                    .fill(bg_fill.gamma_multiply(0.3))
                                    .stroke(stroke)
                                    .show(ui, |ui| {
                                        ui.set_min_width(500.0);
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
                                            },
                                        );
                                    });
                                if response.clicked() || is_clicked {
                                    state.load_project(
                                        Box::new(LocalFileSystem),
                                        &project_folder.join("game.vecta"),
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
                        ui.end_row();
                    }
                });
    });
}

pub fn make_project_list_from_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<(PathBuf, ProjectInfo)> {
    let mut projects_infos = vec![];
    for path in paths {
        if !path.is_dir() {
            // println!("Project path {:?} is not a directory, skipping.", path);
            continue;
        }
        let project_file = path.join("game.vecta");
        if !project_file.is_file() {
            continue;
        }
        let project_manifest_content =
            std::fs::read_to_string(&project_file).unwrap_or_default();
        let project_info = get_project_info(&project_manifest_content);
        let Ok(project_info) = project_info else {
            println!(
                "Failed to parse project info for project at {:?}, skipping.",
                path
            );
            continue;
        };
        projects_infos.push((path.clone(), project_info));
    }
    projects_infos.sort_by(|a, b| a.1.title.cmp(&b.1.title));
    projects_infos
}

