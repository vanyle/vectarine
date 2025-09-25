use std::sync::{LazyLock, Mutex};

use egui::RichText;
use egui::Widget;
use runtime::console::Verbosity;
use runtime::game::Game;
use runtime::lua_env::to_lua;

use crate::editorinterface::EditorState;

pub fn draw_editor_console(editor: &mut EditorState, game: &mut Game, ctx: &egui::Context) {
    if editor.config.borrow().is_console_shown {
        egui::Window::new("Console")
            .default_height(200.0)
            .default_width(300.0)
            .vscroll(false)
            .show(ctx, |ui| {
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
                                let messages = &mut game.lua_env.frame_messages.borrow_mut();
                                for line in messages.iter() {
                                    let msg = &line.msg;
                                    ui.label(
                                        RichText::new(msg).color(egui::Color32::WHITE).monospace(),
                                    );
                                }
                                messages.clear();
                            });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        let response = egui::TextEdit::singleline(&mut editor.text_command)
                            .hint_text("Enter command...")
                            .ui(ui);
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let _ = game.lua_env.default_events.console_command_event.trigger(
                                game.lua_env.lua.as_ref(),
                                to_lua(game.lua_env.lua.as_ref(), editor.text_command.clone())
                                    .unwrap(),
                            );
                            editor.text_command.clear();
                            response.request_focus();
                        }
                        if egui::Button::new("Clear").ui(ui).clicked() {
                            game.lua_env.messages.borrow_mut().clear();
                        }
                    });

                    draw_console_content(ui, game);
                });
            });
    }
}

fn draw_console_content(ui: &mut egui::Ui, game: &mut Game) {
    static ARE_LOGS_ERROR_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));
    static ARE_LOGS_WARN_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));
    static ARE_LOGS_INFO_SHOWN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true));

    ui.horizontal(|ui: &mut egui::Ui| {
        ui.checkbox(&mut ARE_LOGS_INFO_SHOWN.lock().unwrap(), "Infos");
        ui.checkbox(&mut ARE_LOGS_WARN_SHOWN.lock().unwrap(), "Warnings");
        ui.checkbox(&mut ARE_LOGS_ERROR_SHOWN.lock().unwrap(), "Errors");
    });
    egui::ScrollArea::vertical()
        .id_salt("console")
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            let show_errors = *ARE_LOGS_ERROR_SHOWN.lock().unwrap();
            let show_warnings = *ARE_LOGS_WARN_SHOWN.lock().unwrap();
            let show_infos = *ARE_LOGS_INFO_SHOWN.lock().unwrap();

            let messages = &mut game.lua_env.messages.borrow_mut();
            for line in messages.iter().rev() {
                let msg = &line.msg;
                let is_error = line.verbosity == Verbosity::Error;
                let is_warning = line.verbosity == Verbosity::Warn;
                let is_info = line.verbosity == Verbosity::Info;
                if (show_errors && is_error)
                    || (show_warnings && is_warning)
                    || (show_infos && is_info)
                {
                    let text = if is_error {
                        RichText::new(msg).color(egui::Color32::RED)
                    } else if is_warning {
                        RichText::new(msg).color(egui::Color32::YELLOW)
                    } else {
                        RichText::new(msg).color(egui::Color32::WHITE)
                    };
                    ui.label(text.monospace());
                }
            }
            messages.truncate(500);
        });
}
