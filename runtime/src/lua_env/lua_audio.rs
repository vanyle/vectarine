use std::{cell::RefCell, rc::Rc};

use vectarine_plugin_sdk::mlua::{FromLua, IntoLua, UserDataMethods};

use crate::{
    game_resource::{self, ResourceId, audio_resource::AudioResource},
    io,
    lua_env::lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
    make_resource_lua_compatible,
};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct AudioResourceId(ResourceId);
make_resource_lua_compatible!(AudioResourceId);

pub fn setup_audio_api(
    lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
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
