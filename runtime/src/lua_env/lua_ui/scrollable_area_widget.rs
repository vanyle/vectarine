use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::game_resource::ResourceManager;
use crate::graphics::affinetransform::AffineTransform;
use crate::graphics::batchdraw;
use crate::graphics::glstencil::draw_with_mask;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::mlua;

use super::{EventState, VectarineWidget, WidgetBox};

pub struct ScrollableArea {
    pub content: WidgetBox,
    pub view_size: Vec2,
    pub scroll_offset: f32,
    pub scroll_speed: f32,
    pub scrollbar_draw_fn: Option<mlua::Function>,
    pub resources: Rc<ResourceManager>,
    pub gl: Arc<glow::Context>,
    pub event_state: EventState,
    pub dragging_scrollbar: bool,
}

impl ScrollableArea {
    fn content_height(&self) -> f32 {
        self.content.0.size().y()
    }

    fn max_scroll(&self) -> f32 {
        (self.content_height() - self.view_size.y()).max(0.0)
    }

    /// Ratio of the visible portion to the total content height (0.0..=1.0).
    fn visible_ratio(&self) -> f32 {
        let ch = self.content_height();
        if ch <= 0.0 {
            1.0
        } else {
            (self.view_size.y() / ch).min(1.0)
        }
    }

    /// Scroll progress from 0.0 (top) to 1.0 (bottom).
    fn scroll_ratio(&self) -> f32 {
        let max = self.max_scroll();
        if max <= 0.0 {
            0.0
        } else {
            self.scroll_offset / max
        }
    }

    fn draw_default_scrollbar(
        &mut self,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        view_width: f32,
        view_height: f32,
        visible_ratio: f32,
        scroll_ratio: f32,
    ) {
        let bar_width: f32 = 0.02;
        let track_x: f32 = -1.0 + view_width - bar_width;
        let track_color = [0.3, 0.3, 0.3, 0.4];
        let thumb_color = if self.dragging_scrollbar {
            [0.9, 0.9, 0.9, 0.9]
        } else {
            [0.7, 0.7, 0.7, 0.8]
        };

        // Track
        batch
            .borrow_mut()
            .draw_rect(track_x, -1.0, bar_width, view_height, track_color);

        // Thumb
        let thumb_height = (visible_ratio * view_height).max(0.02);
        let thumb_travel = view_height - thumb_height;
        // scroll_ratio 0 = top, so thumb at top means high y
        let thumb_y = -1.0 + thumb_travel * (1.0 - scroll_ratio);
        batch
            .borrow_mut()
            .draw_rect(track_x, thumb_y, bar_width, thumb_height, thumb_color);

        // Handle scrollbar dragging
        let io = io_env.borrow();
        let mouse_state = &io.mouse_state;
        let transform = batch.borrow().affine_transform;
        let local_mouse = transform.inverse_apply(&Vec2::new(mouse_state.x, mouse_state.y));
        let local_mx = local_mouse.x();
        let local_my = local_mouse.y();

        let mouse_on_thumb = local_mx >= track_x
            && local_mx <= track_x + bar_width
            && local_my >= thumb_y
            && local_my <= thumb_y + thumb_height;

        if mouse_state.is_left_just_pressed && mouse_on_thumb {
            self.dragging_scrollbar = true;
        }
        if !mouse_state.is_left_down {
            self.dragging_scrollbar = false;
        }

        if self.dragging_scrollbar && thumb_travel > 0.0 {
            // Map mouse Y to scroll ratio: top of track (high y) = 0, bottom (low y) = 1
            let track_bottom = -1.0_f32;
            let ratio = 1.0 - ((local_my - track_bottom - thumb_height / 2.0) / thumb_travel);
            let new_ratio = ratio.clamp(0.0, 1.0);
            self.scroll_offset = new_ratio * self.max_scroll();
        }
    }
}

impl VectarineWidget for ScrollableArea {
    fn size(&self) -> Vec2 {
        self.view_size
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
        extra: mlua::Value,
    ) -> mlua::Result<()> {
        let content_height = self.content_height();
        let view_height = self.view_size.y();
        let view_width = self.view_size.x();
        let max_scroll = self.max_scroll();

        if current_state.is_mouse_inside {
            let wheel_y = io_env.borrow().mouse_state.wheel_y;
            self.scroll_offset =
                (self.scroll_offset - wheel_y * self.scroll_speed).clamp(0.0, max_scroll);
        }

        self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);

        // scroll_offset=0 means top of content is visible, scroll_offset=max means bottom visible.
        // content_translate_y shifts the content in local space.
        let content_translate_y =
            self.scroll_offset + (view_height - content_height).max(view_height - content_height);

        let current_transform = batch.borrow().affine_transform;

        // Flush batch before stencil operations
        batch.borrow_mut().draw(&self.resources, true);

        // Capture errors from the content draw closure since draw_with_mask expects FnOnce()
        let mut content_error: Option<mlua::Error> = None;

        draw_with_mask(
            &self.gl,
            || {
                // Mask: a rectangle covering the view area (clips both axes)
                batch.borrow_mut().draw_rect(
                    -1.0,
                    -1.0,
                    view_width,
                    view_height,
                    [1.0, 1.0, 1.0, 1.0],
                );
                batch.borrow_mut().draw(&self.resources, true);
            },
            || {
                // Content: translate by scroll offset and draw
                batch.borrow_mut().affine_transform =
                    current_transform.combine(&AffineTransform::new(
                        Vec2::new(0.0, content_translate_y),
                        Vec2::new(1.0, 1.0),
                        0.0,
                    ));
                // Only process child events if mouse is inside the scrollable area
                let result = self.content.0.event_processing_draw(
                    lua,
                    batch,
                    io_env,
                    process_child_events,
                    extra,
                );
                if let Err(e) = result {
                    content_error = Some(e);
                }
                batch.borrow_mut().draw(&self.resources, true);
                batch.borrow_mut().affine_transform = current_transform;
            },
        );

        if let Some(err) = content_error {
            return Err(err);
        }

        // Draw scrollbar (only when content overflows)
        if max_scroll > 0.0 {
            let scroll_ratio = self.scroll_ratio();
            let visible_ratio = self.visible_ratio();
            if let Some(ref draw_fn) = self.scrollbar_draw_fn {
                let info = lua.create_table()?;
                info.raw_set("scrollRatio", scroll_ratio)?;
                info.raw_set("visibleRatio", visible_ratio)?;
                info.raw_set("viewWidth", view_width)?;
                info.raw_set("viewHeight", view_height)?;
                info.raw_set("contentHeight", content_height)?;
                let returned_ratio = draw_fn.call::<f32>((info,))?;
                let clamped = returned_ratio.clamp(0.0, 1.0);
                self.scroll_offset = clamped * max_scroll;
            } else {
                self.draw_default_scrollbar(
                    batch,
                    io_env,
                    view_width,
                    view_height,
                    visible_ratio,
                    scroll_ratio,
                );
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(ScrollableArea {
            content: self.content.clone(),
            view_size: self.view_size,
            scroll_offset: self.scroll_offset,
            scroll_speed: self.scroll_speed,
            scrollbar_draw_fn: self.scrollbar_draw_fn.clone(),
            resources: self.resources.clone(),
            gl: self.gl.clone(),
            event_state: self.event_state.clone(),
            dragging_scrollbar: self.dragging_scrollbar,
        })
    }

    fn debug_label(&self) -> String {
        format!("ScrollableArea({})", self.content.0.debug_label())
    }
}
