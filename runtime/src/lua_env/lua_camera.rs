use std::{cell::RefCell, rc::Rc};

use mlua::{UserDataFields, UserDataMethods};

use crate::{io::IoEnvState, lua_env::lua_fastlist::FastList, lua_env::lua_vec2::Vec2};

#[derive(Clone, Debug)]
pub struct Camera2 {
    pub position: Vec2,
    pub rotation: f32,
    pub zoom: f32,
}

impl mlua::IntoLua for Camera2 {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.create_any_userdata(self).map(mlua::Value::UserData)
    }
}

impl mlua::FromLua for Camera2 {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Camera2".to_string(),
                message: Some("Expected Camera2 userdata".to_string()),
            }),
        }
    }
}

impl Camera2 {
    pub fn new() -> Self {
        Self {
            position: Vec2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        }
    }

    /// Transform a world position to screen position (OpenGL coordinates)
    /// This preserves aspect ratio: a square in world space remains square on screen (in pixels).
    /// Centers (0,0) in world to (0,0) in screen.
    pub fn world_to_screen(&self, point: Vec2, window_size: Vec2) -> Vec2 {
        let aspect = window_size.x() / window_size.y();

        let relative = point - self.position;
        let rotated = relative.rotated(-self.rotation);
        let zoomed = rotated * self.zoom;

        Vec2::new(zoomed.x(), zoomed.y() * aspect)
    }

    /// Transform a screen position (OpenGL coordinates) to world position
    pub fn screen_to_world(&self, point: Vec2, window_size: Vec2) -> Vec2 {
        let aspect = window_size.x() / window_size.y();

        let unscaled = point.with_y(point.y() / aspect);
        let unzoomed = unscaled / self.zoom;
        let unrotated = unzoomed.rotated(self.rotation);

        unrotated + self.position
    }

    /// Check if a point is visible on the screen
    pub fn is_visible(&self, point: Vec2, window_size: Vec2) -> bool {
        let p = self.world_to_screen(point, window_size);
        p.x().abs() <= 1.0 && p.y().abs() <= 1.0
    }
}

