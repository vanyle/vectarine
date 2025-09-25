use runtime::{game::Game, lua_env::stringify_lua_value, mlua};

use crate::editorinterface::EditorState;

pub fn draw_editor_watcher(editor: &mut EditorState, game: &mut Game, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_watcher_window_shown;

    egui::Window::new("Watcher")
        .default_width(400.0)
        .default_height(200.0)
        .open(&mut is_shown)
        .show(ctx, |ui| {
            let globals = game.lua_env.lua.globals();

            // Concept: In the code, you can "watch" a variable with watch(variable)
            // Then it will appear in this list.
            // You can modify the variable with sliders for numbers, color pickers for colors, etc.

            let snake = globals.raw_get::<mlua::Value>("Snake");
            let Ok(snake) = snake else {
                return;
            };

            ui.label(format!("Snake = {}", stringify_lua_value(&snake)));
        });
    editor.config.borrow_mut().is_watcher_window_shown = is_shown;
}
