use std::{ops, rc::Rc, sync::Arc};

use glow::Context;
use mlua::{AnyUserData, FromLua, IntoLua, UserDataMethods};

use crate::{
    auto_impl_lua_copy,
    graphics::glframebuffer::{Viewport, get_viewport},
    lua_env::{add_fn_to_table, lua_vec2::Vec2},
};

// MARK: Type Def

/// Represents a point on the screen
/// This is internally stored in OpenGL coordinates
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ScreenPosition(Vec2);
auto_impl_lua_copy!(ScreenPosition, ScreenPosition);

/// Represents a direction on the screen
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ScreenVec(Vec2);
auto_impl_lua_copy!(ScreenVec, ScreenVec);

impl ScreenPosition {
    pub fn as_vec2(self) -> Vec2 {
        self.0
    }
    pub fn as_px(self, screen_width: f32, screen_height: f32) -> Vec2 {
        Vec2::new(
            (self.0.x() + 1.0) * 0.5 * screen_width,
            (1.0 - self.0.y()) * 0.5 * screen_height,
        )
    }
    pub fn from_opengl(v: Vec2) -> Self {
        ScreenPosition(v)
    }
    pub fn from_px(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + (v.x() * 2.0 / screen_width),
            1.0 - (v.y() * 2.0 / screen_height),
        ))
    }
    pub fn from_vw(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x() * 2.0 / 100.0,
            -1.0 + v.y() * 2.0 / 100.0 * screen_width / screen_height,
        ))
    }
    pub fn from_vh(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x() * 2.0 / 100.0 * screen_height / screen_width,
            -1.0 + v.y() * 2.0 / 100.0,
        ))
    }
}

impl ScreenVec {
    pub fn as_vec2(self) -> Vec2 {
        self.0
    }
    pub fn scale(self, k: f32) -> Self {
        ScreenVec(self.0.scale(k))
    }
    pub fn from_px(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenVec(Vec2::new(
            v.x() * 2.0 / screen_width,
            -v.y() * 2.0 / screen_height,
        ))
    }
    pub fn as_px(self, screen_width: f32, screen_height: f32) -> Vec2 {
        Vec2::new(
            self.0.x() * screen_width * 0.5,
            -self.0.y() * screen_height * 0.5,
        )
    }
}

impl ops::Sub for ScreenPosition {
    type Output = ScreenVec;

    fn sub(self, rhs: Self) -> Self::Output {
        ScreenVec(self.0 - rhs.0)
    }
}

impl ops::Add<ScreenVec> for ScreenPosition {
    type Output = ScreenPosition;

    fn add(self, rhs: ScreenVec) -> Self::Output {
        ScreenPosition(self.0 + rhs.0)
    }
}

