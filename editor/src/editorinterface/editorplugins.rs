use std::{borrow::Cow, fs};

use egui_extras::{Column, TableBody, TableBuilder};
use runtime::egui::{self, Label};

use crate::{
    editorinterface::{
        EditorState,
        extra::geteditorpaths::{get_editor_plugins_path, get_end_of_path},
    },
    pluginsystem::trustedplugin::{self, PluginEntry, TrustedPlugin},
    projectstate::ProjectState,
};

pub fn draw_editor_plugin_manager(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow_mut().is_plugins_window_shown;

    if editor.config.borrow().is_plugins_window_shown {
        let window = egui::Window::new("Plugin manager")
            .resizable(true)
            .default_height(300.0)
            .default_width(700.0)
            .open(&mut is_shown)
            .collapsible(false)
            .vscroll(false);
        let response = window.show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([true; 2])
                .show(ui, |ui| {
                    draw_editor_plugin_manager_content(editor, ui);
                });
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
        if ui.button("Open trusted plugins folder")
            .on_hover_text("Open the folder where trusted plugins are stored. You can add plugins there and they will appear in the list of trusted plugins.")
            .clicked(){
                let plugin_library_path = get_editor_plugins_path();
                if !plugin_library_path.exists() {
                    let _ = fs::create_dir_all(&plugin_library_path);
                }
                let _ = open::that(plugin_library_path);
            }

        if ui.button("Refresh trusted plugin list").clicked() {
            editor.plugins = trustedplugin::load_plugins();
        }
    });

    let plugins = &mut editor.plugins;

    if plugins.is_empty() {
        ui.label("No plugins found").on_hover_text("Plugins are programs that extend Vectarine's functionality. They are files ending with '.vecta.plugin'. You can download plugins or create them using the template provided by Vectarine GitHub repository.");
    } else {
        ui.heading("Trusted plugins").on_hover_text("Trusted plugins are the list of plugins known to the editor. Only plugins of a game that are also inside the trusted list are executed.");

        draw_table_header_for_plugin(ui, "trusted", |body| {
            for plugin in plugins.iter_mut() {
                let row_height = 20.0;
                match plugin {
                    PluginEntry::Trusted(trusted_plugin) => {
                        let game_project = editor.project.borrow();
                        draw_trusted_plugin_row(body, trusted_plugin, game_project.as_ref());
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
                                let path_shown = &malformed
                                    .path
                                    .file_name()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_else(|| get_end_of_path(&malformed.path));
                                ui.label(path_shown);
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
                            row.col(|ui| {
                                ui.label("N/A");
                            });
                        });
                    }
                }
            }
        });
    }

    ui.heading("Game plugins")
        .on_hover_text("Game plugins are the list of plugins belonging to the current game. Only plugins that are also trusted are executed.");

    let project = editor.project.borrow();
    if let Some(project) = &project.as_ref() {
        ui.horizontal(|ui| {
            #[allow(clippy::collapsible_if)]
            if ui
                .button("Open game plugin folder")
                .on_hover_text("Open the folder with the plugins specific to your project")
                .clicked()
            {
                if let Some(folder) = project.project_plugins_folder() {
                    if !folder.exists() {
                        let _ = fs::create_dir_all(&folder);
                    }
                    let _ = open::that(&folder);
                }
            }

            if ui.button("Refresh game plugins list").clicked() {
                let trusted_plugins = editor
                    .plugins
                    .iter()
                    .filter_map(|entry| match entry {
                        PluginEntry::Trusted(trusted_plugin) => Some(trusted_plugin.clone()),
                        PluginEntry::Malformed(_) => None,
                    })
                    .collect::<Vec<TrustedPlugin>>();
                project.refresh_plugin_list(&trusted_plugins);
            }
        });

        let mut game_plugins = project.plugins.borrow_mut();

        draw_table_header_for_game_plugin(ui, "game", |body| {
            for plugin in game_plugins.iter_mut() {
                let row_height = 20.0;
                let filename = plugin
                    .path
                    .file_name()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| get_end_of_path(&plugin.path));

                match plugin.trusted_plugin.as_mut() {
                    Some(trusted_plugin) => {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label(&trusted_plugin.name);
                            });
                            row.col(|ui| {
                                ui.label(filename);
                            });
                            row.col(|ui| {
                                ui.label("This plugin is trusted");
                            });
                        });
                    }
                    None => {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label("⚠️ Untrusted").on_hover_text("This plugin is not trusted and won't be executed. You can add it to the list of trusted plugin to allow its execution.");
                            });
                            row.col(|ui| {
                                ui.label(filename);
                            });
                            row.col(|ui| {
                                if ui.button("Trust").clicked() {
                                    // Trust it and refresh lists.
                                }
                            });
                        });
                    }
                }
            }
        });
    } else {
        ui.label("No project loaded")
            .on_hover_text("Load a project to see its plugins.");
    }
}

