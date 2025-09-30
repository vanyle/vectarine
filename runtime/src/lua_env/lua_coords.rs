use std::{cell::RefCell, ops, rc::Rc};

use mlua::UserData;

use crate::{
    io::IoEnvState,
    lua_env::{add_fn_to_table, get_internals, lua_io::get_env_state, lua_vec2::Vec2},
};

// MARK: Type Def

/// Represents a point on the screen
/// This is internally stored in OpenGL coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPosition(Vec2);

/// Represents a direction on the screen
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenDelta(Vec2);

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
            -1.0 + v.x * 2.0 / screen_width,
            1.0 - v.y * 2.0 / screen_height,
        ))
    }
    pub fn from_vw(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x * 2.0 / 100.0,
            1.0 - v.y * 2.0 / 100.0 * screen_width / screen_height,
        ))
    }
    pub fn from_vh(v: Vec2, screen_width: f32, screen_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x * 2.0 / 100.0 * screen_height / screen_width,
            1.0 - v.y * 2.0 / 100.0,
        ))
    }
}

impl ops::Sub for ScreenPosition {
    type Output = ScreenDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        ScreenDelta(self.0 - rhs.0)
    }
}

impl ops::Add<ScreenDelta> for ScreenPosition {
    type Output = ScreenPosition;

    fn add(self, rhs: ScreenDelta) -> Self::Output {
        ScreenPosition(self.0 + rhs.0)
    }
}

// MARK: Lua

impl UserData for ScreenPosition {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("gl", |_, this, ()| Ok(this.as_vec2()));
        methods.add_method("px", |lua, this, ()| {
            let env_state = get_env_state(lua);
            let screen_width = env_state.borrow().screen_width as f32;
            let screen_height = env_state.borrow().screen_height as f32;
            Ok(this.as_px(screen_width, screen_height))
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

    add_fn_to_table(lua, &coords_module, "gl", move |_, (x, y): (f32, f32)| {
        Ok(ScreenPosition::from_opengl(Vec2::new(x, y)))
    });

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

    Ok(coords_module)
}
