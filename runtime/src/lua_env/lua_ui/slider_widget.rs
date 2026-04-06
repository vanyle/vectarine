use std::cell::RefCell;

use crate::graphics::affinetransform::AffineTransform;
use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::mlua;

use super::{EventState, VectarineWidget, WidgetBox};

pub struct Slider {
    pub size: Vec2,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub step: f32,
    pub on_change: Option<mlua::Function>,
    pub track: WidgetBox,
    pub handle: WidgetBox,
    pub event_state: EventState,
    pub dragging: bool,
}

impl Slider {
    fn snap_value(&self, raw: f32) -> f32 {
        if self.step <= 0.0 {
            return raw.clamp(self.min, self.max);
        }
        let steps = ((raw - self.min) / self.step).round();
        (self.min + steps * self.step).clamp(self.min, self.max)
    }

    /// Returns the normalized position (0..1) of the handle along the track.
    fn ratio(&self) -> f32 {
        let range = self.max - self.min;
        if range <= 0.0 {
            0.0
        } else {
            ((self.value - self.min) / range).clamp(0.0, 1.0)
        }
    }
}

impl VectarineWidget for Slider {
    fn size(&self) -> Vec2 {
        self.size
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
        current_state: EventState,
        process_child_events: bool,
        draw_debug_outline: bool,
        extra: mlua::Value,
    ) -> mlua::Result<()> {
        let slider_width = self.size.x();
        let slider_height = self.size.y();
        let handle_width = self.handle.0.borrow().size().x();

        // The travel range for the handle's left edge
        let travel = (slider_width - handle_width).max(0.0);

        // Handle mouse interaction for dragging
        if current_state.is_mouse_inside || self.dragging {
            let io = io_env.borrow();
            let mouse_state = &io.mouse_state;
            let transform = batch.borrow().affine_transform;
            let local_mouse = transform.inverse_apply(&Vec2::new(mouse_state.x, mouse_state.y));
            let local_mx = local_mouse.x() - (-1.0); // relative to widget left edge

            if current_state.is_mouse_just_pressed {
                self.dragging = true;
            }

            if self.dragging && travel > 0.0 {
                let ratio = (local_mx - handle_width / 2.0) / travel;
                let raw_value = self.min + ratio.clamp(0.0, 1.0) * (self.max - self.min);
                let new_value = self.snap_value(raw_value);

                if (new_value - self.value).abs() > f32::EPSILON {
                    self.value = new_value;
                    if let Some(ref on_change) = self.on_change {
                        on_change.call::<()>((self.value,))?;
                    }
                }
            }
        }

        // Stop dragging when mouse is released
        if self.dragging {
            let io = io_env.borrow();
            if !io.mouse_state.is_left_down {
                self.dragging = false;
            }
        }

        let ratio = self.ratio();
        let handle_x = ratio * travel;

        // Draw the track widget (fills the full slider size)
        let current_transform = batch.borrow().affine_transform;

        // Track: draw at the slider's position, spanning the full width
        let track_height = self.track.0.borrow().size().y();
        let track_y_offset = (slider_height - track_height) / 2.0;
        batch.borrow_mut().affine_transform = current_transform.combine(&AffineTransform::new(
            Vec2::new(0.0, track_y_offset),
            Vec2::new(1.0, 1.0),
            0.0,
        ));
        let result = self.track.0.borrow_mut().event_processing_draw(
            lua,
            batch,
            io_env,
            process_child_events,
            draw_debug_outline,
            extra.clone(),
        );
        batch.borrow_mut().affine_transform = current_transform;
        result?;

        // Handle: draw at the computed horizontal position, vertically centered
        let handle_height = self.handle.0.borrow().size().y();
        let handle_y_offset = (slider_height - handle_height) / 2.0;
        batch.borrow_mut().affine_transform = current_transform.combine(&AffineTransform::new(
            Vec2::new(handle_x, handle_y_offset),
            Vec2::new(1.0, 1.0),
            0.0,
        ));
        let result = self.handle.0.borrow_mut().event_processing_draw(
            lua,
            batch,
            io_env,
            process_child_events,
            draw_debug_outline,
            extra,
        );
        batch.borrow_mut().affine_transform = current_transform;
        result?;

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(Slider {
            size: self.size,
            min: self.min,
            max: self.max,
            value: self.value,
            step: self.step,
            on_change: self.on_change.clone(),
            track: self.track.clone(),
            handle: self.handle.clone(),
            event_state: self.event_state.clone(),
            dragging: self.dragging,
        })
    }

    fn debug_label(&self) -> String {
        format!(
            "Slider({}, {})",
            self.track.0.borrow().debug_label(),
            self.handle.0.borrow().debug_label()
        )
    }
}
