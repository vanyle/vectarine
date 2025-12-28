use std::{cell::RefCell, path::Path, rc::Rc};

use crate::{
    game_resource::{Resource, ResourceId, Status},
    lua_env::run_file_and_display_error_from_lua_handle,
};

pub struct ScriptResource {
    pub script: RefCell<Option<Vec<u8>>>,
    /// If provided when the script is created, the return table of the script will be merged into this table.
    pub target_table: Option<mlua::Table>,
}

impl Resource for ScriptResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &super::DependencyReporter,
        lua: &Rc<mlua::Lua>,
        _gl: std::sync::Arc<glow::Context>,
        path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        run_file_and_display_error_from_lua_handle(lua, &data, path, self.target_table.as_ref());
        self.script.replace(Some(data.to_vec()));
        Status::Loaded
    }

    fn draw_debug_gui(&self, _painter: &mut egui_glow::Painter, ui: &mut egui::Ui) {
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
            target_table: None,
        }
    }
}

impl ScriptResource {
    pub fn make_with_target_table(target_table: mlua::Table) -> Self {
        Self {
            script: RefCell::new(None),
            target_table: Some(target_table),
        }
    }

    pub fn get_exports(&self) -> Option<&mlua::Table> {
        self.target_table.as_ref()
    }
}
