use std::{cell::RefCell, ops, rc::Rc};

use mlua::{AnyUserData, FromLua, UserData};

use crate::{
    io::IoEnvState,
    lua_env::{add_fn_to_table, lua_io::get_env_state, lua_vec2::Vec2},
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
            |_, (this, other): (ScreenVec, AnyUserData)| {
                let other = other.borrow::<ScreenVec>()?;
                Ok(ScreenVec(this.0 + other.0))
            },
        );
    }
}

impl UserData for ScreenPosition {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("gl", |_, this, ()| Ok(this.as_vec2()));
        methods.add_method("px", |lua, this, ()| {
            let env_state = get_env_state(lua);
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(this.as_px(screen_width, screen_height))
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

        methods.add_meta_method(
            mlua::MetaMethod::Sub,
            |_, this: &ScreenPosition, (other,): (AnyUserData,)| {
                let other = other.borrow::<ScreenPosition>().unwrap();
                Ok(*this - *other)
            },
        );

        methods.add_meta_method(mlua::MetaMethod::ToString, |_, pos, _any: mlua::Value| {
            Ok(format!("ScreenPosition({}, {})", pos.0.x, pos.0.y))
        });
    }
}

pub fn setup_coords_api(
    lua: &Rc<mlua::Lua>,
    env_state: &Rc<RefCell<IoEnvState>>,
) -> mlua::Result<mlua::Table> {
    let coords_module = lua.create_table()?;

    add_fn_to_table(lua, &coords_module, "px", {
        let env_state = env_state.clone();
        move |_, (x, y): (f32, f32)| {
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(ScreenPosition::from_px(
                Vec2::new(x, y),
                screen_width,
                screen_height,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "pxDelta", {
        let env_state = env_state.clone();
        move |_, (x, y): (f32, f32)| {
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(ScreenVec::from_px(
                Vec2::new(x, y),
                screen_width,
                screen_height,
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
        let env_state = env_state.clone();
        move |_, (x, y): (f32, f32)| {
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(ScreenPosition::from_vw(
                Vec2::new(x, y),
                screen_width,
                screen_height,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "vwDelta", {
        let env_state = env_state.clone();
        move |_, (x, y): (f32, f32)| {
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(ScreenVec(Vec2::new(
                x * 2.0 / 100.0,
                y * 2.0 / 100.0 * screen_width / screen_height,
            )))
        }
    });

    add_fn_to_table(lua, &coords_module, "vh", {
        let env_state = env_state.clone();
        move |_, (x, y): (f32, f32)| {
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(ScreenPosition::from_vh(
                Vec2::new(x, y),
                screen_width,
                screen_height,
            ))
        }
    });

    add_fn_to_table(lua, &coords_module, "vhDelta", {
        let env_state = env_state.clone();
        move |_, (x, y): (f32, f32)| {
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(ScreenVec(Vec2::new(
                x * 2.0 / 100.0 * screen_height / screen_width,
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
