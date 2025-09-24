use std::{cell::RefCell, rc::Rc};

use mlua::UserData;

use crate::{
    game_resource::{self},
    graphics::{batchdraw, glframebuffer, gltexture::ImageAntialiasing},
    io,
    lua_env::add_fn_to_table,
};

#[derive(Clone)]
pub struct RcFramebuffer(Rc<glframebuffer::Framebuffer>);

impl RcFramebuffer {
    fn new(fb: glframebuffer::Framebuffer) -> Self {
        RcFramebuffer(Rc::new(fb))
    }
    pub fn gl(&self) -> &glframebuffer::Framebuffer {
        self.0.as_ref()
    }
}

impl UserData for RcFramebuffer {}

impl mlua::FromLua for RcFramebuffer {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Framebuffer".to_string(),
                message: Some("Expected Framebuffer userdata".to_string()),
            }),
        }
    }
}

pub fn setup_canvas_api(
    lua: &Rc<mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    _resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let canvas_module = lua.create_table()?;

    add_fn_to_table(lua, &canvas_module, "createCanvas", {
        let batch = batch.clone();
        move |_lua, (width, height): (u32, u32)| {
            let canvas = RcFramebuffer::new(glframebuffer::Framebuffer::new_rgba(
                batch.borrow().drawing_target.gl(),
                width,
                height,
                ImageAntialiasing::LinearWithMipmaps,
            ));
            Ok(canvas)
        }
    });

    add_fn_to_table(lua, &canvas_module, "paint", {
        let batch = batch.clone();
        move |_lua, (canvas, func): (RcFramebuffer, mlua::Function)| {
            let mut result = Ok(());
            canvas.0.using(|| {
                result = func.call::<()>(());
                batch.borrow_mut().draw(true);
            });
            result
        }
    });

    Ok(canvas_module)
}
