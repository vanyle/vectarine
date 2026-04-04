use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::Mutex;
use std::thread;

use runtime::egui;
use runtime::egui::{Color32, RichText, Widget};

use crate::editorinterface::EditorState;
use crate::export::exportproject::{ExportPlatform, export_project};

pub fn draw_editor_export(editor: &mut EditorState, ui: &mut egui::Ui) {
    let mut is_shown = editor.config.borrow().is_export_window_shown;

    egui::Window::new("Export Project")
        .default_width(600.0)
        .default_height(400.0)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut is_shown)
        .show(ui, |ui| {
            draw_editor_export_window(ui, editor);
        });
    editor.config.borrow_mut().is_export_window_shown = is_shown;
}

fn ui_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).heading());
}

fn draw_editor_export_window(ui: &mut egui::Ui, editor: &mut EditorState) {
    let mut project = editor.project.borrow_mut();
    let project = project.as_mut();
    let Some(project) = project else {
        ui.label("No project is currently loaded.");
        return;
    };

    let project_file_path = &mut project.project_path;
    let project_folder = project_file_path
        .parent()
        .expect("Failed to get project folder");

    thread_local! {
        static OBFUSCATE_GAME_DATA: RefCell<bool> = const { RefCell::new(true) };
        static TARGET_PLATFORM: RefCell<ExportPlatform> = const { RefCell::new(ExportPlatform::Web) };
    }

    ui_title(ui, "Optimization");

    OBFUSCATE_GAME_DATA.with_borrow_mut(|obfuscate_game_data| {
        const OBFUSCATION_INFO: &str = "
Obfuscation compresses your game and makes it run faster. \
The content of the distributed version becomes unreadable and uneditable by third-parties.
Read the manual section about obfuscation for more details.
        ";
        ui.checkbox(obfuscate_game_data, "Obfuscate game data")
            .on_hover_text(OBFUSCATION_INFO);
    });

    // -----------------
    ui.add_space(8.0);
    ui_title(ui, "Export platform");
    ui.horizontal_wrapped(|ui| {
        TARGET_PLATFORM.with_borrow_mut(|target_platform| {
            for platform in ExportPlatform::all() {
                ui.selectable_value(target_platform, platform, format!("{}", platform));
            }
        });
    });

    // -----------------
    ui.add_space(8.0);

    ui_title(ui, "Export folder");
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Open export folder")
            .on_hover_text("Open the folder where the exported game will be saved.")
            .clicked()
        {
            let _ = open::that(project_folder);
        }
        ui.label(
            RichText::new(project_folder.display().to_string())
                .monospace()
                .color(Color32::WHITE)
                .background_color(Color32::from_gray(0x22)),
        );
    });

    // -----------------
    ui.add_space(8.0);

    let export_button = egui::Button::new(RichText::new("Export").size(20.0));

    lazy_static! {
        static ref EXPORT_LOG_BUFFER: Mutex<String> = Mutex::new(String::new());
    }

    if export_button.ui(ui).clicked() {
        {
            let mut log_buffer = EXPORT_LOG_BUFFER.lock().expect("Failed to lock log buffer");
            log_buffer.clear();
        }
        let project_path = project.project_path.clone();
        let project_info = project.project_info.clone();
        let obfuscate_data = OBFUSCATE_GAME_DATA.with_borrow(|b| *b);
        let target_platform = TARGET_PLATFORM.with_borrow(|p| *p);

        thread::spawn(move || {
            let result = export_project(
                &project_path,
                &project_info,
                obfuscate_data,
                target_platform,
            );
            if let Err(err_msg) = result {
                let mut log_buffer = EXPORT_LOG_BUFFER.lock().expect("Failed to lock log buffer");
                *log_buffer = format!("Export failed: {}\n", err_msg);
            } else {
                let mut log_buffer = EXPORT_LOG_BUFFER.lock().expect("Failed to lock log buffer");
                *log_buffer = "Export completed successfully.\n".into();
            }
        });
    }
    {
        if let Ok(log_buffer) = EXPORT_LOG_BUFFER.try_lock()
            && !log_buffer.is_empty()
        {
            ui.add_space(8.0);
            ui.label(RichText::new(&*log_buffer).monospace());
        }
    }
}
