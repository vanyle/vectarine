use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc, time::Instant};

use crate::game_resource::ResourceManager;
use crate::graphics::batchdraw;
use crate::graphics::glstencil::draw_with_mask;
use crate::io::IoEnvState;
use crate::lua_env::lua_vec2::Vec2;
use vectarine_plugin_sdk::glow;
use vectarine_plugin_sdk::mlua;

use super::{EventState, VectarineWidget, WidgetBox};

#[derive(Clone, Debug)]
pub enum TabTransitionStyle {
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Toon,
    Custom(mlua::Function),
}

#[derive(Clone)]
pub struct TabTransitionState {
    pub old_tab: String,
    pub duration: f32,
    pub start_time: Instant,
    pub style: TabTransitionStyle,
}

pub struct TabWidget {
    pub tabs: HashMap<String, WidgetBox>,
    pub current_tab: String,
    pub transition: Option<TabTransitionState>,
    pub gl: Arc<glow::Context>,
    pub resources: Rc<ResourceManager>,
    pub event_state: EventState,
}

impl TabWidget {
    pub fn set_active_tab(
        &mut self,
        tab_name: String,
        transition: Option<(f32, TabTransitionStyle)>,
    ) {
        if tab_name == self.current_tab {
            return;
        }
        let old_tab = self.current_tab.clone();
        self.current_tab = tab_name;
        self.transition = transition.map(|(duration, style)| TabTransitionState {
            old_tab,
            duration,
            start_time: Instant::now(),
            style,
        });
    }
}

