use std::{borrow::Cow, fs};

use egui_extras::{Column, TableBody, TableBuilder};
use runtime::egui;

use crate::{
    editorinterface::{
        EditorState,
        extra::geteditorpaths::{get_editor_plugins_path, get_end_of_path},
    },
    pluginsystem::trustedplugin::{self, PluginEntry, TrustedPlugin},
};

pub fn draw_editor_plugin_manager(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow_mut().is_plugins_window_shown;

    if editor.config.borrow().is_plugins_window_shown {
        let window = egui::Window::new("Plugin manager")
            .resizable(true)
            .default_height(300.0)
            .default_width(500.0)
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
        if ui.button("Open plugins folder")
            .on_hover_text("Open the folder where plugins are stored. You can add plugins there and they will appear in the list of trusted plugins.")
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
        ui.heading("Trusted plugins").on_hover_text("Trusted plugins are the list of plugins known to the editor. Only trusted plugins of a game are loaded and executed. To be trusted, a plugin needs to be inside the plugins folder of the editor");

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
                        draw_trusted_plugin_row(&mut body, trusted_plugin);
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
                        });
                    }
                }
            }
        });
    }

    ui.heading("Game plugins")
        .on_hover_text("Game plugins are the list of plugins belonging to the current game. Only trusted plugins are actually loaded and executed.");

    let project = editor.project.borrow();
    if let Some(_project) = &project.as_ref() {
        if ui
            .button("Open game plugin folder")
            .on_hover_text("Open the folder with the plugins specific to your project")
            .clicked()
        {
            // ...
        }
    } else {
        ui.label("No project loaded")
            .on_hover_text("Load a project to see its plugins.");
    }
}

fn draw_trusted_plugin_row(body: &mut TableBody, plugin: &mut TrustedPlugin) {
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
            if ui.button("Add to game").clicked() {
                // ...
            }
        });
    });
}
