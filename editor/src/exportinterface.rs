use std::cell::RefCell;

use crate::editorinterface::EditorState;

pub fn draw_editor_export(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_export_window_shown;

    egui::Window::new("Export Project")
        .default_width(600.0)
        .default_height(400.0)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut is_shown)
        .show(ctx, |ui| {
            draw_editor_export_window(ui, editor);
        });
    editor.config.borrow_mut().is_export_window_shown = is_shown;
}

fn draw_editor_export_window(ui: &mut egui::Ui, editor: &mut EditorState) {
    let mut project = editor.project.borrow_mut();
    let project = project.as_mut();
    let Some(project) = project else {
        ui.label("No project is currently loaded.");
        return;
    };

    let project_file_path = &mut project.project_path;
    let project_folder = project_file_path.parent().unwrap();

    thread_local! {
        static OBFUSCATE_GAME_DATA: RefCell<bool> = const { RefCell::new(true) };
    }

    ui.label("Export your game to a single ZIP file to share it!");
    OBFUSCATE_GAME_DATA.with_borrow_mut(|obfuscate_game_data| {
        const OBFUSCATION_INFO: &str = "
Obfuscation compresses your game and makes it run faster. \
The content of the distributed version becomes unreadable and uneditable by third-parties.
Read the manual section about obfuscation for more details.
        ";
        ui.checkbox(obfuscate_game_data, "Obfuscate game data")
            .on_hover_text(OBFUSCATION_INFO);
    });
    ui.label(format!("Export folder: {}", project_folder.display()));
    if ui
        .button("Open export folder")
        .on_hover_text("Open the folder where the exported game will be saved.")
        .clicked()
    {
        let _ = open::that(project_folder);
    }

    if ui.button("Export").clicked() {
        println!(
            "Export not implemented yet. You can manually zip the game data together with the executable for now."
        );
    }
}
