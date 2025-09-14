use std::fs;

use notify_debouncer_full::DebouncedEvent;
use runtime::helpers::lua_env::{LuaEnvironment, run_file_and_display_error};

// Reload assets corresponding to changed file as needed without blocking
pub fn reload_assets_if_needed(
    lua_for_reload: &LuaEnvironment,
    debounce_receiver: &std::sync::mpsc::Receiver<DebouncedEvent>,
) {
    for event in debounce_receiver.try_iter() {
        for path in event.event.paths {
            // Check if a resource is in the list of path
            // If so, and the resource is in an unloaded / loaded state, load it.

            if path.extension().is_some() && path.extension().unwrap() == "lua" {
                // println!("Reloading script: {}", path.to_string_lossy());
                let content = fs::read(&path);
                let Ok(content) = content else {
                    println!("Failed to read file: {}", path.to_string_lossy());
                    continue;
                };
                run_file_and_display_error(lua_for_reload, &content, &path);
            }
        }
    }
}
