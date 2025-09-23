use std::{cell::RefCell, rc::Rc};

use crate::{
    game_resource::{self},
    graphics::batchdraw,
    io,
    lua_env::add_fn_to_table,
};

pub fn setup_canvas_api(
    lua: &Rc<mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let canvas_module = lua.create_table()?;

    add_fn_to_table(lua, &canvas_module, "create", {
        let batch = batch.clone();
        |lua, (width, height): (u32, u32)| {
            // let canvas = glframebuffer::Framebuffer::new_rgba(
            //     batch.borrow().drawing_target.gl(),
            //     width,
            //     height,
            //     ImageAntialiasing::LinearWithMipmaps,
            // );
            // return Ok(canvas);
            Ok(())
        }
    });

    Ok(canvas_module)
}
