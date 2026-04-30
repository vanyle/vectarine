use std::collections::HashMap;
use std::rc::Rc;

use vectarine_plugin_sdk::mlua::{self, FromLua, IntoLua, UserDataMethods, UserDataRegistry};

use crate::{
    game_resource::{ResourceManager, tile_resource::TilemapResource},
    lua_env::lua_tile::TilemapResourceId,
};

const CHUNK_SIZE: i32 = 16;

pub trait Tilemap {
    fn get_tile(
        &mut self,
        resources: &Rc<ResourceManager>,
        layer: i32,
        x: i32,
        y: i32,
    ) -> Option<u32>;
    fn get_tile_part(
        &mut self,
        resources: &Rc<ResourceManager>,
        layer: i32,
        lx: i32,
        ly: i32,
        hx: i32,
        hy: i32,
        callback: impl FnMut(u32, i32, i32) -> mlua::Result<()>,
    ) -> mlua::Result<()>;
}

/// A generated tilemap is a tilemap that is generated dynamically by a Lua function
/// and is not stored as a resource. It can be used to create infinite tilemaps for example.
pub struct GeneratedTilemap {
    pub get_chunk_fn: mlua::Function,
    pub cache: HashMap<(i32, i32, i32), Vec<u32>>,
}

impl IntoLua for GeneratedTilemap {
    fn into_lua(
        self,
        lua: &mlua::Lua,
    ) -> mlua::Result<mlua::Value> {
        lua.create_any_userdata(self)
            .map(mlua::Value::UserData)
    }
}

impl FromLua for GeneratedTilemap {
    fn from_lua(
        value: mlua::Value,
        _: &mlua::Lua,
    ) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => ud.take::<Self>(),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "GeneratedTilemap".to_string(),
                message: Some("Expected GeneratedTilemap userdata".to_string()),
            }),
        }
    }
}

impl Tilemap for GeneratedTilemap {
    fn get_tile(
        &mut self,
        _resources: &Rc<ResourceManager>,
        layer: i32,
        x: i32,
        y: i32,
    ) -> Option<u32> {
        let chunk_x = x.div_euclid(CHUNK_SIZE);
        let chunk_y = y.div_euclid(CHUNK_SIZE);
        self.ensure_chunk(layer, chunk_x, chunk_y)?;
        let chunk = self.cache.get(&(layer, chunk_x, chunk_y))?;
        let local_x = x.rem_euclid(CHUNK_SIZE) as usize;
        let local_y = y.rem_euclid(CHUNK_SIZE) as usize;
        chunk.get(local_y * CHUNK_SIZE as usize + local_x).copied()
    }

    fn get_tile_part(
        &mut self,
        _resources: &Rc<ResourceManager>,
        layer: i32,
        lx: i32,
        ly: i32,
        hx: i32,
        hy: i32,
        mut callback: impl FnMut(u32, i32, i32) -> mlua::Result<()>,
    ) -> mlua::Result<()> {
        for y in ly..hy {
            for x in lx..hx {
                let chunk_x = x.div_euclid(CHUNK_SIZE);
                let chunk_y = y.div_euclid(CHUNK_SIZE);
                if self.ensure_chunk(layer, chunk_x, chunk_y).is_none() {
                    continue;
                }
                if let Some(chunk) = self.cache.get(&(layer, chunk_x, chunk_y)) {
                    let local_x = x.rem_euclid(CHUNK_SIZE) as usize;
                    let local_y = y.rem_euclid(CHUNK_SIZE) as usize;
                    if let Some(&tile) = chunk.get(local_y * CHUNK_SIZE as usize + local_x) {
                        callback(tile, x, y)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl GeneratedTilemap {
    /// Ensures the chunk at (layer, chunk_x, chunk_y) is in the cache.
    /// Calls the Lua generator function if the chunk is missing.
    fn ensure_chunk(&mut self, layer: i32, chunk_x: i32, chunk_y: i32) -> Option<()> {
        if self.cache.contains_key(&(layer, chunk_x, chunk_y)) {
            return Some(());
        }
        let result: mlua::Table = self.get_chunk_fn.call((layer, chunk_x, chunk_y)).ok()?;
        let size = (CHUNK_SIZE * CHUNK_SIZE) as usize;
        let chunk: Vec<u32> = (1..=size)
            .map(|i| result.get::<u32>(i).unwrap_or(0))
            .collect();
        self.cache.insert((layer, chunk_x, chunk_y), chunk);
        Some(())
    }

    pub fn invalidate(&mut self, layer: i32, x: i32, y: i32) {
        let chunk_x = x.div_euclid(CHUNK_SIZE);
        let chunk_y = y.div_euclid(CHUNK_SIZE);
        self.cache.remove(&(layer, chunk_x, chunk_y));
    }
}

impl Tilemap for TilemapResourceId {
    fn get_tile(
        &mut self,
        resources: &Rc<ResourceManager>,
        layer: i32,
        x: i32,
        y: i32,
    ) -> Option<u32> {
        let tilemap_res = resources.get_by_id::<TilemapResource>(self.0).ok()?;
        let content = tilemap_res.content.borrow();
        let content = content.as_ref()?;
        content
            .layers()
            .nth(layer as usize)?
            .as_tile_layer()?
            .get_tile(x, y)
            .map(|tile| tile.id())
    }

    fn get_tile_part(
        &mut self,
        resources: &Rc<ResourceManager>,
        layer: i32,
        lx: i32,
        ly: i32,
        hx: i32,
        hy: i32,
        mut callback: impl FnMut(u32, i32, i32) -> mlua::Result<()>,
    ) -> mlua::Result<()> {
        let tilemap_res = resources
            .get_by_id::<TilemapResource>(self.0)
            .map_err(|_| mlua::Error::RuntimeError("Tilemap resource not found".to_string()))?;
        let content = tilemap_res.content.borrow();
        let content = content
            .as_ref()
            .ok_or_else(|| mlua::Error::RuntimeError("Tilemap not loaded".to_string()))?;

        let tile_layer = content
            .get_layer(layer as usize)
            .and_then(|l| l.as_tile_layer());
        let Some(tile_layer) = tile_layer else {
            return Err(mlua::Error::RuntimeError(
                "Tilemap layer not found".to_string(),
            ));
        };

        match tile_layer {
            tiled::TileLayer::Finite(finite_layer) => {
                for x in lx..hx {
                    for y in ly..hy {
                        if let Some(tile) = finite_layer.get_tile_data(x, y) {
                            callback(tile.id(), x, y)?;
                        }
                    }
                }
            }
            tiled::TileLayer::Infinite(infinite_layer) => {
                for x in lx..hx {
                    for y in ly..hy {
                        if let Some(tile) = infinite_layer.get_tile_data(x, y) {
                            callback(tile.id(), x, y)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn register_tilemap_methods_on_type<T: Tilemap + 'static>(
    resources: &Rc<ResourceManager>,
    registry: &mut UserDataRegistry<T>,
) {
    registry.add_method_mut("getTile", {
        let resources = resources.clone();
        move |_lua, this, (layer, x, y): (i32, i32, i32)| {
            Ok(this.get_tile(&resources, layer, x, y).unwrap_or(0))
        }
    });

    registry.add_method_mut("getTilemapPart", {
        let resources = resources.clone();
        move |_lua,
              this,
              (layer, lx, ly, hx, hy, access_fn): (
            i32,
            i32,
            i32,
            i32,
            i32,
            mlua::Function,
        )| {
            this.get_tile_part(&resources, layer, lx, ly, hx, hy, |tile_id, x, y| {
                access_fn.call::<()>((tile_id, x, y))
            })
        }
    });
}
