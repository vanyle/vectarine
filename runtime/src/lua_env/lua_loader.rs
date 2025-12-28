use std::{cell::RefCell, path::Path, rc::Rc};

use mlua::UserDataMethods;
use mlua::{FromLua, IntoLua};

use crate::game_resource::tile_resource::TilemapResource;
use crate::lua_env::lua_tile::TilemapResourceId;
use crate::{
    game_resource::{
        ResourceId, ResourceManager, audio_resource::AudioResource, font_resource::FontResource,
        image_resource::ImageResource, shader_resource::ShaderResource,
        text_resource::TextResource, tile_resource::TilesetResource,
    },
    graphics::gltexture::ImageAntialiasing,
    lua_env::{
        add_fn_to_table,
        lua_audio::AudioResourceId,
        lua_canvas::ShaderResourceId,
        lua_image::ImageResourceId,
        lua_resource::{ResourceIdWrapper, ScriptResourceId, register_resource_id_methods_on_type},
        lua_text::FontResourceId,
        lua_tile::TilesetResourceId,
    },
    make_resource_lua_compatible,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct TextResourceId(ResourceId);
make_resource_lua_compatible!(TextResourceId);

pub fn setup_loader_api(
    lua: &Rc<mlua::Lua>,
    resources: &Rc<ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let loader_module = lua.create_table()?;

    lua.register_userdata_type::<ScriptResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);
    })?;

    lua.register_userdata_type::<TextResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);

        registry.add_method("getText", {
            let resources = resources.clone();
            move |lua: &mlua::Lua, this: &TextResourceId, (): ()| {
                let resource = resources.get_by_id::<TextResource>(this.0);
                let Ok(resource) = resource else {
                    return Ok(mlua::Nil);
                };
                let content = resource.content.borrow();
                let Some(content) = content.as_ref() else {
                    return Ok(mlua::Nil);
                };
                let content = String::from_utf8_lossy(content);
                let content = lua.create_string(content.to_string())?;
                Ok(mlua::Value::String(content))
            }
        });
    })?;

    add_fn_to_table(lua, &loader_module, "loadText", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<TextResource>(Path::new(&path));
            Ok(TextResourceId::from_id(id))
        }
    });

    add_fn_to_table(lua, &loader_module, "loadImage", {
        let resources = resources.clone();
        move |_, (path, antialiasing): (String, Option<bool>)| {
            let id = resources.schedule_load_resource_with_builder::<ImageResource, _>(
                Path::new(&path),
                || ImageResource {
                    texture: RefCell::new(None),
                    egui_id: RefCell::new(None),
                    antialiasing: antialiasing.map(|is_antialiasing| {
                        if is_antialiasing {
                            ImageAntialiasing::Linear
                        } else {
                            ImageAntialiasing::Nearest
                        }
                    }),
                },
            );
            mlua::Result::Ok(ImageResourceId::from_id(id))
        }
    });

    add_fn_to_table(lua, &loader_module, "loadFont", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<FontResource>(Path::new(&path));
            Ok(FontResourceId::from_id(id))
        }
    });

    add_fn_to_table(lua, &loader_module, "loadAudio", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<AudioResource>(Path::new(&path));
            Ok(AudioResourceId::from_id(id))
        }
    });

    add_fn_to_table(lua, &loader_module, "loadShader", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<ShaderResource>(Path::new(&path));
            Ok(ShaderResourceId::from_id(id))
        }
    });

    add_fn_to_table(lua, &loader_module, "loadTileset", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<TilesetResource>(Path::new(&path));
            Ok(TilesetResourceId::from_id(id))
        }
    });

    add_fn_to_table(lua, &loader_module, "loadTilemap", {
        let resources = resources.clone();
        move |_, path: String| {
            let id = resources.schedule_load_resource::<TilemapResource>(Path::new(&path));
            Ok(TilemapResourceId::from_id(id))
        }
    });

    add_fn_to_table(lua, &loader_module, "loadScript", {
        let resources = resources.clone();
        move |lua, (path, results): (String, Option<mlua::Table>)| {
            if let Some(target_table) = results {
                let (id, table) =
                    resources.schedule_load_script_resource(Path::new(&path), target_table);
                return Ok((ScriptResourceId::from_id(id), mlua::Value::Table(table)));
            }
            let dummy_table = lua.create_table()?;
            let (id, table) =
                resources.schedule_load_script_resource(Path::new(&path), dummy_table);
            Ok((ScriptResourceId::from_id(id), mlua::Value::Table(table)))
        }
    });

    Ok(loader_module)
}
