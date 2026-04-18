use std::path::Path;
use std::sync::{LazyLock, Mutex};

use regex::Regex;
use runtime::console;
use runtime::console::ConsoleMessage;
use runtime::console::LuaError;
use runtime::egui;
use runtime::egui::{RichText, Widget};
use runtime::game::Game;
use runtime::lua_env::to_lua;

use crate::editorconfig::TextEditor;
use crate::editorinterface::EditorState;
use crate::editorinterface::extra::openfileatline::open_file_at_line;

pub fn draw_editor_console(editor: &mut EditorState, ui: &egui::Ui) {
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
        let response = window.show(ui, |ui| {
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

                egui::Panel::bottom("bottom_panel")
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
                    let prefered_text_editor = editor.config.borrow().text_editor;
                    draw_console_content(ui, project_dir.as_deref(), prefered_text_editor);
                });
        });
        if let Some(response) = response {
            let on_top = Some(response.response.layer_id) == ui.top_layer_id();
            if on_top && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
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
        to_lua(&game.lua_env.lua_handle.lua, command).expect("Failed to convert command to lua"),
    );
}

fn draw_console_content(
    ui: &mut egui::Ui,
    project_path: Option<&Path>,
    prefered_text_editor: Option<TextEditor>,
) {
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
                    ConsoleMessage::LuaError(msg) => {
                        render_lua_error(ui, msg, project_path, prefered_text_editor)
                    }
                    ConsoleMessage::Reload => {
                        ui.separator();
                    }
                };
            });
        });
}

fn render_lua_error(
    ui: &mut egui::Ui,
    error: &LuaError,
    project_path: Option<&Path>,
    prefered_text_editor: Option<TextEditor>,
) {
    error.line_content.iter().enumerate().for_each(|(i, line)| {
        let line_color = if i == 2 {
            egui::Color32::RED
        } else {
            egui::Color32::WHITE
        };
        let label = ui
            .label(
                RichText::new(format!("{}: {}", i + error.line - 2, &line))
                    .color(line_color)
                    .monospace(),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if label.clicked() {
            let Some(project_path) = project_path else {
                return;
            };
            let file = project_path.join(&error.file);
            if file.exists() {
                open_file_at_line(&file, error.line, prefered_text_editor);
            }
        }
    });

    if let Some(project_path) = project_path {
        let mut lines = error.message.split('\n');
        if let Some(first_line) = lines.next() {
            render_error_line_with_links(ui, first_line, error, project_path, prefered_text_editor);
        }
        for line in lines {
            ui.label(RichText::new(line).color(egui::Color32::RED).monospace());
        }
    } else {
        ui.label(
            RichText::new(&error.message)
                .color(egui::Color32::RED)
                .monospace(),
        );
    }
}

fn render_error_line_with_links(
    ui: &mut egui::Ui,
    line: &str,
    error: &LuaError,
    project_path: &Path,
    prefered_text_editor: Option<TextEditor>,
) {
    // Render error message, with clickable file:line links on the first line
    static FILE_LINE_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"([^\s:]+\.\w+):(\d+)").expect("Regex to be valid"));

    let matches: Vec<_> = FILE_LINE_RE.find_iter(line).collect();
    if matches.is_empty() {
        ui.label(RichText::new(line).color(egui::Color32::RED).monospace());
        return;
    }

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let mut last_end = 0;
        for m in &matches {
            if m.start() > last_end {
                ui.label(
                    RichText::new(&line[last_end..m.start()])
                        .color(egui::Color32::RED)
                        .monospace(),
                );
            }

            let link = ui
                .label(
                    RichText::new(m.as_str())
                        .color(egui::Color32::LIGHT_BLUE)
                        .monospace(),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(format!("Open {}", m.as_str()));

            if link.hovered() {
                link.clone().highlight();
            }

            if link.clicked() {
                let file = project_path.join(&error.file);
                if file.exists() {
                    open_file_at_line(&file, error.line, prefered_text_editor);
                }
            }

            last_end = m.end();
        }

        if last_end < line.len() {
            ui.label(
                RichText::new(&line[last_end..])
                    .color(egui::Color32::RED)
                    .monospace(),
            );
        }
    });
}
