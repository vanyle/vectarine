use std::{cell::RefCell, path::Path, rc::Rc};

use crate::{
    game_resource::{Resource, ResourceId, Status},
    lua_env::LuaHandle,
};
use vectarine_plugin_sdk::glow;

#[derive(Debug)]
pub struct ScriptResource {
    pub script: RefCell<Option<Vec<u8>>>,
    /// If provided when the script is created, the return table of the script will be merged into this table.
    pub target_table: Option<vectarine_plugin_sdk::mlua::Table>,
}

impl Resource for ScriptResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        assigned_id: ResourceId,
        dependency_reporter: &super::DependencyReporter,
        lua: &Rc<LuaHandle>,
        _gl: std::sync::Arc<glow::Context>,
        path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        self.script.replace(Some(data.to_vec()));

        let dependency_reporter = dependency_reporter.clone();
        let lua_future = lua.clone();
        let path = path.to_path_buf();
        let future = Box::pin(async move {
            let result = lua_future
                .async_run_file_and_display_error(&data, &path, self.target_table.as_ref())
                .await;
            match result {
                Ok(_) => {
                    dependency_reporter.mark_as_ready(assigned_id);
                    Ok(())
                }
                Err(e) => {
                    dependency_reporter.mark_as_error(assigned_id, e.to_string());
                    Err(e)
                }
            }
        });

        let is_ready = lua.async_handler.borrow_mut().schedule_future(lua, future);
        if is_ready {
            Status::Loaded
        } else {
            Status::Loading
        }
    }

    fn draw_debug_gui(
        &self,
        _painter: &mut vectarine_plugin_sdk::egui_glow::Painter,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    ) {
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
    pub fn make_with_target_table(target_table: vectarine_plugin_sdk::mlua::Table) -> Self {
        Self {
            script: RefCell::new(None),
            target_table: Some(target_table),
        }
    }

    pub fn get_exports(&self) -> Option<&vectarine_plugin_sdk::mlua::Table> {
        self.target_table.as_ref()
    }
}
