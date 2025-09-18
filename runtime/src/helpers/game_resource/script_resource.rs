use std::{cell::RefCell, path::Path};

use crate::helpers::{
    game_resource::{Resource, ResourceManager, Status},
    lua_env::{LuaEnvironment, run_file_and_display_error},
};

pub struct ScriptResource {
    pub script: RefCell<Option<Vec<u8>>>,
}

impl Resource for ScriptResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: usize,
        _dependency_reporter: &super::DependencyReporter,
        // TODO: pass the path and lua_env here.
        _gl: std::sync::Arc<glow::Context>,
        data: &[u8],
    ) -> Status {
        // There is state duplication :'(
        // The option stores the same thing as Status.

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

impl ScriptResource {
    pub fn run_script(&self, lua: &LuaEnvironment, script_path: &Path) {
        let script = self.script.borrow();
        let Some(script_data) = script.as_ref() else {
            return; // resource not loaded
        };
        run_file_and_display_error(lua, script_data, script_path);
    }
}

pub fn run_script_resource(
    lua: &LuaEnvironment,
    resource_manager: &ResourceManager,
    script_resource_id: usize,
) {
    let holder = resource_manager.get_holder_by_id(script_resource_id);
    let Some(holder) = holder else {
        return; // resource not found
    };
    let Ok(script_resource) = holder.get_underlying_resource::<ScriptResource>() else {
        return; // resource type mismatch
    };
    script_resource.run_script(lua, holder.get_path());
}
