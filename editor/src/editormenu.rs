use egui::RichText;

use crate::editorinterface::EditorState;

pub fn draw_editor_menu(editor: &mut EditorState, ctx: &egui::Context) {
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