impl Default for Camera2 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn setup_camera_api(
    lua: &Rc<mlua::Lua>,
    env_state: &Rc<RefCell<IoEnvState>>,
) -> mlua::Result<mlua::Table> {
    lua.register_userdata_type::<Camera2>(|registry| {
        registry.add_field_method_get("position", |_, camera| Ok(camera.position));
        registry.add_field_method_set("position", |_, camera, position: Vec2| {
            camera.position = position;
            Ok(())
        });

        registry.add_field_method_get("rotation", |_, camera| Ok(camera.rotation));
        registry.add_field_method_set("rotation", |_, camera, rotation: f32| {
            camera.rotation = rotation;
            Ok(())
        });

        registry.add_field_method_get("zoom", |_, camera| Ok(camera.zoom));
        registry.add_field_method_set("zoom", |_, camera, zoom: f32| {
            camera.zoom = zoom;
            Ok(())
        });

        registry.add_method("screen", {
            let env_state = env_state.clone();
            move |_, camera, point: Vec2| {
                let state = env_state.borrow();
                let window_size = Vec2::new(
                    state.window_width as f32 / state.px_ratio_x,
                    state.window_height as f32 / state.px_ratio_y,
                );
                Ok(camera.world_to_screen(point, window_size))
            }
        });

        registry.add_method("world", {
            let env_state = env_state.clone();
            move |_, camera, point: Vec2| {
                let state = env_state.borrow();
                let window_size = Vec2::new(
                    state.window_width as f32 / state.px_ratio_x,
                    state.window_height as f32 / state.px_ratio_y,
                );
                Ok(camera.screen_to_world(point, window_size))
            }
        });

        registry.add_method("isVisible", {
            let env_state = env_state.clone();
            move |_, camera, point: Vec2| {
                let state = env_state.borrow();
                let window_size = Vec2::new(
                    state.window_width as f32 / state.px_ratio_x,
                    state.window_height as f32 / state.px_ratio_y,
                );
                Ok(camera.is_visible(point, window_size))
            }
        });

        registry.add_method("screenFastlist", {
            let env_state = env_state.clone();
            move |_, camera, points: FastList| {
                let state = env_state.borrow();
                let window_size = Vec2::new(
                    state.window_width as f32 / state.px_ratio_x,
                    state.window_height as f32 / state.px_ratio_y,
                );
                let aspect = window_size.x() / window_size.y();
                let zoom = camera.zoom;
                let rot_vec = Vec2::from_angle(-camera.rotation);
                let pos = camera.position;

                let data = points
                    .data
                    .iter()
                    .map(|p| {
                        let rel = *p - pos;
                        let rotated = rel.cmul(rot_vec);
                        let zoomed = rotated * zoom;
                        Vec2::new(zoomed.x(), zoomed.y() * aspect)
                    })
                    .collect();

                Ok(FastList::from_vec(data))
            }
        });

        registry.add_method("worldFastlist", {
            let env_state = env_state.clone();
            move |_, camera, points: FastList| {
                let state = env_state.borrow();
                let window_size = Vec2::new(
                    state.window_width as f32 / state.px_ratio_x,
                    state.window_height as f32 / state.px_ratio_y,
                );
                let aspect = window_size.x() / window_size.y();
                let zoom = camera.zoom;
                let rot_vec = Vec2::from_angle(camera.rotation);
                let pos = camera.position;

                let data = points
                    .data
                    .iter()
                    .map(|p| {
                        let unscaled = p.with_y(p.y() / aspect);
                        let unzoomed = unscaled / zoom;
                        let unrotated = unzoomed.cmul(rot_vec);
                        unrotated + pos
                    })
                    .collect();

                Ok(FastList::from_vec(data))
            }
        });

        registry.add_method_mut("moveTowards", |_, camera, (point, amount): (Vec2, f32)| {
            camera.position = camera.position + (point - camera.position) * amount;
            Ok(())
        });

        registry.add_method_mut("rotateTowards", |_, camera, (angle, amount): (f32, f32)| {
            camera.rotation = camera.rotation + (angle - camera.rotation) * amount;
            Ok(())
        });

        registry.add_method_mut("zoomTowards", |_, camera, (zoom, amount): (f32, f32)| {
            camera.zoom = camera.zoom + (zoom - camera.zoom) * amount;
            Ok(())
        });
    })?;

    let camera_module = lua.create_table()?;

    camera_module.set("new", lua.create_function(|_, ()| Ok(Camera2::new()))?)?;

    Ok(camera_module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_screen() {
        let camera = Camera2::new();
        let win = Vec2::new(800.0, 600.0);
        let aspect = 800.0 / 600.0;
        let p0 = Vec2::new(0.0, 0.0);
        let s0 = camera.world_to_screen(p0, win);
        assert!((s0.x() - 0.0).abs() < 1e-6);
        assert!((s0.y() - 0.0).abs() < 1e-6);

        // Aspect ratio preserved: (1,1) maps to (1, aspect)
        let p1 = Vec2::new(1.0, 1.0);
        let s1 = camera.world_to_screen(p1, win);
        assert!((s1.x() - 1.0).abs() < 1e-6);
        assert!((s1.y() - aspect).abs() < 1e-6);
    }

    #[test]
    fn zoom_scaling() {
        let mut camera = Camera2::new();
        camera.zoom = 2.0;
        let win = Vec2::new(100.0, 100.0); // Aspect 1.0
        let p = Vec2::new(1.0, 0.0);
        let s = camera.world_to_screen(p, win);
        assert!((s.x() - 2.0).abs() < 1e-6);
        assert!((s.y() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn round_trip() {
        let mut camera = Camera2::new();
        camera.position = Vec2::new(10.0, -5.0);
        camera.rotation = 1.5;
        camera.zoom = 0.5;

        let win = Vec2::new(800.0, 600.0);
        let p = Vec2::new(123.0, 456.0);

        let s = camera.world_to_screen(p, win);
        let p2 = camera.screen_to_world(s, win);

        assert!((p.x() - p2.x()).abs() < 1e-5);
        assert!((p.y() - p2.y()).abs() < 1e-5);
    }
}
