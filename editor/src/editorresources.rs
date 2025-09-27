use std::sync::Arc;

use egui::ScrollArea;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use runtime::{game::Game, game_resource::get_absolute_path};

use crate::editorinterface::EditorState;

pub fn draw_editor_resources(editor: &mut EditorState, game: &mut Game, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_resources_window_shown;
    egui::Window::new("Resources")
        .default_width(400.0)
        .default_height(200.0)
        .open(&mut is_shown)
        .show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0)) // for the table
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ScrollArea::horizontal().show(ui, |ui| {
                            draw_resource_table(editor, ui, game);
                        });
                    });
                });
        });

    editor.config.borrow_mut().is_resources_window_shown = is_shown;

    if let Some(id) = editor.config.borrow().debug_resource_shown {
        let res = game.lua_env.resources.get_holder_by_id(id);
        egui::Window::new(format!(
            "Resource debug - {}",
            res.get_path().to_string_lossy()
        ))
        .resizable(true)
        .show(ctx, |ui| {
            res.draw_debug_gui(ui);
        });
    }
}

fn draw_resource_table(editor: &mut EditorState, ui: &mut egui::Ui, game: &mut Game) {
    let available_height = ui.available_height();
    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto()) // id
        .column(Column::auto()) // path
        .column(Column::auto()) // type
        .column(
            Column::remainder()
                .at_least(60.0)
                .clip(true)
                .resizable(true),
        ) // status
        .column(Column::auto())
        .min_scrolled_height(0.0)
        .max_scroll_height(available_height);

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("ID");
            });
            header.col(|ui| {
                ui.label("Path");
            });
            header.col(|ui| {
                ui.label("Type");
            });
            header.col(|ui| {
                ui.label("Status");
            });
            header.col(|ui| {
                ui.label("Actions");
            });
        })
        .body(|mut body| {
            for (id, res) in game.lua_env.resources.enumerate() {
                let resources = game.lua_env.resources.clone();
                let status_string = res.get_status().to_string();
                let status_length = status_string.len();
                let row_height = f32::max(20.0, status_length as f32 / 2.0);

                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(id.to_string());
                    });
                    row.col(|ui| {
                        if ui
                            .link(res.get_path().to_string_lossy().to_string())
                            .clicked()
                        {
                            // Open the file
                            let absolute_path = get_absolute_path(res.get_path());
                            open::that(absolute_path).ok();
                        }
                    });
                    row.col(|ui| {
                        ui.label(res.get_type_name().to_string());
                    });
                    row.col(|ui| {
                        // wrapping
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.label(status_string);
                    });
                    row.col(|ui| {
                        if ui.button("Reload").clicked() {
                            let gl: Arc<glow::Context> = editor.gl.clone();
                            resources.reload(
                                id,
                                gl,
                                game.lua_env.lua.clone(),
                                game.lua_env.default_events.resource_loaded_event,
                            );
                        }
                        let mut config = editor.config.borrow_mut();
                        let shown = config.debug_resource_shown == Some(id);
                        let text = if shown { "Hide" } else { "Show" };
                        ui.button(text).clicked().then(|| {
                            if shown {
                                config.debug_resource_shown = None;
                            } else {
                                config.debug_resource_shown = Some(id);
                            }
                        });
                    });
                });
            }
        });
}
