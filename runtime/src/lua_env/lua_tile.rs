use std::rc::Rc;

use mlua::{FromLua, IntoLua, UserDataMethods};

use crate::{
    game_resource::{
        ResourceId, ResourceManager,
        tile_resource::{TilemapResource, TilesetResource},
    },
    lua_env::{
        lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
        lua_vec2::Vec2,
    },
    make_resource_lua_compatible,
};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TilesetResourceId(ResourceId);
make_resource_lua_compatible!(TilesetResourceId);

fn get_tileset_from_resource_id<F, R>(
    resources: &Rc<ResourceManager>,
    tileset_resource_id: TilesetResourceId,
    f: F,
) -> Option<R>
where
    F: FnOnce(&mut tiled::Tileset) -> Option<R>,
{
    let tileset_res = resources.get_by_id::<TilesetResource>(tileset_resource_id.0);
    let Ok(tileset_res) = tileset_res else {
        return None;
    };
    let mut tileset_content = tileset_res.content.borrow_mut();
    let tileset_content = tileset_content.as_mut()?;
    f(tileset_content)
}

// ...

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TilemapResourceId(ResourceId);
make_resource_lua_compatible!(TilemapResourceId);

pub fn setup_tile_api(
    lua: &Rc<mlua::Lua>,
    resources: &Rc<ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let tile_module = lua.create_table()?;

    lua.register_userdata_type::<TilesetResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);

        registry.add_method("getTile", {
            let resources = resources.clone();
            move |lua, tileset_resource_id, (tile_id,): (u32,)| match get_tileset_from_resource_id(
                &resources,
                *tileset_resource_id,
                |tileset_content| {
                    let image = tileset_content.image.as_ref()?;
                    let tile = tileset_content.get_tile(tile_id)?;
                    let columns = tileset_content.columns;
                    let x = ((tile_id % columns) * tileset_content.tile_width) as f32
                        / image.width as f32;
                    let y = ((tile_id / columns) * tileset_content.tile_height) as f32
                        / image.height as f32;

                    let user_type = tile
                        .user_type
                        .clone()
                        .and_then(|s| lua.create_string(s).ok().map(mlua::Value::String))
                        .unwrap_or(mlua::Nil);

                    let result = lua.create_table().ok()?;
                    result.set("pos", Vec2::new(x, y)).ok()?;
                    result.set("probability", tile.probability).ok()?;
                    result.set("type", user_type).ok()?;
                    Some(mlua::Value::Table(result))
                },
            ) {
                Some(value) => Ok(value),
                None => Ok(mlua::Nil),
            }
        });

        registry.add_method("getTileSize", {
            let resources = resources.clone();
            move |_lua, tileset_resource_id, (): ()| match get_tileset_from_resource_id(
                &resources,
                *tileset_resource_id,
                |tileset_content| {
                    Some(Vec2::new(
                        tileset_content.tile_width as f32,
                        tileset_content.tile_height as f32,
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
                    let image = tileset_content.image.as_ref()?;
                    let tile_width = tileset_content.tile_width as f32;
                    let tile_height = tileset_content.tile_height as f32;
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
                    for tile in tileset_content.tiles() {
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

        registry.add_method("getTile", {
            let resources = resources.clone();
            move |_lua, tilemap_resource_id, (layer, x, y): (i32, i32, i32)| {
                let tileset_res = resources.get_by_id::<TilemapResource>(tilemap_resource_id.0);
                let Ok(tileset_res) = tileset_res else {
                    return Ok(0);
                };
                let mut tileset_content = tileset_res.content.borrow_mut();
                let tileset_content = tileset_content.as_mut();
                let tile_id = tileset_content
                    .and_then(|content| content.layers().nth(layer as usize))
                    .and_then(|l| l.as_tile_layer())
                    .and_then(|l| l.get_tile(x, y))
                    .map(|tile| tile.id())
                    .unwrap_or(0);
                Ok(tile_id)
            }
        });

        registry.add_method("getTilemapPart", {
            let resources = resources.clone();
            move |_lua,
                  tilemap_resource_id,
                  (layer, lx, ly, hx, hy, access_fn): (
                i32,
                i32,
                i32,
                i32,
                i32,
                mlua::Function,
            )| {
                let tileset_res = resources.get_by_id::<TilemapResource>(tilemap_resource_id.0);
                let Ok(tileset_res) = tileset_res else {
                    return Ok(());
                };
                let mut tileset_content = tileset_res.content.borrow_mut();
                let tileset_content = tileset_content.as_mut();
                let layer = tileset_content
                    .and_then(|content| content.layers().nth(layer as usize))
                    .and_then(|l| l.as_tile_layer());
                let Some(layer) = layer else{
                    return Ok(());
                };
                match layer {
                    tiled::TileLayer::Finite(finite_layer) => {
                        for x in lx..hx{
                            for y in ly..hy{
                                if let Some(tile) = finite_layer.get_tile_data(x, y) {
                                    let _ = access_fn.call::<()>((tile.id(), x, y));
                                }
                            }
                        }
                    },
                    tiled::TileLayer::Infinite(infinite_layer) => {
                        for x in lx..hx{
                            for y in ly..hy{
                                if let Some(tile) = infinite_layer.get_tile_data(x, y) {
                                    let _ = access_fn.call::<()>((tile.id(), x, y));
                                }
                            }
                        }
                    }
                };

                Ok(())
            }
        });
    })?;

    Ok(tile_module)
}
