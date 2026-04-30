use std::rc::Rc;

use vectarine_plugin_sdk::mlua::{FromLua, IntoLua, UserDataMethods};

use crate::{
    game_resource::{
        ResourceId, ResourceManager,
        tile_resource::{TilemapResource, TilesetContent, TilesetResource},
    },
    lua_env::{
        lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
        lua_tile::tilemap::GeneratedTilemap,
        lua_vec2::Vec2,
    },
    make_resource_lua_compatible,
};

pub mod tilemap;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TilesetResourceId(ResourceId);
make_resource_lua_compatible!(TilesetResourceId);

pub fn get_tileset_from_resource_id<F, R>(
    resources: &Rc<ResourceManager>,
    tileset_resource_id: TilesetResourceId,
    f: F,
) -> Option<R>
where
    F: FnOnce(&mut TilesetContent) -> Option<R>,
{
    let tileset_res = resources.get_by_id::<TilesetResource>(tileset_resource_id.0);
    let Ok(tileset_res) = tileset_res else {
        return None;
    };
    let mut tileset_content = tileset_res.content.borrow_mut();
    let tileset_content = tileset_content.as_mut()?;
    f(tileset_content)
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TilemapResourceId(ResourceId);
make_resource_lua_compatible!(TilemapResourceId);

pub fn get_tilemap_from_resource_id<F, R>(
    resources: &Rc<ResourceManager>,
    tilemap_resource_id: TilemapResourceId,
    f: F,
) -> Option<R>
where
    F: FnOnce(&mut tiled::Map) -> Option<R>,
{
    let tilemap_res = resources.get_by_id::<TilemapResource>(tilemap_resource_id.0);
    let Ok(tilemap_res) = tilemap_res else {
        return None;
    };
    let mut tilemap_content = tilemap_res.content.borrow_mut();
    let tilemap_content = tilemap_content.as_mut()?;
    f(tilemap_content)
}

pub fn setup_tile_api(
    lua: &vectarine_plugin_sdk::mlua::Lua,
    resources: &Rc<ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let tile_module = lua.create_table()?;

    lua.register_userdata_type::<TilesetResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);

        registry.add_method("getTile", {
            let resources = resources.clone();
            move |lua, tileset_resource_id, (tile_id,): (u32,)| match get_tileset_from_resource_id(
                &resources,
                *tileset_resource_id,
                |tileset_content| {
                    let image = tileset_content.tiled.image.as_ref()?;
                    let tile = tileset_content.tiled.get_tile(tile_id)?;
                    let columns = tileset_content.tiled.columns;
                    let x = ((tile_id % columns) * tileset_content.tiled.tile_width) as f32
                        / image.width as f32;
                    let y = ((tile_id / columns) * tileset_content.tiled.tile_height) as f32
                        / image.height as f32;

                    let user_type = tile
                        .user_type
                        .clone()
                        .and_then(|s| {
                            lua.create_string(s)
                                .ok()
                                .map(vectarine_plugin_sdk::mlua::Value::String)
                        })
                        .unwrap_or(vectarine_plugin_sdk::mlua::Nil);

                    let result = lua.create_table().ok()?;
                    result.set("pos", Vec2::new(x, y)).ok()?;
                    result.set("probability", tile.probability).ok()?;
                    result.set("type", user_type).ok()?;
                    Some(vectarine_plugin_sdk::mlua::Value::Table(result))
                },
            ) {
                Some(value) => Ok(value),
                None => Ok(vectarine_plugin_sdk::mlua::Nil),
            }
        });

        registry.add_method("getTileSize", {
            let resources = resources.clone();
            move |_lua, tileset_resource_id, (): ()| match get_tileset_from_resource_id(
                &resources,
                *tileset_resource_id,
                |tileset_content| {
                    Some(Vec2::new(
                        tileset_content.tiled.tile_width as f32,
                        tileset_content.tiled.tile_height as f32,
                    ))
                },
            ) {
                Some(value) => Ok(value),
                None => Ok(Vec2::zero()),
            }
        });

        registry.add_method("getTileRatio", {
            let resources = resources.clone();
            move |_lua, tileset_resource_id, (): ()| match get_tileset_from_resource_id(
                &resources,
                *tileset_resource_id,
                |tileset_content| {
                    let image = tileset_content.tiled.image.as_ref()?;
                    let tile_width = tileset_content.tiled.tile_width as f32;
                    let tile_height = tileset_content.tiled.tile_height as f32;
                    Some(Vec2::new(
                        image.width as f32 / tile_width,
                        image.height as f32 / tile_height,
                    ))
                },
            ) {
                Some(value) => Ok(value),
                None => Ok(Vec2::zero()),
            }
        });

        registry.add_method("getTiles", {
            let resources = resources.clone();
            move |lua, tileset_resource_id, (): ()| match get_tileset_from_resource_id(
                &resources,
                *tileset_resource_id,
                |tileset_content| {
                    let table = lua.create_table().ok()?;
                    for tile in tileset_content.tiled.tiles() {
                        table.push(tile.0).ok()?;
                    }
                    Some(table)
                },
            ) {
                Some(value) => Ok(value),
                None => lua.create_table(),
            }
        })
    })?;

    lua.register_userdata_type::<TilemapResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);
        tilemap::register_tilemap_methods_on_type(resources, registry);
    })?;

    lua.register_userdata_type::<GeneratedTilemap>(|registry| {
        tilemap::register_tilemap_methods_on_type(resources, registry);

        registry.add_method_mut(
            "invalidate",
            |_lua, this, (layer, x, y): (i32, i32, i32)| {
                this.invalidate(layer, x, y);
                Ok(())
            },
        );
    })?;

    tile_module.set(
        "createGeneratedTilemap",
        lua.create_function(|lua, generator: vectarine_plugin_sdk::mlua::Function| {
            let tilemap = GeneratedTilemap {
                get_chunk_fn: generator,
                cache: std::collections::HashMap::new(),
            };
            lua.create_any_userdata(tilemap)
        })?,
    )?;

    Ok(tile_module)
}
