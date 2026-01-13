use std::{cell::RefCell, rc::Rc};

use vectarine_plugin_sdk::mlua::{AnyUserData, FromLua, IntoLua, UserDataMethods, Value};

use crate::{
    game_resource::{
        self, ResourceId, Status,
        font_resource::{self, FontRenderingData, FontResource},
    },
    graphics::batchdraw,
    io,
    lua_env::{
        lua_coord::{ScreenVec, get_pos_as_vec2},
        lua_vec4::{BLACK, Vec4},
    },
};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct FontResourceId(Option<ResourceId>);

impl FontResourceId {
    pub fn from_id(id: ResourceId) -> Self {
        FontResourceId(Some(id))
    }
}

impl IntoLua for FontResourceId {
    fn into_lua(self, lua: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Value> {
        lua.create_any_userdata(self).map(vectarine_plugin_sdk::mlua::Value::UserData)
    }
}

impl FromLua for FontResourceId {
    fn from_lua(value: vectarine_plugin_sdk::mlua::Value, _: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<Self> {
        match value {
            vectarine_plugin_sdk::mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(vectarine_plugin_sdk::mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "FontResourceId".into(),
                message: Some("Expected FontResourceId userdata".into()),
            }),
        }
    }
}

pub fn setup_text_api(
    lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let text_module = lua.create_table()?;

    let default_font_handle = FontResourceId(None);

    lua.register_userdata_type::<FontResourceId>(|registry| {
        registry.add_meta_function(vectarine_plugin_sdk::mlua::MetaMethod::ToString, |_lua, (id,): (FontResourceId,)| {
            if let Some(id) = id.0{
                Ok(format!("Resource({})", id.get_id()))
            }else{
                Ok("FontResource(default)".to_string())
            }
        });
        registry.add_meta_function(vectarine_plugin_sdk::mlua::MetaMethod::Eq, |_lua, (id1, id2): (FontResourceId, FontResourceId)| {
            Ok(id1 == id2)
        });
        registry.add_method("getStatus", {
            let resources = resources.clone();
            move |_, id: &FontResourceId, (): ()| {
                if let Some(id) = id.0{
                    let status = resources.get_holder_by_id(id).get_status();
                    Ok(status.to_string())
                }else{
                    Ok(Status::Loaded.to_string())
                }
            }
        });
        registry.add_method("isReady", {
            let resources = resources.clone();
            move |_, id: &FontResourceId, (): ()|{
                if let Some(id) = id.0 {
                    Ok(resources.get_holder_by_id(id).is_loaded())
                }else{
                    Ok(true)
                }
            }
        });

        registry.add_method("getId", move |_, id: &FontResourceId, (): ()| {
            if let Some(id) = id.0{
                Ok(id.get_id() as i64)
            }else{
                Ok(-1)
            }
        });

        registry.add_method("drawText", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_, font, (text, mpos, lua_size, color): (String, AnyUserData, Value, Option<Vec4>)| {
                let font_size = value_to_text_size(&lua_size)?;
                let pos = get_pos_as_vec2(mpos)?;
                let color = color.unwrap_or(BLACK);
                let draw_with_renderer = |font_renderer: &mut FontRenderingData|{
                    {
                        font_renderer.enrich_atlas(batch.borrow().drawing_target.gl(), &text);
                    }
                    batch
                        .borrow_mut()
                        .draw_text(pos.x(), pos.y(), &text, color.0, font_size, font_renderer);
                };

                if let Some(font_id) = font.0 {
                    let font_resource = resources.get_by_id::<FontResource>(font_id);
                    let Ok(font_resource) = font_resource else {
                        return Ok(());
                    };
                    let mut font_resource = font_resource.font_rendering.borrow_mut();
                    let Some(font_resource) = font_resource.as_mut() else {
                        return Ok(());
                    };
                    draw_with_renderer(font_resource);
                }else{
                    let gl = batch.borrow().drawing_target.gl().clone();
                    font_resource::use_default_font(&gl, draw_with_renderer);
                };
                Ok(())
            }
        });
        registry.add_method("measureText", {
            let resources = resources.clone();
            let env_state = env_state.clone();
            let batch = batch.clone();
            move |lua, font_resource_id, (text, lua_font_size): (String, Value)| {
                let font_size = value_to_text_size(&lua_font_size)?;
                let make_failure_result = ||{
                    let result = match lua.create_table(){
                        Ok(result) => result,
                        Err(e) => return Err(e)
                    };
                    let _ = result.set("width", 0.0);
                    let _ = result.set("height", 0.0);
                    let _ = result.set("bearingY", 0.0);
                    Ok(result)
                };
                let make_measurement = |font_renderer: &mut FontRenderingData|{
                    let env_state = env_state.borrow();
                    let ratio = env_state.window_width as f32 / env_state.window_height as f32;
                    let (width, height, max_ascent) =
                        font_renderer.measure_text(&text, font_size, ratio);
                    let result = match lua.create_table(){
                        Ok(result) => result,
                        Err(e) => return Err(e)
                    };
                    let _ = result.set("width", width);
                    let _ = result.set("height", height);
                    let _ = result.set("bearingY", max_ascent);
                    Ok(result)
                };

                if let Some(font_id) = font_resource_id.0 {
                    let font_resource = resources.get_by_id::<FontResource>(font_id);
                    let Ok(font_resource) = font_resource else {
                        return make_failure_result();
                    };
                    let mut font_resource = font_resource.font_rendering.borrow_mut();
                    let Some(font_resource) = font_resource.as_mut() else {
                        return make_failure_result();
                    };
                    make_measurement(font_resource)
                }else{
                    font_resource::use_default_font(batch.borrow().drawing_target.gl(), make_measurement)
                }
            }
        });
    })?;

    text_module.set("font", default_font_handle)?;

    Ok(text_module)
}

fn value_to_text_size(value: &vectarine_plugin_sdk::mlua::Value) -> vectarine_plugin_sdk::mlua::Result<f32> {
    match value {
        vectarine_plugin_sdk::mlua::Value::Number(n) => Ok(*n as f32),
        vectarine_plugin_sdk::mlua::Value::UserData(user_data) => {
            let screen_vec = user_data.borrow::<ScreenVec>();
            let Ok(vec) = screen_vec else {
                return Err(vectarine_plugin_sdk::mlua::Error::ToLuaConversionError {
                    from: value.type_name().to_string(),
                    to: "number",
                    message: Some("Unable to convert the text size to a number".to_string()),
                });
            };
            Ok(vec.as_vec2().y())
        }
        vectarine_plugin_sdk::mlua::Value::Nil => Ok(0.05),
        _ => Err(vectarine_plugin_sdk::mlua::Error::ToLuaConversionError {
            from: value.type_name().to_string(),
            to: "number",
            message: Some("Unable to convert the text size to a number".to_string()),
        }),
    }
}
