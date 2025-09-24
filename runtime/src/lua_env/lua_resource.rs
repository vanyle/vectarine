use std::{cell::RefCell, path::Path, rc::Rc};

use crate::{
    game_resource::{
        ResourceId, ResourceManager, font_resource::FontResource, image_resource::ImageResource,
        shader_resource,
    },
    graphics::gltexture::ImageAntialiasing,
    lua_env::add_fn_to_table,
};

/// Adds to the Lua environment functions to interact with the outside environment
/// For example, the keyboard, the mouse, the window, etc...
/// This is called the IO API.
pub fn setup_resource_api(
    lua: &Rc<mlua::Lua>,
    resources: &Rc<ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let resources_module = lua.create_table()?;

    add_fn_to_table(lua, &resources_module, "loadImage", {
        let resources = resources.clone();
        move |_, (path, antialiasing): (String, Option<bool>)| {
            let id = resources.schedule_load_resource_with_builder::<ImageResource, _>(
                Path::new(&path),
                || ImageResource {
                    texture: RefCell::new(None),
                    antialiasing: antialiasing.map(|is_antialiasing| {
                        if is_antialiasing {
                            ImageAntialiasing::Linear
                        } else {
                            ImageAntialiasing::Nearest
                        }
                    }),
                },
            );
            Ok(id)
        }
    });

    add_fn_to_table(lua, &resources_module, "loadFont", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<FontResource>(Path::new(&path));
            Ok(id)
        }
    });

    add_fn_to_table(lua, &resources_module, "loadShader", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources
                .schedule_load_resource::<shader_resource::ShaderResource>(Path::new(&path));
            Ok(id)
        }
    });

    add_fn_to_table(lua, &resources_module, "loadScript", {
        let resources = resources.clone();
        move |lua, (path, results): (String, Option<mlua::Table>)| {
            if let Some(target_table) = results {
                let (id, table) =
                    resources.schedule_load_script_resource(Path::new(&path), target_table);
                return Ok((id, mlua::Value::Table(table)));
            }
            let dummy_table = lua.create_table().unwrap();
            let (id, table) =
                resources.schedule_load_script_resource(Path::new(&path), dummy_table);
            Ok((id, mlua::Value::Table(table)))
        }
    });

    add_fn_to_table(lua, &resources_module, "getResourceStatus", {
        let resources = resources.clone();
        move |_, id: ResourceId| {
            let status = resources.get_holder_by_id(id).get_status();
            Ok(status.to_string())
        }
    });

    add_fn_to_table(lua, &resources_module, "isResourceReady", {
        let resources = resources.clone();
        move |_, id: ResourceId| Ok(resources.get_holder_by_id(id).is_loaded())
    });

    Ok(resources_module)
}
