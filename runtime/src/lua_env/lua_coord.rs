use std::cell::RefCell;
use std::rc::Rc;
use std::{ops, sync::Arc};

use vectarine_plugin_sdk::glow::Context;
use vectarine_plugin_sdk::mlua::{self, AnyUserData, FromLua, IntoLua, UserDataMethods};

use crate::lua_env::lua_fastlist::value_to_f32;
use crate::{
    auto_impl_lua_copy,
    graphics::glframebuffer::{Viewport, get_viewport},
    lua_env::{IoEnvState, add_fn_to_table, lua_vec2::Vec2},
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
    #[inline(always)]
    pub fn as_vec2(self) -> Vec2 {
        self.0
    }
    #[inline(always)]
    pub fn as_px(self, window_width: f32, window_height: f32) -> Vec2 {
        Vec2::new(
            (self.0.x() + 1.0) * 0.5 * window_width,
            (1.0 - self.0.y()) * 0.5 * window_height,
        )
    }
    #[inline(always)]
    pub fn from_opengl(v: Vec2) -> Self {
        ScreenPosition(v)
    }
    #[inline(always)]
    pub fn from_px(v: Vec2, window_width: f32, window_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + (v.x() * 2.0 / window_width),
            1.0 - (v.y() * 2.0 / window_height),
        ))
    }
    #[inline(always)]
    pub fn from_vw(v: Vec2, window_width: f32, window_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x() * 2.0 / 100.0,
            -1.0 + v.y() * 2.0 / 100.0 * window_width / window_height,
        ))
    }
    #[inline(always)]
    pub fn from_vh(v: Vec2, window_width: f32, window_height: f32) -> Self {
        ScreenPosition(Vec2::new(
            -1.0 + v.x() * 2.0 / 100.0 * window_height / window_width,
            -1.0 + v.y() * 2.0 / 100.0,
        ))
    }
}

impl ScreenVec {
    #[inline(always)]
    pub fn as_vec2(self) -> Vec2 {
        self.0
    }
    #[inline(always)]
    pub fn scale(self, k: f32) -> Self {
        ScreenVec(self.0.scale(k))
    }
    #[inline(always)]
    pub fn from_px(v: Vec2, window_width: f32, window_height: f32) -> Self {
        ScreenVec(Vec2::new(
            v.x() * 2.0 / window_width,
            -v.y() * 2.0 / window_height,
        ))
    }
    #[inline(always)]
    pub fn as_px(self, window_width: f32, window_height: f32) -> Vec2 {
        Vec2::new(
            self.0.x() * window_width * 0.5,
            -self.0.y() * window_height * 0.5,
        )
    }
}

impl ops::Sub for ScreenPosition {
    type Output = ScreenVec;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        ScreenVec(self.0 - rhs.0)
    }
}

impl ops::Add<ScreenVec> for ScreenPosition {
    type Output = ScreenPosition;
    #[inline(always)]
    fn add(self, rhs: ScreenVec) -> Self::Output {
        ScreenPosition(self.0 + rhs.0)
    }
}

impl ops::Add for ScreenVec {
    type Output = ScreenVec;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        ScreenVec(self.0 + rhs.0)
    }
}

impl ops::Sub for ScreenVec {
    type Output = ScreenVec;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        ScreenVec(self.0 - rhs.0)
    }
}

pub fn extract_vec_for_args(
    lua: &mlua::Lua,
    v_or_x: mlua::Value,
    screensize_or_y: mlua::Value,
    screensize_or_nil: Option<Vec2>,
) -> (Option<Vec2>, Option<Vec2>) {
    if let Some(x) = value_to_f32(&v_or_x)
        && let Some(y) = value_to_f32(&screensize_or_y)
    {
        (Some(Vec2::new(x, y)), screensize_or_nil)
    } else if let Ok(v) = Vec2::from_lua(v_or_x.clone(), lua) {
        let maybe_screensize = Vec2::from_lua(screensize_or_y, lua).ok();
        (Some(v), maybe_screensize)
    } else {
        (None, None)
    }
}

