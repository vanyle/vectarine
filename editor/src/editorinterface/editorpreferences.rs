use std::sync::atomic::{AtomicBool, Ordering};

use runtime::egui;

use crate::editorinterface::EditorState;

use crate::editorconfig::{TextEditor, WindowStyle};

pub fn draw_editor_preferences(editor: &mut EditorState, ui: &mut egui::Ui) {
    let mut is_shown = editor.config.borrow().is_preferences_window_shown;
    static HAS_UNSAVED_CHANGES: AtomicBool = AtomicBool::new(false);

    if is_shown {
        egui::Window::new("Preferences")
            .open(&mut is_shown)
            .resizable(true)
            .default_width(300.0)
            .show(ui, |ui| {
                ui.heading("General");
                {
                    let mut config = editor.config.borrow_mut();
                    let response = ui.checkbox(&mut config.is_always_on_top, "Game always on top");
                    if response.changed() {
                        HAS_UNSAVED_CHANGES.store(true, Ordering::Relaxed);
                    }
                    if response.clicked() {
                        editor
                            .window
                            .borrow_mut()
                            .set_always_on_top(config.is_always_on_top);
                    }
                }

                if editor.config.borrow().window_style == WindowStyle::GameSeparateFromEditor {
                    let mut config = editor.config.borrow_mut();
                    let response =
                        ui.checkbox(&mut config.is_editor_always_on_top, "Editor always on top");
                    if response.changed() {
                        HAS_UNSAVED_CHANGES.store(true, Ordering::Relaxed);
                    }
                    if response.clicked() {
                        editor
                            .editor_specific_window
                            .set_always_on_top(config.is_editor_always_on_top);
                    }
                }

                {
                    let mut config = editor.config.borrow_mut();
                    let mut window_style = config.window_style == WindowStyle::GameWithEditor;
                    let response = ui.checkbox(&mut window_style, "Merge editor and game windows");
                    if response.changed() {
                        HAS_UNSAVED_CHANGES.store(true, Ordering::Relaxed);
                    }
                    if response.clicked() {
                        config.window_style = if window_style {
                            WindowStyle::GameWithEditor
                        } else {
                            WindowStyle::GameSeparateFromEditor
                        };
                    }
                }

                ui.separator();
                ui.heading("External Editor");
                ui.label("Select the default editor used to open scripts.");

                {
                    let mut config = editor.config.borrow_mut();
                    let current_editor = config.text_editor;

                    egui::ComboBox::new("editor_selector", "")
                        .selected_text(format!("{}", current_editor.unwrap_or_default()))
                        .show_ui(ui, |ui| {
                            let editors = [
                                TextEditor::VSCode,
                                TextEditor::Antigravity,
                                TextEditor::Cursor,
                                TextEditor::Zed,
                                TextEditor::SublimeText,
                                TextEditor::Vim,
                                TextEditor::Neovim,
                                TextEditor::Emacs,
                            ];

                            for editor_option in editors {
                                if ui
                                    .selectable_value(
                                        &mut config.text_editor,
                                        Some(editor_option),
                                        format!("{}", editor_option),
                                    )
                                    .changed()
                                {
                                    HAS_UNSAVED_CHANGES.store(true, Ordering::Relaxed);
                                }
                            }
                        });
                }

                if HAS_UNSAVED_CHANGES.load(Ordering::Relaxed) {
                    ui.add_space(10.0);
                    ui.separator();
                    if ui.button("Save preferences").clicked() {
                        editor.save_config();
                        HAS_UNSAVED_CHANGES.store(false, Ordering::Relaxed);
                    }
                }
            });

        editor.config.borrow_mut().is_preferences_window_shown = is_shown;
    }
}
