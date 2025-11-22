use std::{rc::Rc, sync::Arc};

use notify_debouncer_full::{
    DebouncedEvent,
    notify::{EventKind, event::ModifyKind},
};
use runtime::{
    game_resource::{ResourceManager, Status},
    lua_env::LuaEnvironment,
};

// Reload assets corresponding to changed file as needed without blocking
pub fn reload_assets_if_needed(
    gl: &Arc<glow::Context>,
    resources: &Rc<ResourceManager>,
    lua_for_reload: &LuaEnvironment,
    debounce_receiver: &std::sync::mpsc::Receiver<DebouncedEvent>,
) {
    for event in debounce_receiver.try_iter() {
        // Only file modification matters, no creation, deletion, etc...
        let EventKind::Modify(modify) = event.kind else {
            continue;
        };
        // We only care about data modifications, not metadata.
        if !matches!(modify, ModifyKind::Data(_) | ModifyKind::Any) {
            continue;
        }

        for path in event.event.paths {
            // Check if a resource is in the list of path
            // If so, and the resource is in an unloaded / loaded state, load it.
            if let Some(res_id) = resources.get_id_by_path(&path) {
                let res = resources.get_holder_by_id_unchecked(res_id);
                let res_status = res.get_status();
                if matches!(
                    res_status,
                    Status::Unloaded | Status::Loaded | Status::Error(_)
                ) {
                    resources.reload(
                        res_id,
                        gl.clone(),
                        lua_for_reload.lua.clone(),
                        lua_for_reload.default_events.resource_loaded_event.clone(),
                    );
                }
            }
        }
    }
}
