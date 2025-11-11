use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use egui::{Align, Frame, Layout, RichText, Sense, Stroke, UiBuilder};
use egui_extras::{Size, StripBuilder};
use runtime::{
    io::localfs::LocalFileSystem,
    projectinfo::{ProjectInfo, get_project_info},
};

use crate::editorinterface::EditorState;

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
    // ...
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
