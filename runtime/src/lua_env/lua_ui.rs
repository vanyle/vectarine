use std::{cell::RefCell, rc::Rc};

use crate::auto_impl_lua_clone;
use crate::graphics::affinetransform::AffineTransform;
use crate::io::IoEnvState;
use vectarine_plugin_sdk::mlua::{self, userdata::UserDataMethods};
use vectarine_plugin_sdk::mlua::{FromLua, IntoLua};

use crate::{
    game_resource::{self},
    graphics::batchdraw,
    io,
    lua_env::lua_vec2::Vec2,
};

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
    ) {
        let widget_size = self.size();
        let state = self.event_state_mut();
        if process_events {
            let io = io_env.borrow();
            let mouse_state = &io.mouse_state;
            let transform = batch.borrow().affine_transform;

            let widget_bottom_left = transform.apply(&Vec2::new(-1.0, -1.0));
            let widget_top_right = transform.apply(&(Vec2::new(-1.0, -1.0) + widget_size));

            let is_inside = mouse_state.x >= widget_bottom_left.x()
                && mouse_state.x <= widget_top_right.x()
                && mouse_state.y >= widget_bottom_left.y()
                && mouse_state.y <= widget_top_right.y();

            state.is_mouse_just_entered = is_inside && !state.is_mouse_inside;
            state.is_mouse_just_exited = !is_inside && state.is_mouse_inside;
            state.is_mouse_just_pressed =
                is_inside && mouse_state.is_left_down && !state.is_mouse_down;
            state.is_mouse_just_released =
                is_inside && !mouse_state.is_left_down && state.is_mouse_down;

            state.is_mouse_inside = is_inside;
            state.is_mouse_down = mouse_state.is_left_down && is_inside;
        }
        let state = state.clone();
        self.draw(lua, batch, io_env, state);
    }
}

/// Represents a UI widget in Lua. This is the only type Lua has access to.
pub struct WidgetBox(Box<dyn VectarineWidget>);
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

// MARK: Generic
#[derive(Debug, Clone)]
pub struct GenericWidget {
    size: Vec2,
    draw_fn: mlua::Function,
    event_state: EventState,
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
    ) {
        let _ = self.draw_fn.call::<(mlua::Table,)>((current_state
            .to_lua(lua)
            .expect("Convertion to table should never fail"),));
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
}

// MARK: Column

#[derive(Clone, Copy, Debug)]
enum Alignment {
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

pub struct Column {
    children: Vec<WidgetBox>,
    alignment: Alignment,
    padding: Padding,
    gap: f32,
    event_state: EventState,
}

impl VectarineWidget for Column {
    fn size(&self) -> Vec2 {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        for child in &self.children {
            let child_size: crate::math::Vect<2> = child.0.size();
            width = width.max(child_size.x());
            height += child_size.y();
        }
        let gap_total = if self.children.len() > 1 {
            self.gap * (self.children.len() - 1) as f32
        } else {
            0.0
        };
        Vec2::new(
            width + self.padding.left + self.padding.right,
            height + gap_total + self.padding.top + self.padding.bottom,
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
    ) {
        let container_width = self.size().x() - self.padding.left - self.padding.right;
        let mut y_offset = self.padding.bottom;
        // Reverse order because top is Y+, so the first child shown at the top needs to be the last drawn.
        for child in self.children.iter_mut().rev() {
            let child_size = child.0.size();
            let child_width = child_size.x();
            let x_offset = self.padding.left
                + match self.alignment {
                    Alignment::Start => 0.0,
                    Alignment::Center => (container_width - child_width) / 2.0,
                    Alignment::End => container_width - child_width,
                };
            let current_transform = batch.borrow().affine_transform;
            batch.borrow_mut().affine_transform = current_transform.combine(&AffineTransform::new(
                Vec2::new(x_offset, y_offset),
                Vec2::new(1.0, 1.0),
                0.0,
            ));
            child.0.event_processing_draw(lua, batch, io_env, true);
            batch.borrow_mut().affine_transform = current_transform;
            y_offset += child_size.y() + self.gap;
        }
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(Column {
            children: self.children.to_vec(),
            alignment: self.alignment,
            padding: self.padding,
            gap: self.gap,
            event_state: self.event_state.clone(),
        })
    }
}

// MARK: Row

pub struct Row {
    children: Vec<WidgetBox>,
    alignment: Alignment,
    padding: Padding,
    gap: f32,
    event_state: EventState,
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
    ) {
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
            child.0.event_processing_draw(lua, batch, io_env, true);
            batch.borrow_mut().affine_transform = current_transform;
            x_offset += child_size.x() + self.gap;
        }
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
            move |lua, widget, (): ()| {
                widget.0.event_processing_draw(lua, &batch, &io_env, true);
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

    Ok(ui_module)
}