impl VectarineWidget for TabWidget {
    fn size(&self) -> Vec2 {
        self.tabs
            .get(&self.current_tab)
            .map(|w| w.0.borrow().size())
            .unwrap_or(Vec2::new(0.0, 0.0))
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
        draw_debug_outline: bool,
        extra: mlua::Value,
    ) -> mlua::Result<()> {
        // Compute transition progress from wall-clock time
        let transition_info = self.transition.as_ref().map(|t| {
            let progress = (t.start_time.elapsed().as_secs_f32() / t.duration).min(1.0);
            (t.old_tab.clone(), t.style.clone(), progress)
        });

        // Clear completed transitions
        if matches!(&transition_info, Some((_, _, p)) if *p >= 1.0) {
            self.transition = None;
        }

        if let Some((old_tab_key, style, progress)) = transition_info
            && progress < 1.0
        {
            let widget_size = self.size();
            let w = widget_size.x();
            let h = widget_size.y();

            match style {
                TabTransitionStyle::SlideUp => {
                    return self.draw_wipe(
                        lua,
                        batch,
                        io_env,
                        process_child_events,
                        draw_debug_outline,
                        extra,
                        &old_tab_key,
                        (-1.0, -1.0 + progress * h, w, (1.0 - progress) * h),
                        (-1.0, -1.0, w, progress * h),
                    );
                }
                TabTransitionStyle::SlideDown => {
                    return self.draw_wipe(
                        lua,
                        batch,
                        io_env,
                        process_child_events,
                        draw_debug_outline,
                        extra,
                        &old_tab_key,
                        (-1.0, -1.0, w, (1.0 - progress) * h),
                        (-1.0, -1.0 + (1.0 - progress) * h, w, progress * h),
                    );
                }
                TabTransitionStyle::SlideLeft => {
                    return self.draw_wipe(
                        lua,
                        batch,
                        io_env,
                        process_child_events,
                        draw_debug_outline,
                        extra,
                        &old_tab_key,
                        (-1.0, -1.0, (1.0 - progress) * w, h),
                        (-1.0 + (1.0 - progress) * w, -1.0, progress * w, h),
                    );
                }
                TabTransitionStyle::SlideRight => {
                    return self.draw_wipe(
                        lua,
                        batch,
                        io_env,
                        process_child_events,
                        draw_debug_outline,
                        extra,
                        &old_tab_key,
                        (-1.0 + progress * w, -1.0, (1.0 - progress) * w, h),
                        (-1.0, -1.0, progress * w, h),
                    );
                }
                TabTransitionStyle::Toon => {
                    return self.draw_toon(
                        lua,
                        batch,
                        io_env,
                        process_child_events,
                        draw_debug_outline,
                        extra,
                        &old_tab_key,
                        w,
                        h,
                        progress,
                    );
                }
                TabTransitionStyle::Custom(func) => {
                    let old_widget = self.tabs.get(&old_tab_key).cloned();
                    let new_widget = self.tabs.get(&self.current_tab).cloned();
                    return func.call::<()>((extra, old_widget, new_widget, progress));
                }
            }
        }

        // No transition — draw current tab
        if let Some(widget) = self.tabs.get(&self.current_tab) {
            widget.0.borrow_mut().event_processing_draw(
                lua,
                batch,
                io_env,
                process_child_events,
                draw_debug_outline,
                extra,
            )?;
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(TabWidget {
            tabs: self
                .tabs
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            current_tab: self.current_tab.clone(),
            transition: self.transition.clone(),
            gl: self.gl.clone(),
            resources: self.resources.clone(),
            event_state: self.event_state.clone(),
        })
    }

    fn debug_label(&self) -> String {
        let tab_labels: Vec<String> = self.tabs.keys().map(|k| format!("\"{}\"", k)).collect();
        format!("Tabs({})", tab_labels.join(", "))
    }
}

impl TabWidget {
    /// Draws a wipe transition (slide) between old and new tabs using stencil masks.
    #[allow(clippy::too_many_arguments)]
    fn draw_wipe(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        process_child_events: bool,
        draw_debug_outline: bool,
        extra: mlua::Value,
        old_tab_key: &str,
        old_mask: (f32, f32, f32, f32),
        new_mask: (f32, f32, f32, f32),
    ) -> mlua::Result<()> {
        let extra_for_new = extra.clone();
        batch.borrow_mut().draw(&self.resources, true);

        // Draw old tab with shrinking mask (no event processing)
        if let Some(old_widget) = self.tabs.get(old_tab_key) {
            let (_, content_result) = draw_with_mask(
                &self.gl,
                || {
                    batch.borrow_mut().draw_rect(
                        old_mask.0,
                        old_mask.1,
                        old_mask.2,
                        old_mask.3,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                    batch.borrow_mut().draw(&self.resources, true);
                },
                || {
                    let result = old_widget.0.borrow_mut().event_processing_draw(
                        lua,
                        batch,
                        io_env,
                        false,
                        draw_debug_outline,
                        extra,
                    );
                    batch.borrow_mut().draw(&self.resources, true);
                    result
                },
            );
            content_result?;
        }

        // Draw new tab with growing mask
        if let Some(new_widget) = self.tabs.get(&self.current_tab) {
            let (_, content_result) = draw_with_mask(
                &self.gl,
                || {
                    batch.borrow_mut().draw_rect(
                        new_mask.0,
                        new_mask.1,
                        new_mask.2,
                        new_mask.3,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                    batch.borrow_mut().draw(&self.resources, true);
                },
                || {
                    let result = new_widget.0.borrow_mut().event_processing_draw(
                        lua,
                        batch,
                        io_env,
                        process_child_events,
                        draw_debug_outline,
                        extra_for_new,
                    );
                    batch.borrow_mut().draw(&self.resources, true);
                    result
                },
            );
            content_result?;
        }

        Ok(())
    }

    /// Draws a circle-wipe (toon) transition: old tab fully visible, new tab revealed through expanding circle.
    #[allow(clippy::too_many_arguments)]
    fn draw_toon(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        process_child_events: bool,
        draw_debug_outline: bool,
        extra: mlua::Value,
        old_tab_key: &str,
        w: f32,
        h: f32,
        progress: f32,
    ) -> mlua::Result<()> {
        let extra_for_new = extra.clone();

        // Draw old tab fully (no mask, no event processing)
        if let Some(old_widget) = self.tabs.get(old_tab_key) {
            old_widget.0.borrow_mut().event_processing_draw(
                lua,
                batch,
                io_env,
                false,
                draw_debug_outline,
                extra,
            )?;
        }

        batch.borrow_mut().draw(&self.resources, true);

        // Draw new tab inside expanding circle centered on the widget
        let cx = -1.0 + w / 2.0;
        let cy = -1.0 + h / 2.0;
        let max_radius = (w * w + h * h).sqrt() / 2.0;
        let radius = progress * max_radius;

        if let Some(new_widget) = self.tabs.get(&self.current_tab) {
            let (_, content_result) = draw_with_mask(
                &self.gl,
                || {
                    batch
                        .borrow_mut()
                        .draw_circle(cx, cy, radius, [1.0, 1.0, 1.0, 1.0]);
                    batch.borrow_mut().draw(&self.resources, true);
                },
                || {
                    let result = new_widget.0.borrow_mut().event_processing_draw(
                        lua,
                        batch,
                        io_env,
                        process_child_events,
                        draw_debug_outline,
                        extra_for_new,
                    );
                    batch.borrow_mut().draw(&self.resources, true);
                    result
                },
            );
            content_result?;
        }

        Ok(())
    }
}