pub fn setup_coords_api(
    lua: &mlua::Lua,
    gl: &Arc<Context>,
    env_state: &Rc<RefCell<IoEnvState>>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let coords_module = lua.create_table()?;

    lua.register_userdata_type::<ScreenVec>(|registry| {
        let gl = gl.clone();
        let env_state = env_state.clone();
        registry.add_meta_function(
            vectarine_plugin_sdk::mlua::MetaMethod::Add,
            #[inline(always)]
            |_, (this, other): (ScreenVec, ScreenVec)| Ok(ScreenVec(this.0 + other.0)),
        );
        registry.add_meta_function(
            vectarine_plugin_sdk::mlua::MetaMethod::Sub,
            #[inline(always)]
            |_, (this, other): (ScreenVec, ScreenVec)| Ok(ScreenVec(this.0 - other.0)),
        );
        registry.add_method(
            "gl",
            #[inline(always)]
            |_, this, ()| Ok(this.as_vec2()),
        );
        registry.add_method(
            "px",
            #[inline(always)]
            move |_lua, this, (framebuffer_size,): (Option<Vec2>,)| {
                let viewport = if let Some(screen_size) = framebuffer_size {
                    Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
                } else {
                    let drawable_viewport = get_viewport(&gl);
                    let env = env_state.borrow();
                    Viewport::from_size(
                        (drawable_viewport.width as f32 / env.px_ratio_x) as i32,
                        (drawable_viewport.height as f32 / env.px_ratio_y) as i32,
                    )
                };
                Ok(this.as_px(viewport.width as f32, viewport.height as f32))
            },
        );
        registry.add_method(
            "scale",
            #[inline(always)]
            |_, this, (k,): (f32,)| Ok(this.scale(k)),
        );

        registry.add_meta_function(
            vectarine_plugin_sdk::mlua::MetaMethod::Add,
            #[inline(always)]
            |_, (this, k): (ScreenVec, ScreenVec)| Ok(this + k),
        );

        registry.add_meta_function(
            vectarine_plugin_sdk::mlua::MetaMethod::Sub,
            #[inline(always)]
            |_, (this, k): (ScreenVec, ScreenVec)| Ok(this - k),
        );

        registry.add_meta_method(
            vectarine_plugin_sdk::mlua::MetaMethod::ToString,
            |_, pos, _any: vectarine_plugin_sdk::mlua::Value| {
                Ok(format!("ScreenVec({:.4}, {:.4})", pos.0.x(), pos.0.y()))
            },
        );
    })?;

    lua.register_userdata_type::<ScreenPosition>(|registry| {
        let gl = gl.clone();
        let env_state = env_state.clone();
        registry.add_method(
            "gl",
            #[inline(always)]
            |_, this, ()| Ok(this.as_vec2()),
        );
        registry.add_method(
            "px",
            #[inline(always)]
            move |_lua, this, (screen_size,): (Option<Vec2>,)| {
                let viewport = if let Some(screen_size) = screen_size {
                    Viewport::from_size(screen_size.x() as i32, screen_size.y() as i32)
                } else {
                    let drawable_viewport = get_viewport(&gl);
                    let env = env_state.borrow();
                    Viewport::from_size(
                        (drawable_viewport.width as f32 / env.px_ratio_x) as i32,
                        (drawable_viewport.height as f32 / env.px_ratio_y) as i32,
                    )
                };
                Ok(this.as_px(viewport.width as f32, viewport.height as f32))
            },
        );

        registry.add_meta_function(
            vectarine_plugin_sdk::mlua::MetaMethod::Add,
            #[inline(always)]
            |_lua, (this, other): (ScreenPosition, AnyUserData)| {
                let is_other_screen_pos = other.is::<ScreenPosition>();
                if is_other_screen_pos {
                    return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                        "Cannot add two ScreenPosition together. Did you mean to use a ScreenDelta?"
                        .to_string(),
                    ));
                }
                let other = other.borrow::<ScreenVec>()?;
                Ok(this + *other)
            },
        );

        registry.add_meta_function(
            vectarine_plugin_sdk::mlua::MetaMethod::Sub,
            #[inline(always)]
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

        registry.add_meta_method(
            vectarine_plugin_sdk::mlua::MetaMethod::ToString,
            |_, pos, _any: vectarine_plugin_sdk::mlua::Value| {
                Ok(format!(
                    "ScreenPosition({:.4}, {:.4})",
                    pos.0.x(),
                    pos.0.y()
                ))
            },
        );
    })?;

    add_fn_to_table(lua, &coords_module, "px", {
        let gl = gl.clone();
        #[inline(always)]
        move |lua,
              (v_or_x, screen_size_or_y, screen_size_or_nil): (
            mlua::Value,
            mlua::Value,
            Option<Vec2>,
        )| {
            let (v, screen_size) =
                extract_vec_for_args(lua, v_or_x, screen_size_or_y, screen_size_or_nil);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };

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
        let env_state = env_state.clone();
        #[inline(always)]
        move |lua,
              (v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil): (
            mlua::Value,
            mlua::Value,
            Option<Vec2>,
        )| {
            let (v, framebuffer_size) =
                extract_vec_for_args(lua, v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };
            let viewport = if let Some(framebuffer_size) = framebuffer_size {
                Viewport::from_size(framebuffer_size.x() as i32, framebuffer_size.y() as i32)
            } else {
                let drawable_viewport = get_viewport(&gl);
                let env = env_state.borrow();
                Viewport::from_size(
                    (drawable_viewport.width as f32 / env.px_ratio_x) as i32,
                    (drawable_viewport.height as f32 / env.px_ratio_y) as i32,
                )
            };
            Ok(ScreenVec::from_px(
                v,
                viewport.width as f32,
                viewport.height as f32,
            ))
        }
    });

    add_fn_to_table(
        lua,
        &coords_module,
        "gl",
        #[inline(always)]
        move |lua, (v_or_x, y_or_nil): (mlua::Value, mlua::Value)| {
            let (v, _) = extract_vec_for_args(lua, v_or_x, y_or_nil, None);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };
            Ok(ScreenPosition::from_opengl(v))
        },
    );

    add_fn_to_table(
        lua,
        &coords_module,
        "glVec",
        #[inline(always)]
        move |lua, (v_or_x, y_or_nil): (mlua::Value, mlua::Value)| {
            let (v, _) = extract_vec_for_args(lua, v_or_x, y_or_nil, None);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };
            Ok(ScreenVec(v))
        },
    );

    add_fn_to_table(lua, &coords_module, "vw", {
        let gl = gl.clone();
        #[inline(always)]
        move |lua,
              (v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil): (
            mlua::Value,
            mlua::Value,
            Option<Vec2>,
        )| {
            let (v, framebuffer_size) =
                extract_vec_for_args(lua, v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };
            let viewport = if let Some(framebuffer_size) = framebuffer_size {
                Viewport::from_size(framebuffer_size.x() as i32, framebuffer_size.y() as i32)
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
        #[inline(always)]
        move |lua,
              (v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil): (
            mlua::Value,
            mlua::Value,
            Option<Vec2>,
        )| {
            let (v, framebuffer_size) =
                extract_vec_for_args(lua, v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };
            let viewport = if let Some(framebuffer_size) = framebuffer_size {
                Viewport::from_size(framebuffer_size.x() as i32, framebuffer_size.y() as i32)
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
        #[inline(always)]
        move |lua,
              (v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil): (
            mlua::Value,
            mlua::Value,
            Option<Vec2>,
        )| {
            let (v, framebuffer_size) =
                extract_vec_for_args(lua, v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };
            let viewport = if let Some(framebuffer_size) = framebuffer_size {
                Viewport::from_size(framebuffer_size.x() as i32, framebuffer_size.y() as i32)
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
        #[inline(always)]
        move |lua,
              (v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil): (
            mlua::Value,
            mlua::Value,
            Option<Vec2>,
        )| {
            let (v, framebuffer_size) =
                extract_vec_for_args(lua, v_or_x, framebuffer_size_or_y, framebuffer_size_or_nil);
            let Some(v) = v else {
                return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                    "Invalid arguments".to_string(),
                ));
            };
            let viewport = if let Some(framebuffer_size) = framebuffer_size {
                Viewport::from_size(framebuffer_size.x() as i32, framebuffer_size.y() as i32)
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

pub fn get_pos_as_vec2(
    userdata: vectarine_plugin_sdk::mlua::AnyUserData,
) -> vectarine_plugin_sdk::mlua::Result<Vec2> {
    let pos = userdata.borrow::<ScreenPosition>();
    let err: vectarine_plugin_sdk::mlua::Error = match pos {
        Ok(pos) => return Ok(pos.as_vec2()),
        Err(err) => err,
    };
    if matches!(err, vectarine_plugin_sdk::mlua::Error::UserDataTypeMismatch) {
        let vec = userdata.borrow::<Vec2>()?;
        Ok(*vec)
    } else {
        Err(err)
    }
}

pub fn get_size_as_vec2(
    userdata: vectarine_plugin_sdk::mlua::AnyUserData,
) -> vectarine_plugin_sdk::mlua::Result<Vec2> {
    let size = userdata.borrow::<ScreenVec>();
    let err: vectarine_plugin_sdk::mlua::Error = match size {
        Ok(size) => return Ok(size.as_vec2()),
        Err(err) => err,
    };
    if matches!(err, vectarine_plugin_sdk::mlua::Error::UserDataTypeMismatch) {
        let vec = userdata.borrow::<Vec2>()?;
        Ok(*vec)
    } else {
        Err(err)
    }
}