fn draw_trusted_plugin_row(
    body: &mut TableBody,
    plugin: &mut TrustedPlugin,
    game_project: Option<&ProjectState>,
) {
    let ui = body.ui_mut();
    let font_id = egui::TextStyle::Body.resolve(ui.style());

    let description_width = body.widths()[2];

    let ui = body.ui_mut();
    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple(
            plugin.description.clone(),
            font_id,
            ui.visuals().text_color(),
            description_width,
        ))
    });

    let row_height = galley.size().y + 8.0;

    body.row(row_height, |mut row| {
        row.col(|ui| {
            if ui.link(&plugin.name).on_hover_text(&plugin.url).clicked() {
                // For safety reasons, we're not opening a file
                if plugin.url.starts_with("http") {
                    let _ = open::that(&plugin.url);
                }
            }
        });
        row.col(|ui| {
            let path_shown = &plugin
                .path
                .file_name()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| get_end_of_path(&plugin.path));
            ui.label(path_shown);
        });
        row.col(|ui| {
            let label = egui::Label::new(&plugin.description).wrap();
            ui.add(label);
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
            let label = Label::new(plugin.hash.to_string()).wrap();
            if ui.add(label).on_hover_text("Click to copy").clicked() {
                ui.ctx().copy_text(plugin.hash.to_string());
            }
        });
        row.col(|ui| {
            if let Some(game_project) = game_project {
                // First, check if the plugin is already added
                let game_plugins = game_project.plugins.borrow();
                let is_added = game_plugins.iter().any(|p| {
                    p.trusted_plugin.as_ref().map(|plugin| plugin.hash) == Some(plugin.hash)
                });
                if is_added {
                    ui.label("Added");
                } else if ui.button("Add to game").clicked() {
                    game_project.add_plugin(plugin.clone());
                }
            }
        });
    });
}

/// Draws a table header for a plugin list.
/// This table has 6 columns:
/// - Name
/// - Filename
/// - About
/// - Supported platforms
/// - Hash
/// - Actions
fn draw_table_header_for_plugin(
    ui: &mut egui::Ui,
    salt: &str,
    body_renderer: impl FnOnce(&mut TableBody),
) {
    ui.push_id(salt, |ui| {
        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .auto_shrink(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto()) // Name
            .column(Column::auto().at_most(200.0).clip(true)) // Path
            .column(Column::auto().resizable(true)) // About (description, version, url, errors, supported platforms, ...)
            .column(Column::auto()) // Supported platforms
            .column(Column::auto()) // Hash
            .column(Column::auto()) // Actions
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        let table = table.header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("Name");
                });
                header.col(|ui| {
                    ui.label("Filename");
                });
                header.col(|ui| {
                    ui.label("Description");
                });
                header.col(|ui| {
                    ui.label("Supported platforms").on_hover_text("Your game will only be available on the platforms that are supported by all of the plugins your game uses.");
                });
                header.col(|ui| {
                    ui.label("Hash").on_hover_text("You can compare this hash with the one on the plugin's website if it exists to make sure the plugin is not corrupted or malicious.");
                });
                header.col(|ui| {
                    ui.label("Actions");
                });
            });
        table.body(|mut body| {
            body_renderer(&mut body);
        });
    });
}

// Draw a table header for game plugins
// This table has 3 columns
// Name, Filename and Actions
fn draw_table_header_for_game_plugin(
    ui: &mut egui::Ui,
    salt: &str,
    body_renderer: impl FnOnce(&mut TableBody),
) {
    ui.push_id(salt, |ui| {
        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .auto_shrink(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto()) // Name
            .column(Column::auto().at_most(200.0).clip(true)) // Path
            .column(Column::auto()) // Actions
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);
        let table = table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("Trusted Name");
            });
            header.col(|ui| {
                ui.label("Filename");
            });
            header.col(|ui| {
                ui.label("Actions");
            });
        });
        table.body(|mut body| {
            body_renderer(&mut body);
        });
    });
}
