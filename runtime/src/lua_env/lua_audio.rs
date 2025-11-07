use std::{cell::RefCell, rc::Rc};

use crate::{
    game_resource::{self, ResourceId, audio_resource::AudioResource},
    io,
    lua_env::add_fn_to_table,
};

pub fn setup_audio_api(
    lua: &Rc<mlua::Lua>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let audio_module = lua.create_table()?;

    add_fn_to_table(lua, &audio_module, "play", {
        let resources = Rc::clone(resources);
        move |_lua, (resource_id, is_loop, fade_in): (ResourceId, Option<bool>, Option<f32>)| {
            let audio_res = resources.get_by_id::<AudioResource>(resource_id);
            let Ok(audio_res) = audio_res else {
                return Ok(());
            };
            let is_loop = is_loop.unwrap_or(false);
            audio_res.play(is_loop, fade_in.map(|f| f as i32));
            Ok(())
        }
    });

    add_fn_to_table(lua, &audio_module, "pause", {
        let resources = Rc::clone(resources);
        move |_lua, (resource_id, _fade_out): (ResourceId, Option<f32>)| {
            let audio_res = resources.get_by_id::<AudioResource>(resource_id);
            let Ok(audio_res) = audio_res else {
                return Ok(());
            };
            audio_res.pause(); // fade_out not available yet.
            Ok(())
        }
    });

    add_fn_to_table(lua, &audio_module, "resume", {
        let resources = Rc::clone(resources);
        move |_lua, (resource_id, _fade_out): (ResourceId, Option<f32>)| {
            let audio_res = resources.get_by_id::<AudioResource>(resource_id);
            let Ok(audio_res) = audio_res else {
                return Ok(());
            };
            audio_res.resume(); // fade_out not available yet.
            Ok(())
        }
    });

    add_fn_to_table(lua, &audio_module, "setVolume", {
        let resources = Rc::clone(resources);
        move |_lua, (resource_id, volume): (ResourceId, f32)| {
            let audio_res = resources.get_by_id::<AudioResource>(resource_id);
            let Ok(audio_res) = audio_res else {
                return Ok(());
            };
            let _ = audio_res.set_volume(volume);
            Ok(())
        }
    });

    add_fn_to_table(lua, &audio_module, "getVolume", {
        let resources = Rc::clone(resources);
        move |_lua, (resource_id,): (ResourceId,)| {
            let audio_res = resources.get_by_id::<AudioResource>(resource_id);
            let Ok(audio_res) = audio_res else {
                return Ok(0.0);
            };
            Ok(audio_res.get_volume())
        }
    });

    Ok(audio_module)
}
