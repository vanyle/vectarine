use std::{cell::RefCell, fs, rc::Rc, sync::Arc};

use notify_debouncer_full::DebouncedEvent;
use runtime::helpers::{
    game_resource::{ResourceManager, ResourceStatus},
    lua_env::{LuaEnvironment, run_file_and_display_error},
};

// Reload assets corresponding to changed file as needed without blocking
pub fn reload_assets_if_needed(
    gl: &Arc<glow::Context>,
    resources: &Rc<RefCell<ResourceManager>>,
    lua_for_reload: &LuaEnvironment,
    debounce_receiver: &std::sync::mpsc::Receiver<DebouncedEvent>,
) {
    for event in debounce_receiver.try_iter() {
        for path in event.event.paths {
            // Check if a resource is in the list of path
            // If so, and the resource is in an unloaded / loaded state, load it.
            if let Some(res) = resources.borrow().get_by_path(&path) {
                let res_status = res.get_loading_status();
                if matches!(
                    res_status,
                    ResourceStatus::Loaded | ResourceStatus::Error(_)
                ) {
                    res.reload(gl.clone());
                }
            };

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
