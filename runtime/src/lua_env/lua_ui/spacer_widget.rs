use std::cell::RefCell;

use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::mlua;

use super::{EventState, VectarineWidget};

#[derive(Debug, Clone)]
pub struct SpacerWidget {
    pub size: Vec2,
    pub event_state: EventState,
}

impl VectarineWidget for SpacerWidget {
    fn size(&self, _lua: &mlua::Lua) -> Vec2 {
        self.size
    }
    fn draw(
        &mut self,
        _lua: &mlua::Lua,
        _batch: &RefCell<batchdraw::BatchDraw2d>,
        _io_env: &RefCell<IoEnvState>,
        _current_state: EventState,
        _process_child_events: bool,
        _draw_debug_outline: bool,
        _extra: mlua::Value,
    ) -> mlua::Result<()> {
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(self.clone())
    }
    fn event_state_mut(&mut self) -> &mut EventState {
        &mut self.event_state
    }
    fn event_state(&self) -> &EventState {
        &self.event_state
    }

    fn debug_label(&self) -> String {
        "SpacerWidget".to_string()
    }
}
