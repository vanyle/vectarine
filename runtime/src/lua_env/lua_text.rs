use std::{cell::RefCell, rc::Rc};

use mlua::{AnyUserData, FromLua, IntoLua, Table, UserDataMethods};

use crate::{
    game_resource::{self, ResourceId, font_resource::FontResource},
    graphics::batchdraw,
    io,
    lua_env::{
        lua_coord::get_pos_as_vec2,
        lua_graphics::table_to_color,
        lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
    },
};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct FontResourceId(ResourceId);

impl ResourceIdWrapper for FontResourceId {
    fn to_resource_id(&self) -> ResourceId {
        self.0
    }
    fn from_id(id: ResourceId) -> Self {
        Self(id)
    }
}

impl IntoLua for FontResourceId {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.create_any_userdata(self).map(mlua::Value::UserData)
    }
}

impl FromLua for FontResourceId {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "FontResource".to_string(),
                message: Some("Expected FontResource userdata".to_string()),
            }),
        }
    }
}

pub fn setup_text_api(
    lua: &Rc<mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let text_module = lua.create_table()?;

    lua.register_userdata_type::<FontResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);

        registry.add_method("drawText", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_, font, (text, mpos, size, color): (String, AnyUserData, f32, Table)| {
                let pos = get_pos_as_vec2(mpos)?;
                let color = table_to_color(color);
                let font_resource = resources.get_by_id::<FontResource>(font.0);
                let Ok(font_resource) = font_resource else {
                    return Ok(());
                };
                let font_resource = font_resource.font_rendering.borrow();
                let Some(font_resource) = font_resource.as_ref() else {
                    return Ok(());
                };
                batch
                    .borrow_mut()
                    .draw_text(pos.x, pos.y, &text, color, size, font_resource);
                Ok(())
            }
        });
        registry.add_method("measureText", {
            let resources = resources.clone();
            let env_state = env_state.clone();
            move |lua, font_resource_id, (text, font_size): (String, f32)| {
                let font_resource = resources.get_by_id::<FontResource>(font_resource_id.0);
                let result = lua.create_table().unwrap();
                let Ok(font_resource) = font_resource else {
                    let _ = result.set("width", 0.0);
                    let _ = result.set("height", 0.0);
                    let _ = result.set("bearingY", 0.0);
                    return Ok(result);
                };
                let env_state = env_state.borrow();
                let ratio = env_state.window_width as f32 / env_state.window_height as f32;
                let (width, height, max_ascent) =
                    font_resource.measure_text(&text, font_size, ratio);
                let _ = result.set("width", width);
                let _ = result.set("height", height);
                let _ = result.set("bearingY", max_ascent);
                Ok(result)
            }
        });
    })?;

    Ok(text_module)
}
