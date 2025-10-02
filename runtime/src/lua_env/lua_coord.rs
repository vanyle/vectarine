use std::{ops, rc::Rc};

use mlua::{AnyUserData, FromLua, IntoLua, UserData};

use crate::{
    graphics::glframebuffer::get_viewport,
    lua_env::{add_fn_to_table, get_gl_handle, lua_vec2::Vec2},
};

// MARK: Type Def

/// Represents a point on the screen
/// This is internally stored in OpenGL coordinates
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ScreenPosition(Vec2);

/// Represents a direction on the screen
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ScreenVec(Vec2);

impl ScreenPosition {
    pub fn as_vec2(self) -> Vec2 {
        self.0
    }
    pub fn as_px(self, screen_width: f32, screen_height: f32) -> Vec2 {
        Vec2::new(
            (self.0.x + 1.0) * 0.5 * screen_width,
            (1.0 - self.0.y) * 0.5 * screen_height,
        )
    }
    pub fn from_opengl(v: Vec2) -> Self {
        ScreenPosition(v)
    }
    pub fn from_px(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + (v.x * 2.0 / screen_width),
            1.0 - (v.y * 2.0 / screen_height),
        ))
    }
    pub fn from_vw(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x * 2.0 / 100.0,
            -1.0 + v.y * 2.0 / 100.0 * screen_width / screen_height,
        ))
    }
    pub fn from_vh(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x * 2.0 / 100.0 * screen_height / screen_width,
            -1.0 + v.y * 2.0 / 100.0,
        ))
    }
}

impl ScreenVec {
    pub fn as_vec2(self) -> Vec2 {
        self.0
    }
    pub fn from_px(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenVec(Vec2::new(
            v.x * 2.0 / screen_width,
            -v.y * 2.0 / screen_height,
        ))
    }
    pub fn as_px(self, screen_width: f32, screen_height: f32) -> Vec2 {
        Vec2::new(
            self.0.x * screen_width * 0.5,
            -self.0.y * screen_height * 0.5,
        )
    }
}

impl FromLua for ScreenPosition {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "ScreenPosition".to_string(),
                message: Some("Expected ScreenPosition userdata".to_string()),
            }),
        }
    }
}

impl FromLua for ScreenVec {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "ScreenVec".to_string(),
                message: Some("Expected ScreenVec userdata".to_string()),
            }),
        }
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

// MARK: Lua

impl UserData for ScreenVec {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(
            mlua::MetaMethod::Add,
            |_, (this, other): (ScreenVec, ScreenVec)| Ok(ScreenVec(this.0 + other.0)),
        );
        methods.add_meta_function(
            mlua::MetaMethod::Sub,
            |_, (this, other): (ScreenVec, ScreenVec)| Ok(ScreenVec(this.0 - other.0)),
        );

        methods.add_method("gl", |_, this, ()| Ok(this.as_vec2()));
        methods.add_method("px", |lua, this, ()| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(this.as_px(viewport.width as f32, viewport.height as f32))
        });
    }
}

impl UserData for ScreenPosition {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("gl", |_, this, ()| Ok(this.as_vec2()));
        methods.add_method("px", |lua, this, ()| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(this.as_px(viewport.width as f32, viewport.height as f32))
        });

        methods.add_meta_function(
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

        methods.add_meta_function(
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

        methods.add_meta_method(mlua::MetaMethod::ToString, |_, pos, _any: mlua::Value| {
            Ok(format!("ScreenPosition({}, {})", pos.0.x, pos.0.y))
        });
    }
}

pub fn setup_coords_api(lua: &Rc<mlua::Lua>) -> mlua::Result<mlua::Table> {
    let coords_module = lua.create_table()?;

    add_fn_to_table(lua, &coords_module, "px", {
        move |lua, (x, y): (f32, f32)| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(ScreenPosition::from_px(
                Vec2::new(x, y),
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "pxDelta", {
        move |lua, (x, y): (f32, f32)| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(ScreenVec::from_px(
                Vec2::new(x, y),
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "gl", move |_, (x, y): (f32, f32)| {
        Ok(ScreenPosition::from_opengl(Vec2::new(x, y)))
    });

    add_fn_to_table(
        lua,
        &coords_module,
        "glDelta",
        move |_, (x, y): (f32, f32)| Ok(ScreenVec(Vec2::new(x, y))),
    );

    add_fn_to_table(lua, &coords_module, "vw", {
        move |lua, (x, y): (f32, f32)| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(ScreenPosition::from_vw(
                Vec2::new(x, y),
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "vwDelta", {
        move |lua, (x, y): (f32, f32)| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(ScreenVec(Vec2::new(
                x * 2.0 / 100.0,
                y * 2.0 / 100.0 * viewport.width as f32 / viewport.height as f32,
            )))
        }
    });

    add_fn_to_table(lua, &coords_module, "vh", {
        move |lua, (x, y): (f32, f32)| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(ScreenPosition::from_vh(
                Vec2::new(x, y),
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "vhDelta", {
        move |lua, (x, y): (f32, f32)| {
            let gl = get_gl_handle(lua);
            let viewport = get_viewport(&gl);
            Ok(ScreenVec(Vec2::new(
                x * 2.0 / 100.0 * viewport.height as f32 / viewport.width as f32,
                y * 2.0 / 100.0,
            )))
        }
    });

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
