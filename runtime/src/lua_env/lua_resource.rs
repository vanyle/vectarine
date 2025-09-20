use std::{path::Path, rc::Rc};

use crate::{
    game_resource::{
        ResourceId, ResourceManager, font_resource::FontResource, image_resource::ImageResource,
        script_resource::ScriptResource,
    },
    lua_env::add_global_fn,
};

/// Adds to the Lua environment functions to interact with the outside environment
/// For example, the keyboard, the mouse, the window, etc...
/// This is called the IO API.
pub fn setup_resource_api(
    lua: &Rc<mlua::Lua>,
    resources: &Rc<ResourceManager>,
) -> mlua::Result<()> {
    add_global_fn(lua, "loadImage", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<ImageResource>(Path::new(&path));
            Ok(id)
        }
    });

    add_global_fn(lua, "loadFont", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<FontResource>(Path::new(&path));
            Ok(id)
        }
    });

    add_global_fn(lua, "loadScript", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<ScriptResource>(Path::new(&path));
            Ok(id)
        }
    });

    add_global_fn(lua, "getResourceStatus", {
        let resources = resources.clone();
        move |_, id: ResourceId| {
            let status = resources.get_holder_by_id(id).get_status();
            Ok(status.to_string())
        }
    });

    add_global_fn(lua, "isResourceReady", {
        let resources = resources.clone();
        move |_, id: ResourceId| Ok(resources.get_holder_by_id(id).is_loaded())
    });

    Ok(())
}
