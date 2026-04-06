use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::game_resource::ResourceManager;
use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_text::FontResourceId;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::mlua;

use super::{Alignment, EventState, VectarineWidget};

pub struct TextWidget {
    pub size: Vec2,
    pub get_text_fn: mlua::Function,
    pub gl: Arc<glow::Context>,
    pub align: Alignment,
    pub font_id: FontResourceId,
    pub resources: Rc<ResourceManager>,
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
        extra: mlua::Value,
    ) -> mlua::Result<()> {
        let event_table = current_state
            .to_lua(lua)
            .expect("Conversion to table should never fail");
        let result = self.get_text_fn.call::<mlua::Table>((event_table, extra))?;
        let text = result.raw_get::<String>("text")?;
        let color: [f32; 4] = match result.raw_get::<crate::lua_env::lua_vec4::Vec4>("color") {
            Ok(c) => c.0,
            Err(_) => [1.0, 1.0, 1.0, 1.0],
        };

        let widget_width = self.size.x();
        let widget_height = self.size.y();

        let io = io_env.borrow();
        let aspect_ratio = io.window_width as f32 / io.window_height as f32;

        let align = self.align;
        self.font_id
            .get_font_resource(&self.gl, &self.resources, |font_renderer| {
                // Because font_size is linear, we calculate everything based on a font size of 1.0
                let (measured_width, _measured_height, _max_ascent) =
                    font_renderer.measure_text(&text, 1.0, aspect_ratio);

                let font_size_that_fills_the_height = widget_height;

                let final_font_size =
                    if measured_width * font_size_that_fills_the_height > widget_width {
                        widget_width / measured_width
                    } else {
                        font_size_that_fills_the_height
                    };

                let text_width = measured_width * final_font_size;

                let x = -1.0
                    + match align {
                        Alignment::Start => 0.0,
                        Alignment::Center => (widget_width - text_width) / 2.0,
                        Alignment::End => widget_width - text_width,
                    };

                font_renderer.enrich_atlas(&self.gl, &text);
                batch.borrow_mut().draw_text(
                    x,
                    -1.0 + font_renderer.get_max_baseline_height(final_font_size),
                    &text,
                    color,
                    final_font_size,
                    font_renderer,
                );
            });
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(TextWidget {
            size: self.size,
            get_text_fn: self.get_text_fn.clone(),
            gl: self.gl.clone(),
            align: self.align,
            font_id: self.font_id,
            resources: self.resources.clone(),
            event_state: self.event_state.clone(),
        })
    }

    fn event_state_mut(&mut self) -> &mut EventState {
        &mut self.event_state
    }

    fn event_state(&self) -> &EventState {
        &self.event_state
    }

    fn debug_label(&self) -> String {
        "Text".to_string()
    }
}
