use egui::{Popup, RichText, UiBuilder};

use crate::editorinterface::{EditorState, emptyscreen::open_file_dialog_and_load_project};

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

    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::R)) {
        editor.reload_project();
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
                        if ui.button("Reload project (Ctrl+R)").clicked() {
                            editor.reload_project();
                        }

                        if ui.button("Close project").clicked() {
                            editor.close_project();
                        }

                        if ui.button("Export...").clicked() {
                            let mut config = editor.config.borrow_mut();
                            config.is_export_window_shown = true;
                        }
                    });

                    if ui.button(exit_text).clicked() {
                        std::process::exit(0);
                    }
                });
                let popup_menu = Popup::menu(&ui.button("Tools"))
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);

                popup_menu.show(|ui| {
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
