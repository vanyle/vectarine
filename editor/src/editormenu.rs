use egui::{RichText, UiBuilder};

use crate::editorinterface::{EditorState, open_file_dialog_and_load_project};

pub fn draw_editor_menu(editor: &mut EditorState, ctx: &egui::Context) {
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num1)) {
        let mut config = editor.config.borrow_mut();
        config.is_console_shown = !config.is_console_shown;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num2)) {
        let mut config = editor.config.borrow_mut();
        config.is_resources_window_shown = !config.is_resources_window_shown;
    }

    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num3)) {
        let mut config = editor.config.borrow_mut();
        config.is_watcher_window_shown = !config.is_watcher_window_shown;
    }

    egui::TopBottomPanel::top("toppanel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Vectarine Editor").size(18.0));
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let exit_text = if cfg!(target_os = "macos") {
                        "Exit (Cmd+Q)"
                    } else {
                        "Exit (Alt+F4)"
                    };
                    if ui.button("Open project").clicked() {
                        open_file_dialog_and_load_project(editor);
                    }

                    let is_project_loaded = editor.project.borrow().is_some();
                    let mut ui_builder = UiBuilder::new();
                    if !is_project_loaded {
                        ui_builder = ui_builder.disabled();
                    }
                    ui.scope_builder(ui_builder, |ui| {
                        if ui.button("Close project").clicked() {
                            editor.close_project();
                        }

                        if ui.button("Export...").clicked() {
                            // TO-DO: implement export dialog
                            println!("Export not implemented yet. You can manually zip the game data together with the executable for now.");
                        }
                    });

                    if ui.button(exit_text).clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Tools", |ui| {
                    if ui.button("Console (Ctrl+1)").clicked() {
                        let mut config = editor.config.borrow_mut();
                        config.is_console_shown = !config.is_console_shown;
                    }
                    if ui.button("Resources (Ctrl+2)").clicked() {
                        let mut config = editor.config.borrow_mut();
                        config.is_resources_window_shown = !config.is_resources_window_shown;
                    }
                    if ui.button("Watcher (Ctrl+3)").clicked() {
                        let mut config = editor.config.borrow_mut();
                        config.is_watcher_window_shown = !config.is_watcher_window_shown;
                    }
                    {
                        let mut config = editor.config.borrow_mut();
                        if ui
                            .checkbox(&mut config.is_always_on_top, "Always on top")
                            .clicked()
                        {
                            editor
                                .window
                                .borrow_mut()
                                .set_always_on_top(config.is_always_on_top);
                        }
                    }
                    if ui.button("Save editor config").clicked() {
                        editor.save_config();
                    }
                });
            });
        });
        // let window_handle = editor.window.borrow().raw();
        // sdl2_sys::SDL_SetWindowHitTest(window_handle, callback, callback_data)
    });
}