pub fn setup_coords_api(lua: &Rc<mlua::Lua>, gl: &Arc<Context>) -> mlua::Result<mlua::Table> {
    let coords_module = lua.create_table()?;

    lua.register_userdata_type::<ScreenVec>(|registry| {
        let gl = gl.clone();
        registry.add_meta_function(
            mlua::MetaMethod::Add,
            |_, (this, other): (ScreenVec, ScreenVec)| Ok(ScreenVec(this.0 + other.0)),
        );
        registry.add_meta_function(
            mlua::MetaMethod::Sub,
            |_, (this, other): (ScreenVec, ScreenVec)| Ok(ScreenVec(this.0 - other.0)),
        );
        registry.add_method("gl", |_, this, ()| Ok(this.as_vec2()));
        registry.add_method("px", move |_lua, this, (screen_size,): (Option<Vec2>,)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(this.as_px(viewport.width as f32, viewport.height as f32))
        });
        registry.add_method("scale", |_, this, (k,): (f32,)| Ok(this.scale(k)));

        registry.add_meta_method(mlua::MetaMethod::ToString, |_, pos, _any: mlua::Value| {
            Ok(format!("ScreenVec({:.4}, {:.4})", pos.0.x(), pos.0.y()))
        });
    })?;

    lua.register_userdata_type::<ScreenPosition>(|registry| {
        let gl = gl.clone();
        registry.add_method("gl", |_, this, ()| Ok(this.as_vec2()));
        registry.add_method("px", move |_lua, this, (screen_size,): (Option<Vec2>,)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(this.as_px(viewport.width as f32, viewport.height as f32))
        });

        registry.add_meta_function(
            mlua::MetaMethod::Add,
            |_lua, (this, other): (ScreenPosition, AnyUserData)| {
                let is_other_screen_pos = other.is::<ScreenPosition>();
                if is_other_screen_pos {
                    return Err(mlua::Error::RuntimeError(
                        "Cannot add two ScreenPosition together. Did you mean to use a ScreenDelta?"
                        .to_string(),
                    ));
                }
                let other = other.borrow::<ScreenVec>()?;
                Ok(this + *other)
            },
        );

        registry.add_meta_function(
            mlua::MetaMethod::Sub,
            |lua, (this, other): (ScreenPosition, AnyUserData)| {
                let as_screen_pos = other.borrow::<ScreenPosition>();
                if let Ok(as_screen_pos) = as_screen_pos {
                    return ScreenVec(this.0 - as_screen_pos.0).into_lua(lua);
                }
                let as_screen_vec = other.borrow::<ScreenVec>();
                if let Ok(as_screen_vec) = as_screen_vec {
                    return (ScreenPosition(this.0 - as_screen_vec.0)).into_lua(lua);
                }
                let as_vec = other.borrow::<Vec2>()?;
                (ScreenPosition(this.0 - *as_vec)).into_lua(lua)
            },
        );

        registry.add_meta_method(mlua::MetaMethod::ToString, |_, pos, _any: mlua::Value| {
            Ok(format!(
                "ScreenPosition({:.4}, {:.4})",
                pos.0.x(),
                pos.0.y()
            ))
        });
    })?;

    add_fn_to_table(lua, &coords_module, "px", {
        let gl = gl.clone();
        move |_lua, (v, screen_size): (Vec2, Option<Vec2>)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(ScreenPosition::from_px(
                v,
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "pxVec", {
        let gl = gl.clone();
        move |_lua, (v, screen_size): (Vec2, Option<Vec2>)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(ScreenVec::from_px(
                v,
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "gl", move |_, (v,): (Vec2,)| {
        Ok(ScreenPosition::from_opengl(v))
    });

    add_fn_to_table(lua, &coords_module, "glVec", move |_, (v,): (Vec2,)| {
        Ok(ScreenVec(v))
    });

    add_fn_to_table(lua, &coords_module, "vw", {
        let gl = gl.clone();
        move |_lua, (v, screen_size): (Vec2, Option<Vec2>)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(ScreenPosition::from_vw(
                v,
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "vwVec", {
        let gl = gl.clone();
        move |_lua, (v, screen_size): (Vec2, Option<Vec2>)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(ScreenVec(Vec2::new(
                v.x() * 2.0 / 100.0,
                v.y() * 2.0 / 100.0 * viewport.width as f32 / viewport.height as f32,
            )))
        }
    });

    add_fn_to_table(lua, &coords_module, "vh", {
        let gl = gl.clone();
        move |_lua, (v, screen_size): (Vec2, Option<Vec2>)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(ScreenPosition::from_vh(
                v,
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "vhVec", {
        let gl = gl.clone();
        move |_lua, (v, screen_size): (Vec2, Option<Vec2>)| {
            let viewport = if let Some(screen_size) = screen_size {
                Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
            } else {
                get_viewport(&gl)
            };
            Ok(ScreenVec(Vec2::new(
                v.x() * 2.0 / 100.0 * viewport.height as f32 / viewport.width as f32,
                v.y() * 2.0 / 100.0,
            )))
        }
    });

    coords_module.set("CENTER", ScreenPosition::from_opengl(Vec2::zero()))?;
    coords_module.set(
        "TOP_LEFT",
        ScreenPosition::from_opengl(Vec2::new(-1.0, 1.0)),
    )?;
    coords_module.set(
        "TOP_RIGHT",
        ScreenPosition::from_opengl(Vec2::new(1.0, 1.0)),
    )?;
    coords_module.set(
        "BOTTOM_LEFT",
        ScreenPosition::from_opengl(Vec2::new(-1.0, -1.0)),
    )?;
    coords_module.set(
        "BOTTOM_RIGHT",
        ScreenPosition::from_opengl(Vec2::new(1.0, -1.0)),
    )?;

    Ok(coords_module)
}

pub fn get_pos_as_vec2(userdata: mlua::AnyUserData) -> mlua::Result<Vec2> {
    let pos = userdata.borrow::<ScreenPosition>();
    let err: mlua::Error = match pos {
        Ok(pos) => return Ok(pos.as_vec2()),
        Err(err) => err,
    };
    if matches!(err, mlua::Error::UserDataTypeMismatch) {
        let vec = userdata.borrow::<Vec2>()?;
        Ok(*vec)
    } else {
        Err(err)
    }
}

pub fn get_size_as_vec2(userdata: mlua::AnyUserData) -> mlua::Result<Vec2> {
    let size = userdata.borrow::<ScreenVec>();
    let err: mlua::Error = match size {
        Ok(size) => return Ok(size.as_vec2()),
        Err(err) => err,
    };
    if matches!(err, mlua::Error::UserDataTypeMismatch) {
        let vec = userdata.borrow::<Vec2>()?;
        Ok(*vec)
    } else {
        Err(err)
    }
}
