use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::game_resource::ResourceManager;
use crate::game_resource::font_resource::FontRenderingData;
use crate::graphics::batchdraw;
use crate::io::IoEnvState;
use crate::lua_env::lua_text::FontResourceId;
use crate::lua_env::lua_vec2::Vec2;
use crate::lua_env::lua_vec4::Vec4;
use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::mlua;

use super::{Alignment, EventState, VectarineWidget};

struct TextLayout {
    font_size: f32,
    aspect_ratio: f32,
    widget_width: f32,
    align: Alignment,
}

impl TextLayout {
    fn measure_width(&self, font_renderer: &FontRenderingData, text: &str) -> f32 {
        font_renderer
            .measure_text(text, self.font_size, self.aspect_ratio)
            .0
    }
}

pub struct TextWidget {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub get_text_fn: mlua::Function,
    pub get_color_fn: Option<mlua::Function>,
    pub font_id: FontResourceId,
    pub font_size: f32,
    pub align: Alignment,
    // For now, we don't handle line-height or letter spacing.
    pub gl: Arc<glow::Context>,
    pub resources: Rc<ResourceManager>,
    pub event_state: EventState,
}

impl VectarineWidget for TextWidget {
    // Currently, performance is `size` is very bad.
    // We could cache the value in case get_text_fn is actually just a constant.
    // We'll do that later if people complain.
    fn size(&self, lua: &mlua::Lua, io_env: &RefCell<IoEnvState>, extra: &mlua::Value) -> Vec2 {
        let aspect_ratio = {
            let io = io_env.borrow();
            io.window_width as f32 / io.window_height as f32
        };
        match (self.width, self.height) {
            // Easy case: width and height are known and are the exact size of the widget.
            (Some(w), Some(h)) => Vec2::new(w, h),
            (Some(w), None) => {
                // The widget is `w` wide, and the text wraps to as many lines as it needs.
                let event_table = self
                    .event_state
                    .to_lua(lua)
                    .expect("Conversion to table should never fail");
                let text = self
                    .get_text_fn
                    .call::<String>((&event_table, extra))
                    .expect("Failed to call get_text_fn");

                let layout = TextLayout {
                    font_size: self.font_size,
                    aspect_ratio,
                    widget_width: w,
                    align: self.align,
                };

                let line_count =
                    self.font_id
                        .get_font_resource(&self.gl, &self.resources, |font_renderer| {
                            wrap_lines(font_renderer, &text, &layout).len()
                        });

                let Some(line_count) = line_count else {
                    // Font not loaded => Text not rendered, so we return a height of 0.0.
                    return Vec2::new(w, 0.0);
                };

                // A line height is, by definition, the font_size, and we always show at least one line.
                Vec2::new(w, line_count.max(1) as f32 * self.font_size)
            }
            (None, maybe_height) => {
                // The height is the height of a single line, and the width is the width of the text.
                let event_table = self
                    .event_state
                    .to_lua(lua)
                    .expect("Conversion to table should never fail");
                let text = self
                    .get_text_fn
                    .call::<String>((&event_table, extra))
                    .expect("Failed to call get_text_fn");

                let Some((w, _h, _max_ascent)) =
                    self.font_id
                        .get_font_resource(&self.gl, &self.resources, |font_renderer| {
                            font_renderer.measure_text(&text, self.font_size, aspect_ratio)
                        })
                else {
                    // Font not loaded => Text not rendered, so we return a size of 0.0, 0.0
                    return Vec2::new(0.0, maybe_height.unwrap_or(0.0));
                };
                // The height of the line should not depend on the actual written text!
                // Remember that font_size is, by definition, the maximum height a line of text can have.
                Vec2::new(w, maybe_height.unwrap_or(self.font_size))
            }
        }
    }

