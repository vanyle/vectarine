mod column_widget;
mod generic_widget;
mod image_widget;
mod row_widget;
mod scrollable_area_widget;
mod stack_widget;
mod text_widget;

use std::{cell::RefCell, rc::Rc};

use crate::auto_impl_lua_clone;
use crate::graphics::batchdraw;
use crate::graphics::shape::Quad;
use crate::io::IoEnvState;
use vectarine_plugin_sdk::mlua::{self, userdata::UserDataMethods};
use vectarine_plugin_sdk::mlua::{FromLua, IntoLua};

use crate::{game_resource, io, lua_env::lua_vec2::Vec2};

use column_widget::Column;
use generic_widget::GenericWidget;
use image_widget::ImageWidget;
use row_widget::Row;
use scrollable_area_widget::ScrollableArea;
use stack_widget::Stack;
use text_widget::TextWidget;

// MARK: Widget Trait

pub trait WidgetToAny: 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn std::any::Any>;
}

impl<T: VectarineWidget + 'static> WidgetToAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn std::any::Any> {
        self
    }
}

/// Represents a UI widget in Vectarine from the Rust side.
pub trait VectarineWidget: WidgetToAny {
    fn size(&self) -> Vec2;
    fn draw(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        current_state: EventState,
        process_child_events: bool,
        extra: mlua::Value,
    );

    /// A dyn-compatible version of Clone, allowing us to deep copy widgets.
    fn clone_box(&self) -> Box<dyn VectarineWidget>;
    fn event_state_mut(&mut self) -> &mut EventState;
    fn event_state(&self) -> &EventState;

    fn event_processing_draw(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        process_events: bool,
        extra: mlua::Value,
    ) {
        let widget_size = self.size();
        let state = self.event_state_mut();
        if process_events {
            let io = io_env.borrow();
            let mouse_state = &io.mouse_state;
            let transform = batch.borrow().affine_transform;

            // Compute the 4 screen-space corners of the widget (handles rotation)
            let origin = Vec2::new(-1.0, -1.0);
            let bl = transform.apply(&origin);
            let br = transform.apply(&Vec2::new(origin.x() + widget_size.x(), origin.y()));
            let tr = transform.apply(&(origin + widget_size));
            let tl = transform.apply(&Vec2::new(origin.x(), origin.y() + widget_size.y()));

            let mouse = Vec2::new(mouse_state.x, mouse_state.y);
            let is_inside = Quad {
                p1: bl,
                p2: br,
                p3: tr,
                p4: tl,
            }
            .inside(mouse);

            state.is_mouse_just_entered = is_inside && !state.is_mouse_inside;
            state.is_mouse_just_exited = !is_inside && state.is_mouse_inside;
            state.is_mouse_just_pressed =
                is_inside && mouse_state.is_left_down && !state.is_mouse_down;
            state.is_mouse_just_released =
                is_inside && !mouse_state.is_left_down && state.is_mouse_down;

            state.is_mouse_inside = is_inside;
            state.is_mouse_down = mouse_state.is_left_down && is_inside;
        } else {
            // Events suppressed — clear all state
            *state = EventState::default();
        }
        let process_child_events = process_events && state.is_mouse_inside;
        let state = state.clone();
        self.draw(lua, batch, io_env, state, process_child_events, extra);
    }
}

/// Represents a UI widget in Lua. This is the only type Lua has access to.
pub struct WidgetBox(pub(crate) Box<dyn VectarineWidget>);
impl Clone for WidgetBox {
    fn clone(&self) -> Self {
        WidgetBox(self.0.clone_box())
    }
}
auto_impl_lua_clone!(WidgetBox, WidgetBox);

impl WidgetBox {
    pub fn get_underlying_widget<T: VectarineWidget>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref::<T>()
    }
}

// MARK: Shared types

#[derive(Debug, Clone, Default)]
pub struct EventState {
    pub is_mouse_inside: bool,
    pub is_mouse_down: bool,
    pub is_mouse_just_pressed: bool,
    pub is_mouse_just_released: bool,
    pub is_mouse_just_entered: bool,
    pub is_mouse_just_exited: bool,
}

