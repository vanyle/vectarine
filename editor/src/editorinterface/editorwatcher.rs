use std::{cell::RefCell, thread::LocalKey};

use egui_extras::{Size, StripBuilder};
use runtime::egui;
use runtime::egui::RichText;
use runtime::{
    lua_env::lua_physics::Object2,
    lua_env::{lua_vec2::Vec2, lua_vec4::Vec4, stringify_lua_value},
    mlua,
};

use crate::editorinterface::EditorState;

const MAX_WATCHED_VARIABLES: usize = 20;
const MAX_TABLE_INSPECTION_DEPTH: usize = 2;

pub fn draw_editor_watcher(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_watcher_window_shown;

    let maybe_response = egui::Window::new("Watcher")
        .default_width(400.0)
        .default_height(200.0)
        .open(&mut is_shown)
        .collapsible(false)
        .show(ctx, |ui| {
            draw_editor_watcher_window(ui, editor);
        });
    if let Some(response) = maybe_response {
        let on_top = Some(response.response.layer_id) == ctx.top_layer_id();
        if on_top && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            is_shown = false;
        }
    }
    editor.config.borrow_mut().is_watcher_window_shown = is_shown;
}

fn draw_editor_watcher_window(ui: &mut egui::Ui, editor: &mut EditorState) {
    let mut project = editor.project.borrow_mut();
    let game = match project.as_mut() {
        Some(proj) => Some(&mut proj.game),
        None => None,
    };

    let Some(game) = game else {
        ui.label("No project loaded");
        return;
    };

    let globals = game.lua_env.lua.globals();

    thread_local! {
        static SEARCH_BOX_CONTENT: RefCell<String> = const { RefCell::new(String::new()) };
        static WATCHED_VARIABLES_NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
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
                                draw_watched_variable(ui, &game.lua_env.lua, &globals, vars, idx);
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
    watched_variable_names: &'static LocalKey<RefCell<Vec<String>>>,
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
        content.clear();
        watched_variable_names.with_borrow_mut(|vars| {
            let Some(first_key) = search_results.first() else {
                return;
            };
            let key = stringify_lua_value(first_key);
            if !vars.iter().any(|v| v == &key) {
                vars.push(key);
            }
        });
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
                                if !vars.contains(&key_str) {
                                    vars.push(key_str.clone());
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
    lua: &mlua::Lua,
    globals: &mlua::Table,
    var_keys: &mut Vec<String>,
    idx: usize,
) {
    let var = var_keys.get(idx).cloned();
    let Some(var) = var else {
        return; // removed by another watcher
    };
    let lua_key = lua.create_string(var.clone());
    let Ok(lua_key) = lua_key else {
        return;
    };
    let lua_key = mlua::Value::String(lua_key);
    let watched_value = globals.raw_get::<mlua::Value>(var.clone());
    let Ok(watched_value) = watched_value else {
        return;
    };
    let var_name = &var;
    let var_type = watched_value.type_name();

    egui::CollapsingHeader::new(format!("{} - {}", var_name, var_type)).show(ui, |ui| {
        ui.button("Remove")
            .on_hover_text("Remove from watch list")
            .clicked()
            .then(|| {
                var_keys.remove(idx);
            });
        draw_any_watcher(
            ui,
            globals,
            &lua_key,
            &watched_value,
            MAX_TABLE_INSPECTION_DEPTH,
        );
    });
}

fn draw_any_watcher(
    ui: &mut egui::Ui,
    variable_parent: &mlua::Table,
    value_global_name: &mlua::Value,
    watched_value: &mlua::Value,
    max_depth: usize,
) {
    if let mlua::Value::Table(table) = watched_value {
        draw_table_watcher(ui, table, max_depth);
        return;
    }
    if let mlua::Value::Boolean(b) = watched_value {
        draw_boolean_watcher(ui, *b, |new_val| {
            let _ = variable_parent.raw_set(value_global_name, new_val);
        });
        return;
    }
    if let mlua::Value::Integer(n) = watched_value {
        draw_number_watcher(ui, *n as f64, |new_val| {
            let _ = variable_parent.raw_set(value_global_name, new_val);
        });
        return;
    }
    if let mlua::Value::Number(n) = watched_value {
        draw_number_watcher(ui, *n, |new_val| {
            let _ = variable_parent.raw_set(value_global_name, new_val);
        });
        return;
    }
    // Note: based on the variable name (value_global_name), we can draw a better picker.
    // For example, if there is color in the name, we can draw a color picker.

    if let mlua::Value::UserData(ud) = watched_value {
        let maybe_vec = ud.borrow_mut::<Vec2>();
        if let Ok(mut vec) = maybe_vec {
            draw_vec2_watcher(ui, &mut vec);
            return;
        }
        let maybe_vec = ud.borrow_mut::<Vec4>();
        if let Ok(mut vec) = maybe_vec {
            let var_name = stringify_lua_value(value_global_name);
            draw_vec4_watcher(ui, &mut vec, var_name.contains("color"));
            return;
        }
        let maybe_object = ud.borrow_mut::<Object2>();
        if let Ok(mut object) = maybe_object {
            draw_object_watcher(ui, &mut object);
            return;
        }
    }

    ui.label(format!(
        "Non editable value: {}",
        stringify_lua_value(watched_value)
    ));
}

fn draw_table_watcher(ui: &mut egui::Ui, table: &mlua::Table, max_depth: usize) {
    let pairs = table.pairs::<mlua::Value, mlua::Value>();
    for pair in pairs.flatten() {
        let (key, value) = pair;
        ui.horizontal(|ui| {
            if max_depth == 0 {
                ui.label(format!("{}:", stringify_lua_value(&key)));
                ui.label("...");
            } else if let mlua::Value::Table(_) = value {
                egui::CollapsingHeader::new(format!("{}:", stringify_lua_value(&key))).show(
                    ui,
                    |ui| {
                        draw_any_watcher(ui, table, &key, &value, max_depth - 1);
                    },
                );
            } else {
                ui.label(format!("{}:", stringify_lua_value(&key)));
                draw_any_watcher(ui, table, &key, &value, max_depth - 1);
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
        let mut x = vec.0[0];
        let mut y = vec.0[1];
        if ui
            .add(egui::DragValue::new(&mut x).prefix("x: ").speed(0.1))
            .changed()
        {
            vec.0[0] = x;
        }
        if ui
            .add(egui::DragValue::new(&mut y).prefix("y: ").speed(0.1))
            .changed()
        {
            vec.0[1] = y;
        }
    });
}

fn draw_vec4_watcher(ui: &mut egui::Ui, vec: &mut Vec4, is_color: bool) {
    ui.horizontal(|ui| {
        let mut x = vec.0[0];
        let mut y = vec.0[1];
        let mut z = vec.0[2];
        let mut w = vec.0[3];
        if is_color {
            ui.color_edit_button_rgba_unmultiplied(&mut vec.0);
        } else {
            if ui
                .add(egui::DragValue::new(&mut x).prefix("x: ").speed(0.1))
                .changed()
            {
                vec.0[0] = x;
            }
            if ui
                .add(egui::DragValue::new(&mut y).prefix("y: ").speed(0.1))
                .changed()
            {
                vec.0[1] = y;
            }
            if ui
                .add(egui::DragValue::new(&mut z).prefix("z: ").speed(0.1))
                .changed()
            {
                vec.0[2] = z;
            }
            if ui
                .add(egui::DragValue::new(&mut w).prefix("w: ").speed(0.1))
                .changed()
            {
                vec.0[3] = w;
            }
        }
    });
}

fn draw_object_watcher(ui: &mut egui::Ui, object: &mut Object2) {
    if object.is_out_of_world() {
        ui.label("Object is out of world");
    } else if let Some(position) = object.position()
        && let Some(velocity) = object.velocity()
    {
        let mut position = position;
        let mut velocity = velocity;
        ui.horizontal(|ui| {
            ui.label("Position");
            draw_vec2_watcher(ui, &mut position);
        });
        ui.horizontal(|ui| {
            ui.label("Velocity");
            draw_vec2_watcher(ui, &mut velocity);
        });
        // TODO: Rotation, tags, and extras could also be shown here.
        object.set_position(position);
        object.set_velocity(velocity);
    }
}
