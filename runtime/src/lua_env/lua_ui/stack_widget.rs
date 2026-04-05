use std::cell::RefCell;

use crate::graphics::affinetransform::AffineTransform;
use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::mlua;

use super::{Alignment, EventState, VectarineWidget, WidgetBox};

pub struct Stack {
    pub children: Vec<WidgetBox>,
    pub align_x: Alignment,
    pub align_y: Alignment,
    pub event_state: EventState,
}

impl VectarineWidget for Stack {
    fn size(&self) -> Vec2 {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        for child in &self.children {
            let child_size = child.0.size();
            width = width.max(child_size.x());
            height = height.max(child_size.y());
        }
        Vec2::new(width, height)
    }

    fn event_state(&self) -> &EventState {
        &self.event_state
    }

    fn event_state_mut(&mut self) -> &mut EventState {
        &mut self.event_state
    }

    fn draw(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        _current_state: EventState,
        process_child_events: bool,
        extra: mlua::Value,
    ) {
        let stack_size = self.size();
        let stack_width = stack_size.x();
        let stack_height = stack_size.y();

        for child in &mut self.children {
            let child_size = child.0.size();
            let child_width = child_size.x();
            let child_height = child_size.y();

            let x_offset = match self.align_x {
                Alignment::Start => 0.0,
                Alignment::Center => (stack_width - child_width) / 2.0,
                Alignment::End => stack_width - child_width,
            };
            let y_offset = match self.align_y {
                Alignment::Start => 0.0,
                Alignment::Center => (stack_height - child_height) / 2.0,
                Alignment::End => stack_height - child_height,
            };

            let current_transform = batch.borrow().affine_transform;
            batch.borrow_mut().affine_transform = current_transform.combine(&AffineTransform::new(
                Vec2::new(x_offset, y_offset),
                Vec2::new(1.0, 1.0),
                0.0,
            ));
            child
                .0
                .event_processing_draw(lua, batch, io_env, process_child_events, extra.clone());
            batch.borrow_mut().affine_transform = current_transform;
        }
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(Stack {
            children: self.children.to_vec(),
            align_x: self.align_x,
            align_y: self.align_y,
            event_state: self.event_state.clone(),
        })
    }
}
