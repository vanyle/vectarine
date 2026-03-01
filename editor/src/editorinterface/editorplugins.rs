use std::{borrow::Cow, fs};

use egui_extras::{Column, TableBuilder};
use runtime::egui;

use crate::{
    editorinterface::{EditorState, extra::geteditorpaths::get_editor_plugins_path},
    pluginsystem::trustedplugin::{self, PluginEntry, TrustedPlugin},
};

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

fn draw_editor_plugin_manager_content(editor: &mut EditorState, ui: &mut egui::Ui) {
    ui.horizontal(|ui|{
        if ui.button("Open plugin folder")
            .on_hover_text("Open the folder where plugins are stored. You can add plugins there and they will appear in the list of available plugins.")
            .clicked(){
                let plugin_library_path = get_editor_plugins_path();
                if !plugin_library_path.exists() {
                    let _ = fs::create_dir_all(&plugin_library_path);
                }
                let _ = open::that(plugin_library_path);
            }

        if ui.button("Refresh plugin list").clicked() {
            editor.plugins = trustedplugin::load_plugins();
        }
    });

    let plugins = &mut editor.plugins;

    if plugins.is_empty() {
        ui.label("No plugins found").on_hover_text("Plugins are programs that extend Vectarine's functionality. They are files ending with '.vecta.plugin'. You can download plugins or create them using the template provided by Vectarine GitHub repository.");
    } else {
        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .auto_shrink(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto()) // Name
            .column(Column::auto().at_most(100.0).clip(true)) // Path
            .column(Column::auto()) // About (description, version, url, errors, supported platforms, ...)
            .column(Column::auto()) // Supported platforms
            .column(Column::auto()) // Actions
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);
        let table = table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("Name");
            });
            header.col(|ui| {
                ui.label("Path");
            });
            header.col(|ui| {
                ui.label("Description");
            });
            header.col(|ui| {
                ui.label("Supported platforms");
            });
            header.col(|ui| {
                ui.label("Actions");
            });
        });
        table.body(|mut body| {
            for plugin in plugins.iter_mut() {
                let row_height = 20.0;
                match plugin {
                    PluginEntry::Trusted(trusted_plugin) => {
                        body.row(row_height, |mut row| {
                            draw_trusted_plugin_row(&mut row, trusted_plugin);
                        });
                    }
                    PluginEntry::Malformed(malformed) => {
                        let filename = malformed
                            .path
                            .file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_else(|| Cow::from("???"));
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label(filename);
                            });
                            row.col(|ui| {
                                ui.label(malformed.path.to_string_lossy());
                            });
                            row.col(|ui| {
                                ui.label(&malformed.error);
                            });
                            row.col(|ui| {
                                ui.label("N/A");
                            });
                            row.col(|ui| {
                                ui.label("N/A");
                            });
                        });
                    }
                }
            }
        });
    }
}

fn draw_trusted_plugin_row(row: &mut egui_extras::TableRow, plugin: &mut TrustedPlugin) {
    row.col(|ui| {
        ui.label(&plugin.name);
    });
    row.col(|ui| {
        ui.label(plugin.path.to_string_lossy());
    });
    row.col(|ui| {
        ui.label(&plugin.description);
    });
    row.col(|ui| {
        let supported_platforms = plugin
            .supported_platforms
            .iter()
            .map(|platform| format!("{}", platform))
            .collect::<Vec<_>>()
            .join(", ");
        ui.label(supported_platforms);
    });
    row.col(|ui| {
        ui.label("Actions");
    });
}
