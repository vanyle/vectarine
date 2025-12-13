use std::path::Path;
use std::process::Command;
use std::sync::{LazyLock, Mutex};

use egui::RichText;
use egui::Widget;
use runtime::console;
use runtime::console::ConsoleMessage;
use runtime::console::LuaError;
use runtime::game::Game;
use runtime::lua_env::to_lua;

use crate::editorinterface::EditorState;

pub fn draw_editor_console(editor: &mut EditorState, ctx: &egui::Context) {
    let mut project = editor.project.borrow_mut();
    let mut is_shown = editor.config.borrow_mut().is_console_shown;

    let project_dir = project
        .as_ref()
        .and_then(|proj| proj.project_path.parent())
        .map(|p| p.to_path_buf());

    let game = match project.as_mut() {
        Some(proj) => Some(&mut proj.game),
        None => None,
    };

    if editor.config.borrow().is_console_shown {
        let window = egui::Window::new("Console")
            .default_height(200.0)
            .default_width(300.0)
            .open(&mut is_shown)
            .collapsible(false)
            .vscroll(false);
        let response = window.show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let response = egui::TextEdit::singleline(&mut editor.text_command)
                        .hint_text("Enter command...")
                        .ui(ui);

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        try_send_command_to_game(&game, &editor.text_command);
                        editor.text_command.clear();
                        response.request_focus();
                    }
                    if egui::Button::new("Clear").ui(ui).clicked() {
                        console::clear_all_logs();
                    }
                });

                egui::TopBottomPanel::bottom("bottom_panel")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        ui.label(
                            RichText::new("Frame messages")
                                .size(14.0)
                                .color(egui::Color32::LIGHT_BLUE),
                        )
                        .on_hover_text("Frame messages are reset every frame. They are useful to debug things that happen every frame.");
                        egui::ScrollArea::vertical()
                            .id_salt("frame console")
                            .auto_shrink(false)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                console::consume_frame_logs(|msg| {
                                    ui.label(
                                        RichText::new(msg).color(egui::Color32::WHITE).monospace(),
                                    );
                                });
                            });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    draw_console_content(ui, project_dir.as_deref());
                });
        });
        if let Some(response) = response {
            let on_top = Some(response.response.layer_id) == ctx.top_layer_id();
            if on_top && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
            {
                is_shown = false;
            }
        }
        editor.config.borrow_mut().is_console_shown = is_shown;
    }
}

pub fn try_send_command_to_game(game: &Option<&mut Game>, command: &str) {
    let Some(game) = game else {
        return;
    };
    let _ = game.lua_env.default_events.console_command_event.trigger(
        to_lua(game.lua_env.lua.as_ref(), command).expect("Failed to convert command to lua"),
    );
}

fn draw_console_content(ui: &mut egui::Ui, project_path: Option<&Path>) {
    static ARE_LOGS_ERROR_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));
    static ARE_LOGS_WARN_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));
    static ARE_LOGS_INFO_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));

    ui.horizontal(|ui: &mut egui::Ui| {
        if let Ok(mut infos) = ARE_LOGS_INFO_SHOWN.lock() {
            ui.checkbox(&mut infos, "Infos");
        }
        if let Ok(mut warnings) = ARE_LOGS_WARN_SHOWN.lock() {
            ui.checkbox(&mut warnings, "Warnings");
        }
        if let Ok(mut errors) = ARE_LOGS_ERROR_SHOWN.lock() {
            ui.checkbox(&mut errors, "Errors");
        }
    });
    egui::ScrollArea::vertical()
        .id_salt("console")
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            let show_errors = ARE_LOGS_ERROR_SHOWN.lock().map(|e| *e).unwrap_or_default();
            let show_warnings = ARE_LOGS_WARN_SHOWN.lock().map(|e| *e).unwrap_or_default();
            let show_infos = ARE_LOGS_INFO_SHOWN.lock().map(|e| *e).unwrap_or_default();

            console::get_logs(|msg| {
                if matches!(msg, ConsoleMessage::Info(_)) && !show_infos {
                    return;
                }
                if matches!(msg, ConsoleMessage::Warning(_)) && !show_warnings {
                    return;
                }
                if matches!(msg, ConsoleMessage::Error(_) | ConsoleMessage::LuaError(_))
                    && !show_errors
                {
                    return;
                }
                match msg {
                    ConsoleMessage::Info(msg) => {
                        ui.label(
                            RichText::new(format!("{}", msg))
                                .color(egui::Color32::WHITE)
                                .monospace(),
                        );
                    }
                    ConsoleMessage::Warning(msg) => {
                        ui.label(
                            RichText::new(format!("{}", msg))
                                .color(egui::Color32::YELLOW)
                                .monospace(),
                        );
                    }
                    ConsoleMessage::Error(msg) => {
                        ui.label(
                            RichText::new(format!("{}", msg))
                                .color(egui::Color32::RED)
                                .monospace(),
                        );
                    }
                    ConsoleMessage::LuaError(msg) => render_lua_error(ui, msg, project_path),
                };
            });
        });
}

fn render_error_fallback(ui: &mut egui::Ui, error: &str) {
    ui.label(RichText::new(error).color(egui::Color32::RED).monospace());
}

fn render_lua_error(ui: &mut egui::Ui, error: &LuaError, project_path: Option<&Path>) {
    let Some(project_path) = project_path else {
        render_error_fallback(ui, &error.message);
        return;
    };
    let file = project_path.join(&error.file);
    if !file.exists() {
        render_error_fallback(ui, &error.message);
        return;
    }
    let content = std::fs::read_to_string(&file);
    let Ok(content) = content else {
        render_error_fallback(ui, &error.message);
        return;
    };
    content
        .lines()
        .skip(error.line - 3) // lines are 0-indexed, but error.line is 1-indexed
        .take(5)
        .enumerate()
        .for_each(|(i, line)| {
            let marker = if i == 2 { "=>" } else { "  " };
            let label = ui
                .label(
                    RichText::new(format!("{}:{}{}", i + error.line - 2, marker, &line))
                        .color(egui::Color32::WHITE)
                        .monospace(),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if label.clicked() {
                open_file_at_line(&file, error.line);
            }
        });

    render_error_fallback(ui, &error.message);
}

// There is no standard way to do this, so we try different editors
// Ideally the user should be able his preferred editor
// Roughly sorted by popularity (least to most popular)
fn open_file_at_line(file: &Path, line: usize) {
    // Antigravity
    let is_antigravity = which::which("antigravity").is_ok();
    if is_antigravity {
        let res = Command::new("antigravity")
            .args(["--goto", format!("{}:{}", file.display(), line).as_str()])
            .spawn();
        if res.is_ok() {
            return;
        }
    }

    // Sublime Text
    let is_sublime = which::which("subl").is_ok();
    if is_sublime {
        let res = Command::new("subl")
            .args([format!("{}:{}", file.display(), line).as_str()])
            .spawn();
        if res.is_ok() {
            return;
        }
    }

    // Zed
    let is_zed = which::which("zed").is_ok();
    if is_zed {
        let res = Command::new("zed")
            .args([format!("{}:{}", file.display(), line).as_str()])
            .spawn();
        if res.is_ok() {
            return;
        }
    }

    // VSCode
    let is_code = which::which("code").is_ok();
    if is_code {
        // code --goto "path/to/file:line"
        let res = Command::new("code")
            .args(["--goto", format!("{}:{}", file.display(), line).as_str()])
            .spawn();
        if res.is_ok() {
            return;
        }
    }

    let _ = open::that(file);
}