impl EventState {
    pub fn to_lua(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
        let table = lua.create_table()?;
        table.raw_set("isMouseInside", self.is_mouse_inside)?;
        table.raw_set("isMouseDown", self.is_mouse_down)?;
        table.raw_set("isMouseJustPressed", self.is_mouse_just_pressed)?;
        table.raw_set("isMouseJustReleased", self.is_mouse_just_released)?;
        table.raw_set("isMouseJustEntered", self.is_mouse_just_entered)?;
        table.raw_set("isMouseJustExited", self.is_mouse_just_exited)?;
        Ok(table)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Alignment {
    Start, // Top-Left
    Center,
    End, // Bottom-Right
}

#[derive(Clone, Copy, Debug)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for Padding {
    fn default() -> Self {
        Padding {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

// MARK: Lua API

fn parse_padding_from_table(table: &mlua::Table) -> Padding {
    // padding can be a single number (uniform) or a table with top/bottom/left/right
    if let Ok(uniform) = table.raw_get::<f32>("padding") {
        return Padding {
            top: uniform,
            bottom: uniform,
            left: uniform,
            right: uniform,
        };
    }
    let padding_table = table.raw_get::<mlua::Table>("padding").ok();
    match padding_table {
        Some(pt) => Padding {
            top: pt.raw_get::<f32>("top").unwrap_or(0.0),
            bottom: pt.raw_get::<f32>("bottom").unwrap_or(0.0),
            left: pt.raw_get::<f32>("left").unwrap_or(0.0),
            right: pt.raw_get::<f32>("right").unwrap_or(0.0),
        },
        None => Padding::default(),
    }
}

fn parse_gap_from_table(table: &mlua::Table) -> f32 {
    table.raw_get::<f32>("gap").unwrap_or(0.0)
}

fn parse_alignment_from_table(table: &mlua::Table) -> Alignment {
    match table.raw_get::<String>("align").ok().as_deref() {
        Some("center") => Alignment::Center,
        Some("end") => Alignment::End,
        _ => Alignment::Start,
    }
}

pub fn setup_ui_api(
    lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    _resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let ui_module = lua.create_table()?;

    lua.register_userdata_type::<WidgetBox>(|registry| {
        registry.add_method("size", |_, widget, (): ()| Ok(widget.0.size()));
        registry.add_method_mut("draw", {
            let batch = batch.clone();
            let io_env = env_state.clone();
            move |lua, widget, extra: mlua::Value| {
                widget
                    .0
                    .event_processing_draw(lua, &batch, &io_env, true, extra);
                Ok(())
            }
        });
        registry.add_method("eventState", |lua, widget, (): ()| {
            widget.0.event_state().to_lua(lua)
        });
    })?;

    ui_module.raw_set(
        "widget",
        lua.create_function(|_lua, (size, draw_fn): (Vec2, mlua::Function)| {
            let widget = WidgetBox(Box::new(GenericWidget {
                size,
                draw_fn,
                event_state: EventState::default(),
            }));
            Ok(widget)
        })?,
    )?;

    ui_module.raw_set(
        "column",
        lua.create_function(|_lua, (options, children): (mlua::Table, Vec<WidgetBox>)| {
            let padding = parse_padding_from_table(&options);
            let gap = parse_gap_from_table(&options);
            let alignment = parse_alignment_from_table(&options);
            let column = WidgetBox(Box::new(Column {
                children,
                alignment,
                padding,
                gap,
                event_state: EventState::default(),
            }));
            Ok(column)
        })?,
    )?;

    ui_module.raw_set(
        "row",
        lua.create_function(|_lua, (options, children): (mlua::Table, Vec<WidgetBox>)| {
            let padding = parse_padding_from_table(&options);
            let gap = parse_gap_from_table(&options);
            let alignment = parse_alignment_from_table(&options);
            let row = WidgetBox(Box::new(Row {
                children,
                alignment,
                padding,
                gap,
                event_state: EventState::default(),
            }));
            Ok(row)
        })?,
    )?;

    ui_module.raw_set("scrollableArea", {
        let resources = _resources.clone();
        let gl = batch.borrow().drawing_target.gl().clone();
        lua.create_function(
            move |_lua,
                  (content, view_size, scrollbar_draw_fn): (
                WidgetBox,
                Vec2,
                Option<mlua::Function>,
            )| {
                let widget = WidgetBox(Box::new(ScrollableArea {
                    content,
                    view_size,
                    scroll_offset: 0.0,
                    scroll_speed: 0.1,
                    scrollbar_draw_fn,
                    resources: resources.clone(),
                    gl: gl.clone(),
                    event_state: EventState::default(),
                    dragging_scrollbar: false,
                }));
                Ok(widget)
            },
        )?
    })?;

    ui_module.raw_set("text", {
        let gl = batch.borrow().drawing_target.gl().clone();
        let resources = _resources.clone();
        lua.create_function(
            move |_lua, (size, options, get_text_fn): (Vec2, mlua::Table, mlua::Function)| {
                let align = match options.raw_get::<String>("align").ok().as_deref() {
                    Some("left") => Alignment::Start,
                    Some("right") => Alignment::End,
                    _ => Alignment::Center,
                };
                let font_id = options
                    .raw_get::<crate::lua_env::lua_text::FontResourceId>("font")
                    .unwrap_or_else(|_| crate::lua_env::lua_text::FontResourceId::default_font());
                let widget = WidgetBox(Box::new(TextWidget {
                    size,
                    get_text_fn,
                    gl: gl.clone(),
                    align,
                    font_id,
                    resources: resources.clone(),
                    event_state: EventState::default(),
                }));
                Ok(widget)
            },
        )?
    })?;

    ui_module.raw_set("image", {
        let resources = _resources.clone();
        lua.create_function(
            move |_lua,
                  (size, image_id, options): (
                Vec2,
                crate::lua_env::lua_image::ImageResourceId,
                mlua::Table,
            )| {
                let preserve_aspect_ratio = options
                    .raw_get::<bool>("preserveAspectRatio")
                    .unwrap_or(false);
                let tint_fn = options.raw_get::<mlua::Function>("tint").ok();
                let nine_slicing = options.raw_get::<f32>("nineSlicing").ok();

                let widget = WidgetBox(Box::new(ImageWidget {
                    size,
                    image_id,
                    resources: resources.clone(),
                    preserve_aspect_ratio,
                    tint_fn,
                    nine_slicing,
                    event_state: EventState::default(),
                }));
                Ok(widget)
            },
        )?
    })?;

    ui_module.raw_set(
        "stack",
        lua.create_function(|_lua, (options, children): (mlua::Table, Vec<WidgetBox>)| {
            let align_x = match options.raw_get::<String>("alignX").ok().as_deref() {
                Some("center") => Alignment::Center,
                Some("end") => Alignment::End,
                _ => Alignment::Start,
            };
            let align_y = match options.raw_get::<String>("alignY").ok().as_deref() {
                Some("center") => Alignment::Center,
                Some("end") => Alignment::End,
                _ => Alignment::Start,
            };
            let stack = WidgetBox(Box::new(Stack {
                children,
                align_x,
                align_y,
                event_state: EventState::default(),
            }));
            Ok(stack)
        })?,
    )?;

    Ok(ui_module)
}
