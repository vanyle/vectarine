use std::{cell::RefCell, rc::Rc};

use vectarine_plugin_sdk::mlua::{self, AnyUserData, FromLua, IntoLua, UserDataMethods};

use crate::{
    auto_impl_lua_copy, console,
    game_resource::{
        self, ResourceId, ResourceManager, image_resource::ImageResource,
        tile_resource::TilesetContent,
    },
    graphics::{batchdraw, shape::Quad},
    io,
    lua_env::{
        lua_coord::{get_pos_as_vec2, get_size_as_vec2},
        lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
        lua_tile::{TilesetResourceId, get_tileset_from_resource_id},
        lua_vec2::Vec2,
        lua_vec4::{Vec4, WHITE},
        stringify_lua_value,
    },
    make_resource_lua_compatible,
};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct ImageResourceId(pub ResourceId);
make_resource_lua_compatible!(ImageResourceId);

#[derive(Debug, Clone, Copy)]
pub struct ImageWithTileset {
    pub image_id: ImageResourceId,
    pub tileset_id: TilesetResourceId,
}
auto_impl_lua_copy!(ImageWithTileset, ImageWithTileset);

pub fn setup_image_api(
    lua: &vectarine_plugin_sdk::mlua::Lua,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let image_module = lua.create_table()?;

    lua.register_userdata_type::<ImageResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);

        registry.add_method("getSize", {
            let resources = resources.clone();
            move |_lua, image_resource_id, (): ()| {
                let image_resource = resources
                    .get_by_id::<game_resource::image_resource::ImageResource>(image_resource_id.0);
                let Ok(image_resource) = image_resource else {
                    return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                        "ImageResource not found".to_string(),
                    ));
                };
                let image_resource = image_resource.texture.borrow();
                let Some(image_texture) = image_resource.as_ref() else {
                    return Err(vectarine_plugin_sdk::mlua::Error::RuntimeError(
                        "ImageResource texture not loaded".to_string(),
                    ));
                };
                let size = Vec2::new(image_texture.width() as f32, image_texture.height() as f32);
                Ok(size)
            }
        });

        registry.add_method("draw", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_lua,
                  image_resource_id,
                  (mpos, msize, color): (AnyUserData, AnyUserData, Option<Vec4>)| {
                let pos = get_pos_as_vec2(mpos)?;
                let size = get_size_as_vec2(msize)?;
                let tex = resources.get_by_id::<ImageResource>(image_resource_id.0);
                let Ok(tex) = tex else {
                    return Ok(());
                };
                let tex = tex.texture.borrow();
                let Some(tex) = tex.as_ref() else {
                    return Ok(());
                };
                batch.borrow_mut().draw_image(
                    pos.x(),
                    pos.y(),
                    size.x(),
                    size.y(),
                    tex,
                    color.unwrap_or(WHITE).0,
                );
                Ok(())
            }
        });

        registry.add_method("drawPart", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_,
                  image_resource_id,
                  (mp1, mp2, mp3, mp4, src_pos, src_size, color): (
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                Vec2,
                Vec2,
                Option<Vec4>,
            )| {
                let p1 = get_pos_as_vec2(mp1)?;
                let p2 = get_pos_as_vec2(mp2)?;
                let p3 = get_pos_as_vec2(mp3)?;
                let p4 = get_pos_as_vec2(mp4)?;
                let tex = resources.get_by_id::<ImageResource>(image_resource_id.0);
                let Ok(tex) = tex else {
                    return Ok(());
                };
                let tex = tex.texture.borrow();
                let Some(tex) = tex.as_ref() else {
                    return Ok(());
                };
                let quad = Quad { p1, p2, p3, p4 };
                batch.borrow_mut().draw_image_part(
                    quad,
                    tex,
                    src_pos,
                    src_size,
                    color.unwrap_or(WHITE).0,
                );
                Ok(())
            }
        });

        registry.add_method(
            "withTileset",
            |_, image_resource_id, (tileset_id,): (TilesetResourceId,)| {
                Ok(ImageWithTileset {
                    image_id: *image_resource_id,
                    tileset_id,
                })
            },
        );
    })?;

    lua.register_userdata_type::<ImageWithTileset>(|registry| {
        registry.add_method("drawTile", {
            let resources = resources.clone();
            let batch = batch.clone();
            move |_,
                  image_with_tileset,
                  (tile_id, pos, size, color): (
                mlua::Value,
                AnyUserData,
                AnyUserData,
                Option<Vec4>,
            )| {
                let pos = get_pos_as_vec2(pos)?;
                let size = get_size_as_vec2(size)?;
                draw_tile_part(
                    &resources,
                    &batch,
                    image_with_tileset,
                    &[(
                        tile_id,
                        Quad {
                            p1: pos,
                            p2: Vec2::new(pos.x() + size.x(), pos.y()),
                            p3: Vec2::new(pos.x() + size.x(), pos.y() + size.y()),
                            p4: Vec2::new(pos.x(), pos.y() + size.y()),
                        },
                    )],
                    lua_value_to_tile_id,
                    color,
                );
                Ok(())
            }
        });

        registry.add_method("drawTileQuad", {
            let resources = resources.clone();
            let batch = batch.clone();
            move |_,
                  image_with_tileset,
                  (tile_id, p1, p2, p3, p4, color): (
                mlua::Value,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                Option<Vec4>,
            )| {
                let p1 = get_pos_as_vec2(p1)?;
                let p2 = get_pos_as_vec2(p2)?;
                let p3 = get_pos_as_vec2(p3)?;
                let p4 = get_pos_as_vec2(p4)?;
                let quad = Quad { p1, p2, p3, p4 };
                draw_tile_part(
                    &resources,
                    &batch,
                    image_with_tileset,
                    &[(tile_id, quad)],
                    lua_value_to_tile_id,
                    color,
                );
                Ok(())
            }
        });
    })?;

    Ok(image_module)
}

