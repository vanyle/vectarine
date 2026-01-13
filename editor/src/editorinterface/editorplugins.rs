use runtime::egui;

use crate::editorinterface::EditorState;

pub fn draw_editor_plugin_manager(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow_mut().is_plugins_window_shown;

    if editor.config.borrow().is_plugins_window_shown {
        let window = egui::Window::new("Plugin manager")
            .default_height(200.0)
            .default_width(300.0)
            .open(&mut is_shown)
            .collapsible(false)
            .vscroll(false);
        let response = window.show(ctx, |ui| {
            draw_editor_plugin_manager_content(editor, ui);
        });
        if let Some(response) = response {
            let on_top = Some(response.response.layer_id) == ctx.top_layer_id();
            if on_top && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
            {
                is_shown = false;
            }
        }
    }
    editor.config.borrow_mut().is_plugins_window_shown = is_shown;
}

fn draw_editor_plugin_manager_content(_editor: &mut EditorState, ui: &mut egui::Ui) {
    ui.label("No plugins found").on_hover_text("Plugins are programs that extend Vectarine's functionality. They are files ending with '.vecta.plugin'. You can download plugins or create them using the template provided by Vectarine GitHub repository.");
    if ui.button("Open plugin folder")
        .on_hover_text("Open the folder where plugins are stored. You can add plugins there and they will appear in the list of available plugins.")
        .clicked(){
            // let _ = open::that("plugins");
            // ...
        }
}
