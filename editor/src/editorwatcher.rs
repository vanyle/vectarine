use std::{cell::RefCell, thread::LocalKey};

use egui::RichText;
use egui_extras::{Size, StripBuilder};
use runtime::{
    game::Game,
    lua_env::{lua_vec2::Vec2, stringify_lua_value},
    mlua,
};

use crate::editorinterface::EditorState;

const MAX_WATCHED_VARIABLES: usize = 20;

pub fn draw_editor_watcher(editor: &mut EditorState, game: &mut Game, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_watcher_window_shown;

    egui::Window::new("Watcher")
        .default_width(400.0)
        .default_height(200.0)
        .open(&mut is_shown)
        .show(ctx, |ui| {
            draw_editor_watcher_window(ui, game);
        });
    editor.config.borrow_mut().is_watcher_window_shown = is_shown;
}

fn draw_editor_watcher_window(ui: &mut egui::Ui, game: &mut Game) {
    let globals = game.lua_env.lua.globals();

    thread_local! {
        static SEARCH_BOX_CONTENT: RefCell<String> = const { RefCell::new(String::new()) };
        static WATCHED_VARIABLES_NAMES: RefCell<Vec<mlua::Value>> = const { RefCell::new(Vec::new()) };
    }

    let watched_vars_len = WATCHED_VARIABLES_NAMES.with_borrow(|vars| vars.len());
    if watched_vars_len < MAX_WATCHED_VARIABLES {
        SEARCH_BOX_CONTENT.with_borrow_mut(|content| {
            draw_search_variable_box(ui, content, &globals, &WATCHED_VARIABLES_NAMES);
        });
    } else {
        ui.label(
            RichText::new(format!(
                "{watched_vars_len}/{MAX_WATCHED_VARIABLES} variables watched"
            ))
            .strong(),
        );
    }

    StripBuilder::new(ui)
        .size(Size::remainder().at_least(100.0))
        .vertical(|mut strip| {
            strip.cell(|ui| {
                ui.label(RichText::new("Watched Variables").heading());
                egui::ScrollArea::vertical()
                    .id_salt("watched variables")
                    .max_height(800.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        WATCHED_VARIABLES_NAMES.with_borrow_mut(|vars| {
                            for idx in 0..vars.len() {
                                draw_watched_variable(ui, &globals, vars, idx);
                            }
                        });
                    });
            });
        });
}

fn draw_search_variable_box(
    ui: &mut egui::Ui,
    content: &mut String,
    globals: &mlua::Table,
    watched_variable_names: &'static LocalKey<RefCell<Vec<mlua::Value>>>,
) {
    let search_results = globals
        .pairs::<mlua::Value, mlua::Value>()
        .flatten()
        .flat_map(|(key, _)| {
            let key_str = stringify_lua_value(&key);
            if key_str.to_lowercase().contains(&content.to_lowercase()) {
                Some(key)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let response = egui::TextEdit::singleline(content)
        .hint_text("Search for a global to watch")
        .show(ui)
        .response;

    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        // Clear search box on enter
        if search_results.len() == 1 {
            content.clear();
            watched_variable_names.with_borrow_mut(|vars| {
                let key = search_results[0].clone();
                if !vars.iter().any(|v| v == &key) {
                    vars.push(key);
                }
            });
        }
        response.request_focus(); // keep focus on enter
    }

    if !content.is_empty() {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for result in search_results.iter() {
                    ui.horizontal(|ui| {
                        let key_str = stringify_lua_value(result);
                        ui.label(format!("Watch {}", key_str));
                        if ui.button("+").on_hover_text("Add to watch list").clicked() {
                            watched_variable_names.with_borrow_mut(|vars| {
                                if !vars.iter().any(|v| v == result) {
                                    vars.push(result.clone());
                                }
                            });
                        }
                    });
                }
            });
    }
}

fn draw_watched_variable(
    ui: &mut egui::Ui,
    globals: &mlua::Table,
    var_keys: &mut Vec<mlua::Value>,
    idx: usize,
) {
    let var = var_keys.get(idx).cloned();
    let Some(var) = var else {
        return; // removed by another watcher
    };
    let watched_value = globals.raw_get::<mlua::Value>(&var);
    let Ok(watched_value) = watched_value else {
        return;
    };
    let var_name = stringify_lua_value(&var);
    let var_type = watched_value.type_name();

    egui::CollapsingHeader::new(format!("{} - {}", var_name, var_type)).show(ui, |ui| {
        ui.button("Remove")
            .on_hover_text("Remove from watch list")
            .clicked()
            .then(|| {
                var_keys.remove(idx);
            });
        draw_any_watcher(ui, globals, &var, &watched_value);
    });
}

fn draw_any_watcher(
    ui: &mut egui::Ui,
    variable_parent: &mlua::Table,
    value_global_name: &mlua::Value,
    watched_value: &mlua::Value,
) {
    if let mlua::Value::Table(table) = watched_value {
        draw_table_watcher(ui, table);
    } else if let mlua::Value::Boolean(b) = watched_value {
        draw_boolean_watcher(ui, *b, |new_val| {
            let _ = variable_parent.raw_set(value_global_name, new_val);
        });
    } else if let mlua::Value::Integer(n) = watched_value {
        draw_number_watcher(ui, *n as f64, |new_val| {
            let _ = variable_parent.raw_set(value_global_name, new_val);
        });
    } else if let mlua::Value::Number(n) = watched_value {
        draw_number_watcher(ui, *n, |new_val| {
            let _ = variable_parent.raw_set(value_global_name, new_val);
        });
    } else if let mlua::Value::UserData(ud) = watched_value {
        let maybe_vec = ud.borrow_mut::<Vec2>();
        if let Ok(mut vec) = maybe_vec {
            draw_vec2_watcher(ui, &mut vec);
        }
    } else {
        ui.label(format!(
            "Non editable value: {}",
            stringify_lua_value(watched_value)
        ));
    }
}

fn draw_table_watcher(ui: &mut egui::Ui, table: &mlua::Table) {
    let pairs = table.pairs::<mlua::Value, mlua::Value>();
    for pair in pairs.flatten() {
        let (key, value) = pair;
        ui.horizontal(|ui| {
            ui.label(format!("{}:", stringify_lua_value(&key)));
            if let mlua::Value::Table(_) = value {
                ui.label("Table");
            } else {
                draw_any_watcher(ui, table, &key, &value);
            }
        });
    }
}

fn draw_boolean_watcher<F>(ui: &mut egui::Ui, value: bool, set_value: F)
where
    F: Fn(bool),
{
    ui.horizontal(|ui| {
        let mut val = value;
        if ui.checkbox(&mut val, "Value").changed() {
            set_value(val);
        }
    });
}

fn draw_number_watcher<F>(ui: &mut egui::Ui, value: f64, set_value: F)
where
    F: Fn(f64),
{
    ui.horizontal(|ui| {
        let mut val = value;
        if ui.add(egui::DragValue::new(&mut val).speed(0.1)).changed() {
            set_value(val);
        }
    });
}

fn draw_vec2_watcher(ui: &mut egui::Ui, vec: &mut Vec2) {
    ui.horizontal(|ui| {
        let mut x = vec.x;
        let mut y = vec.y;
        if ui
            .add(egui::DragValue::new(&mut x).prefix("x: ").speed(0.1))
            .changed()
        {
            vec.x = x;
        }
        if ui
            .add(egui::DragValue::new(&mut y).prefix("y: ").speed(0.1))
            .changed()
        {
            vec.y = y;
        }
    });
}
