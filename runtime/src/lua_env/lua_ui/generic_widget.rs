use std::cell::RefCell;

use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::mlua;

use super::{EventState, VectarineWidget};

#[derive(Debug, Clone)]
pub struct GenericWidget {
    pub size: Vec2,
    pub draw_fn: mlua::Function,
    pub event_state: EventState,
}

impl VectarineWidget for GenericWidget {
    fn size(&self) -> Vec2 {
        self.size
    }
    fn draw(
        &mut self,
        lua: &mlua::Lua,
        _batch: &RefCell<batchdraw::BatchDraw2d>,
        _io_env: &RefCell<IoEnvState>,
        current_state: EventState,
        _process_child_events: bool,
        _draw_debug_outline: bool,
        extra: mlua::Value,
    ) -> mlua::Result<()> {
        self.draw_fn.call::<()>((
            current_state
                .to_lua(lua)
                .expect("Convertion to table should never fail"),
            extra,
        ))
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
        "Widget".to_string()
    }
}
