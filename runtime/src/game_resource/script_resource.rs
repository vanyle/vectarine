use std::{cell::RefCell, path::Path, rc::Rc};

use crate::{
    game_resource::{Resource, ResourceId, ResourceManager, Status},
    lua_env::{
        LuaEnvironment, run_file_and_display_error, run_file_and_display_error_from_lua_handle,
    },
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
    script_resource_id: ResourceId,
) {
    let holder = resource_manager.get_holder_by_id(script_resource_id);
    let Ok(script_resource) = holder.get_underlying_resource::<ScriptResource>() else {
        return; // resource type mismatch
    };
    script_resource.run_script(lua, holder.get_path());
}