fn lua_value_to_tile_id(lua_value: &mlua::Value, tileset: &TilesetContent) -> Option<i64> {
    match lua_value {
        // Lua integers are i32 or i64 depending on the platform, so we need this cast.
        #[allow(clippy::unnecessary_cast)]
        mlua::Value::Integer(id) => Some(*id as i64),
        mlua::Value::String(name) => {
            let id = tileset.type_mapping.get(&*name.as_bytes());
            let Some(id) = id else {
                let name_str = name.to_string_lossy();
                console::print_err(format!("Tile name '{}' not found in tileset", name_str));
                return None;
            };
            Some(*id as i64)
        }
        _ => {
            console::print_err(format!(
                "Unable to draw tile {} as it is not a string or an id",
                stringify_lua_value(lua_value)
            ));
            None
        }
    }
}

pub fn draw_tile_part<T>(
    resources: &Rc<ResourceManager>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    image_with_tileset: &ImageWithTileset,
    tile_ids_with_quads: &[(T, Quad)],
    t_to_i64: impl Fn(&T, &TilesetContent) -> Option<i64>,
    color: Option<Vec4>,
) {
    let tex = resources.get_by_id::<ImageResource>(image_with_tileset.image_id.0);
    get_tileset_from_resource_id(resources, image_with_tileset.tileset_id, |tileset| {
        let tex = tex.ok()?;
        let tex = tex.texture.borrow();
        let tex = tex.as_ref()?;

        let column_count = tileset.tiled.columns as i64;
        let tile_width = tileset.tiled.tile_width as i64;
        let tile_height = tileset.tiled.tile_height as i64;
        let spacing = tileset.tiled.spacing as i64;
        let margin = tileset.tiled.margin as i64;

        let (quads, uv_pos_size): (Vec<Quad>, Vec<(Vec2, Vec2)>) = tile_ids_with_quads
            .iter()
            .filter_map(|(tile_id, quad)| {
                // tile_id can be a number or a string. It it's a string, we need to find the corresponding tile id in the tileset
                let id = t_to_i64(tile_id, tileset)?;
                let x = id % column_count * (tile_width + spacing) + margin;
                let y = id / column_count * (tile_height + spacing) + margin;

                let src_pos = Vec2::new(
                    x as f32 / tex.width() as f32,
                    y as f32 / tex.height() as f32,
                );
                let src_size = Vec2::new(
                    tile_width as f32 / tex.width() as f32,
                    tile_height as f32 / tex.height() as f32,
                );
                Some((*quad, (src_pos, src_size)))
            })
            .unzip();

        batch
            .borrow_mut()
            .draw_images_part(&quads, tex, &uv_pos_size, color.unwrap_or(WHITE).0);
        Some(())
    });
}