    fn draw(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        current_state: EventState,
        _process_child_events: bool,
        _draw_debug_outline: bool,
        extra: mlua::Value,
    ) -> mlua::Result<()> {
        let event_table = current_state
            .to_lua(lua)
            .expect("Conversion to table should never fail");
        // Double-borrow issue can happen here as get_text_fn might need to borrow self.
        let text = { self.get_text_fn.call::<String>((&event_table, &extra))? };
        let color = match &self.get_color_fn {
            None => Vec4::new(0.0, 0.0, 0.0, 1.0),
            Some(function) => function.call::<Vec4>((&event_table, &extra))?,
        };

        let aspect_ratio = {
            let io = io_env.borrow();
            io.window_width as f32 / io.window_height as f32
        };

        let true_size = self.size(lua, io_env, &extra);

        self.font_id
            .get_font_resource(&self.gl, &self.resources, |font_renderer| {
                draw_fixed_size(
                    font_renderer,
                    &self.gl,
                    batch,
                    &text,
                    color.0,
                    TextLayout {
                        font_size: self.font_size,
                        aspect_ratio,
                        widget_width: true_size.x(),
                        align: self.align,
                    },
                    true_size.y(),
                );
            });
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(TextWidget {
            get_text_fn: self.get_text_fn.clone(),
            gl: self.gl.clone(),
            align: self.align,
            font_id: self.font_id,
            resources: self.resources.clone(),
            event_state: self.event_state.clone(),
            get_color_fn: self.get_color_fn.clone(),
            font_size: self.font_size,
            width: self.width,
            height: self.height,
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

fn draw_aligned_line(
    font_renderer: &FontRenderingData,
    batch: &RefCell<batchdraw::BatchDraw2d>,
    line: &str,
    color: [f32; 4],
    layout: &TextLayout,
    y: f32,
) {
    let line_width = layout.measure_width(font_renderer, line);

    let x = -1.0
        + match layout.align {
            Alignment::Start => 0.0,
            Alignment::Center => (layout.widget_width - line_width) / 2.0,
            Alignment::End => layout.widget_width - line_width,
        };

    batch
        .borrow_mut()
        .draw_text(x, y, line, color, layout.font_size, font_renderer);
}

/// Wraps `text` into lines that fit within the layout's widget width.
/// Words are split on spaces. If a single word is wider than the width, it gets its own line.
fn wrap_lines(font_renderer: &FontRenderingData, text: &str, layout: &TextLayout) -> Vec<String> {
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        let words: Vec<&str> = paragraph.split(' ').collect();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();

        for word in &words {
            if current_line.is_empty() {
                let word_width = layout.measure_width(font_renderer, word);
                if word_width > layout.widget_width {
                    lines.push(word.to_string());
                } else {
                    current_line = word.to_string();
                }
            } else {
                let candidate = format!("{current_line} {word}");
                let candidate_width = layout.measure_width(font_renderer, &candidate);

                if candidate_width <= layout.widget_width {
                    current_line = candidate;
                } else {
                    lines.push(current_line);
                    current_line = word.to_string();
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    lines
}

/// Truncates a line so it fits in the layout's widget width when "..." is appended.
fn truncate_with_ellipsis(
    font_renderer: &FontRenderingData,
    line: &str,
    layout: &TextLayout,
) -> String {
    let ellipsis = "...";
    let ellipsis_width = layout.measure_width(font_renderer, ellipsis);
    if ellipsis_width > layout.widget_width {
        return ellipsis.to_string();
    }

    let mut truncated = String::new();
    for c in line.chars() {
        truncated.push(c);
        let w = layout.measure_width(font_renderer, &truncated);
        if w + ellipsis_width > layout.widget_width {
            truncated.pop();
            break;
        }
    }
    truncated.push_str(ellipsis);
    truncated
}

fn draw_fixed_size(
    font_renderer: &mut FontRenderingData,
    gl: &Arc<glow::Context>,
    batch: &RefCell<batchdraw::BatchDraw2d>,
    text: &str,
    color: [f32; 4],
    layout: TextLayout,
    widget_height: f32,
) {
    let mut lines = wrap_lines(font_renderer, text, &layout);

    let line_height = layout.font_size;
    let baseline_height = font_renderer.get_max_baseline_height(layout.font_size);
    let max_lines = ((widget_height + 0.001) / line_height).floor().max(1.0) as usize;

    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            *last = truncate_with_ellipsis(font_renderer, last, &layout);
        }
    }

    let all_text: String = lines.join("");
    font_renderer.enrich_atlas(gl, &all_text);

    for (i, line) in lines.iter().enumerate() {
        let y = -1.0 + widget_height - (i as f32) * line_height - line_height + baseline_height;
        draw_aligned_line(font_renderer, batch, line, color, &layout, y);
    }
}
