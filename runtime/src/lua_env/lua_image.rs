use std::{cell::RefCell, rc::Rc};

use mlua::{AnyUserData, FromLua, IntoLua, UserDataMethods};

use crate::{
    game_resource::{self, ResourceId, image_resource::ImageResource},
    graphics::{batchdraw, shape::Quad},
    io,
    lua_env::{
        lua_coord::{get_pos_as_vec2, get_size_as_vec2},
        lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
        lua_vec2::Vec2,
    },
    make_resource_lua_compatible,
};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct ImageResourceId(ResourceId);
make_resource_lua_compatible!(ImageResourceId);

pub fn setup_image_api(
    lua: &Rc<mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let image_module = lua.create_table()?;

    lua.register_userdata_type::<ImageResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);

        registry.add_method("getSize", {
            let resources = resources.clone();
            move |_lua, image_resource_id, (): ()| {
                let image_resource = resources
                    .get_by_id::<game_resource::image_resource::ImageResource>(image_resource_id.0);
                let Ok(image_resource) = image_resource else {
                    return Err(mlua::Error::RuntimeError(
                        "ImageResource not found".to_string(),
                    ));
                };
                let image_resource = image_resource.texture.borrow();
                let Some(image_texture) = image_resource.as_ref() else {
                    return Err(mlua::Error::RuntimeError(
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
            move |_lua, image_resource_id, (mpos, msize): (AnyUserData, AnyUserData)| {
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
                batch
                    .borrow_mut()
                    .draw_image(pos.x, pos.y, size.x, size.y, tex);
                Ok(())
            }
        });

        registry.add_method("drawPart", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_,
                  image_resource_id,
                  (mp1, mp2, mp3, mp4, src_pos, src_size): (
                AnyUserData,
                AnyUserData,
                AnyUserData,
                AnyUserData,
                Vec2,
                Vec2,
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
                batch
                    .borrow_mut()
                    .draw_image_part(quad, tex, src_pos, src_size);
                Ok(())
            }
        });
    })?;

    Ok(image_module)
}
