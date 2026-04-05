use std::{cell::RefCell, rc::Rc};

use crate::auto_impl_lua_clone;
use crate::graphics::affinetransform::AffineTransform;
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
    // Drawing mutates a batch, so we need more arguments here.
    fn draw(&self, batch: &RefCell<batchdraw::BatchDraw2d>);
    /// A dyn-compatible version of Clone, allowing us to deep copy widgets.
    fn clone_box(&self) -> Box<dyn VectarineWidget>;
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

// MARK: Generic
#[derive(Debug, Clone)]
pub struct GenericWidget {
    size: Vec2,
    draw_fn: mlua::Function,
}

impl VectarineWidget for GenericWidget {
    fn size(&self) -> Vec2 {
        self.size
    }
    fn draw(&self, _batch: &RefCell<batchdraw::BatchDraw2d>) {
        let _ = self.draw_fn.call::<()>(());
    }
    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(self.clone())
    }
}

// MARK: Column

#[derive(Clone, Copy, Debug)]
enum Alignment {
    Start, // Top-Left
    Center,
    End, // Bottom-Right
}

pub struct Column {
    children: Vec<WidgetBox>,
    alignment: Alignment,
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
        Vec2::new(width, height)
    }

    fn draw(&self, batch: &RefCell<batchdraw::BatchDraw2d>) {
        let mut y_offset = 0.0;
        for child in &self.children {
            let current_transform = batch.borrow().affine_transform;
            batch.borrow_mut().affine_transform = current_transform.combine(&AffineTransform::new(
                Vec2::new(0.0, y_offset),
                Vec2::new(1.0, 1.0),
                0.0,
            ));
            // As draw can borrow mutably, we need to separate mutable borrows of the batch here.
            child.0.draw(batch); // We ignore alignment currently.
            batch.borrow_mut().affine_transform = current_transform;
            y_offset += child.0.size().y();
        }
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(Column {
            children: self.children.to_vec(),
            alignment: self.alignment,
        })
    }
}

// MARK: Row

pub struct Row {
    children: Vec<WidgetBox>,
    alignment: Alignment,
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
        Vec2::new(width, height)
    }

    fn draw(&self, batch: &RefCell<batchdraw::BatchDraw2d>) {
        let mut x_offset = 0.0;
        for child in &self.children {
            let current_transform = batch.borrow().affine_transform;
            batch.borrow_mut().affine_transform = current_transform.combine(&AffineTransform::new(
                Vec2::new(x_offset, 0.0),
                Vec2::new(1.0, 1.0),
                0.0,
            ));
            child.0.draw(batch);
            batch.borrow_mut().affine_transform = current_transform;
            x_offset += child.0.size().x();
        }
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(Row {
            children: self.children.to_vec(),
            alignment: self.alignment,
        })
    }
}

// pub struct Align {
//     child: Box<dyn VectarineWidget>,
//     alignment_y: Alignment,
//     alignment_x: Alignment,
//     size: Vec2,
// }

// MARK: Lua API

pub fn setup_ui_api(
    lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    _resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let ui_module = lua.create_table()?;

    lua.register_userdata_type::<WidgetBox>(|registry| {
        registry.add_method("size", |_, widget, (): ()| Ok(widget.0.size()));
        registry.add_method("draw", {
            let batch = batch.clone();
            move |_, widget, (): ()| {
                widget.0.draw(&batch);
                Ok(())
            }
        });
    })?;

    ui_module.raw_set(
        "widget",
        lua.create_function(|_lua, (size, draw_fn): (Vec2, mlua::Function)| {
            let widget = WidgetBox(Box::new(GenericWidget { size, draw_fn }));
            Ok(widget)
        })?,
    )?;

    ui_module.raw_set(
        "column",
        lua.create_function(|_lua, (children,): (Vec<WidgetBox>,)| {
            let column = WidgetBox(Box::new(Column {
                children,
                alignment: Alignment::Start,
            }));
            Ok(column)
        })?,
    )?;

    ui_module.raw_set(
        "row",
        lua.create_function(|_lua, (children,): (Vec<WidgetBox>,)| {
            let row = WidgetBox(Box::new(Row {
                children,
                alignment: Alignment::Start,
            }));
            Ok(row)
        })?,
    )?;

    Ok(ui_module)
}
