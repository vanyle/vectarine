use std::{cell::RefCell, rc::Rc};

use mlua::{FromLua, IntoLua, UserDataMethods};

use crate::{
    game_resource::{self, ResourceId, audio_resource::AudioResource},
    io,
    lua_env::lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct AudioResourceId(ResourceId);

impl ResourceIdWrapper for AudioResourceId {
    fn to_resource_id(&self) -> ResourceId {
        self.0
    }
    fn from_id(id: ResourceId) -> Self {
        Self(id)
    }
}

impl IntoLua for AudioResourceId {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.create_any_userdata(self).map(mlua::Value::UserData)
    }
}

impl FromLua for AudioResourceId {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "AudioResource".to_string(),
                message: Some("Expected AudioResource userdata".to_string()),
            }),
        }
    }
}

pub fn setup_audio_api(
    lua: &Rc<mlua::Lua>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let audio_module = lua.create_table()?;

    lua.register_userdata_type::<AudioResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);

        registry.add_method("play", {
            let resources = Rc::clone(resources);
            move |_lua, audio_resource_id, (is_loop, fade_in): (Option<bool>, Option<f32>)| {
                let audio_res = resources.get_by_id::<AudioResource>(audio_resource_id.0);
                let Ok(audio_res) = audio_res else {
                    return Ok(());
                };
                let is_loop = is_loop.unwrap_or(false);
                audio_res.play(is_loop, fade_in.map(|f| f as i32));
                Ok(())
            }
        });
        registry.add_method("pause", {
            let resources = Rc::clone(resources);
            move |_lua, audio_resource_id, (_fade_out,): (Option<f32>,)| {
                let audio_res = resources.get_by_id::<AudioResource>(audio_resource_id.0);
                let Ok(audio_res) = audio_res else {
                    return Ok(());
                };
                audio_res.pause(); // fade_out not available yet.
                Ok(())
            }
        });
        registry.add_method("resume", {
            let resources = Rc::clone(resources);
            move |_lua, audio_resource_id, (_fade_out,): (Option<f32>,)| {
                let audio_res = resources.get_by_id::<AudioResource>(audio_resource_id.0);
                let Ok(audio_res) = audio_res else {
                    return Ok(());
                };
                audio_res.resume(); // fade_out not available yet.
                Ok(())
            }
        });
        registry.add_method("setVolume", {
            let resources = Rc::clone(resources);
            move |_lua, audio_resource_id, (volume,): (f32,)| {
                let audio_res = resources.get_by_id::<AudioResource>(audio_resource_id.0);
                let Ok(audio_res) = audio_res else {
                    return Ok(());
                };
                let _ = audio_res.set_volume(volume);
                Ok(())
            }
        });
        registry.add_method("getVolume", {
            let resources = Rc::clone(resources);
            move |_lua, audio_resource_id, (): ()| {
                let audio_res = resources.get_by_id::<AudioResource>(audio_resource_id.0);
                let Ok(audio_res) = audio_res else {
                    return Ok(0.0);
                };
                Ok(audio_res.get_volume())
            }
        });
    })?;

    Ok(audio_module)
}
