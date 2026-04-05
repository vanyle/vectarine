use std::cell::RefCell;

use crate::graphics::affinetransform::AffineTransform;
use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::mlua;

use super::{Alignment, EventState, Padding, VectarineWidget, WidgetBox};

pub struct Row {
    pub children: Vec<WidgetBox>,
    pub alignment: Alignment,
    pub padding: Padding,
    pub gap: f32,
    pub event_state: EventState,
}

impl VectarineWidget for Row {
    fn size(&self) -> Vec2 {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        for child in &self.children {
            let child_size: crate::math::Vect<2> = child.0.size();
            width += child_size.x();
            height = height.max(child_size.y());
        }
        let gap_total = if self.children.len() > 1 {
            self.gap * (self.children.len() - 1) as f32
        } else {
            0.0
        };
        Vec2::new(
            width + gap_total + self.padding.left + self.padding.right,
            height + self.padding.top + self.padding.bottom,
        )
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
    ) -> mlua::Result<()> {
        let container_height = self.size().y() - self.padding.top - self.padding.bottom;
        let mut x_offset = self.padding.left;
        for child in &mut self.children {
            let child_size = child.0.size();
            let child_height = child_size.y();
            let y_offset = self.padding.bottom
                + match self.alignment {
                    Alignment::Start => 0.0,
                    Alignment::Center => (container_height - child_height) / 2.0,
                    Alignment::End => container_height - child_height,
                };
            let current_transform = batch.borrow().affine_transform;
            batch.borrow_mut().affine_transform = current_transform.combine(&AffineTransform::new(
                Vec2::new(x_offset, y_offset),
                Vec2::new(1.0, 1.0),
                0.0,
            ));
            let result = child.0.event_processing_draw(
                lua,
                batch,
                io_env,
                process_child_events,
                extra.clone(),
            );
            batch.borrow_mut().affine_transform = current_transform;
            result?;
            x_offset += child_size.x() + self.gap;
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(Row {
            children: self.children.to_vec(),
            alignment: self.alignment,
            padding: self.padding,
            gap: self.gap,
            event_state: self.event_state.clone(),
        })
    }

    fn debug_label(&self) -> String {
        let children: Vec<String> = self.children.iter().map(|c| c.0.debug_label()).collect();
        format!("Row({})", children.join(", "))
    }
}
