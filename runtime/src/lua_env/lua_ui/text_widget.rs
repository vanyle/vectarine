use std::{cell::RefCell, sync::Arc};

use crate::game_resource::font_resource;
use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::mlua;

use super::{EventState, VectarineWidget};

pub struct TextWidget {
    pub size: Vec2,
    pub get_text_fn: mlua::Function,
    pub gl: Arc<glow::Context>,
    pub event_state: EventState,
}

impl VectarineWidget for TextWidget {
    fn size(&self) -> Vec2 {
        self.size
    }

    fn draw(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        current_state: EventState,
        _process_child_events: bool,
    ) {
        let event_table = current_state
            .to_lua(lua)
            .expect("Conversion to table should never fail");
        let Ok(result) = self.get_text_fn.call::<mlua::Table>((event_table,)) else {
            return;
        };
        let Ok(text) = result.raw_get::<String>("text") else {
            return;
        };
        let color: [f32; 4] = match result.raw_get::<crate::lua_env::lua_vec4::Vec4>("color") {
            Ok(c) => c.0,
            Err(_) => [1.0, 1.0, 1.0, 1.0],
        };

        let widget_width = self.size.x();
        let widget_height = self.size.y();

        let io = io_env.borrow();
        let aspect_ratio = io.window_width as f32 / io.window_height as f32;

        let gl = &self.gl;
        font_resource::use_default_font(gl, |font_renderer| {
            let font_size = widget_height;

            let (measured_width, _measured_height, _max_ascent) =
                font_renderer.measure_text(&text, font_size, aspect_ratio);

            let final_font_size = if measured_width > widget_width {
                font_size * (widget_width / measured_width)
            } else {
                font_size
            };

            let (final_width, final_height, _) =
                font_renderer.measure_text(&text, final_font_size, aspect_ratio);

            // Center the text within the widget area
            let x = -1.0 + (widget_width - final_width) / 2.0;
            let y = -1.0 + (widget_height - final_height) / 2.0;

            font_renderer.enrich_atlas(gl, &text);
            batch
                .borrow_mut()
                .draw_text(x, y, &text, color, final_font_size, font_renderer);
        });
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(TextWidget {
            size: self.size,
            get_text_fn: self.get_text_fn.clone(),
            gl: self.gl.clone(),
            event_state: self.event_state.clone(),
        })
    }

    fn event_state_mut(&mut self) -> &mut EventState {
        &mut self.event_state
    }

    fn event_state(&self) -> &EventState {
        &self.event_state
    }
}
