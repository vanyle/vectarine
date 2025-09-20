use std::{cell::RefCell, path::Path, rc::Rc};

use crate::{
    game_resource::{Resource, ResourceId, Status},
    lua_env::run_file_and_display_error_from_lua_handle,
};

pub struct ScriptResource {
    pub script: RefCell<Option<Vec<u8>>>,
}

impl Resource for ScriptResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &super::DependencyReporter,
        lua: Rc<mlua::Lua>,
        _gl: std::sync::Arc<glow::Context>,
        path: &Path,
        data: &[u8],
    ) -> Status {
        run_file_and_display_error_from_lua_handle(&lua, data, path);
        self.script.replace(Some(data.to_vec()));
        Status::Loaded
    }

    fn draw_debug_gui(&self, ui: &mut egui::Ui) {
        // If we wanted a script editor, it would be here.
        ui.label("[TODO] Script Resource debug gui");
    }

    fn get_type_name(&self) -> &'static str {
        "Script"
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            script: RefCell::new(None),
        }
    }
}
