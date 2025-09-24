use std::{cell::RefCell, ops::Deref, rc::Rc};

use mlua::UserData;

use crate::{
    game_resource::{self, ResourceId},
    graphics::{batchdraw, glframebuffer, gltexture::ImageAntialiasing},
    io,
    lua_env::add_fn_to_table,
};

#[derive(Clone)]
pub struct RcFramebuffer {
    buffer: Rc<glframebuffer::Framebuffer>,
    shader: RefCell<Option<ResourceId>>,
}

impl RcFramebuffer {
    fn new(fb: glframebuffer::Framebuffer) -> Self {
        RcFramebuffer {
            buffer: Rc::new(fb),
            shader: RefCell::new(None),
        }
    }
    pub fn gl(&self) -> &glframebuffer::Framebuffer {
        self.buffer.deref()
    }
    pub fn current_shader(&self) -> Option<ResourceId> {
        *self.shader.borrow()
    }
}

impl UserData for RcFramebuffer {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "setShader",
            |_, canvas, (shader,): (Option<ResourceId>,)| {
                canvas.shader.replace(shader);
                Ok(())
            },
        );
    }
}

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
    resources: &Rc<game_resource::ResourceManager>,
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
        let resources = resources.clone();
        move |_lua, (canvas, func): (RcFramebuffer, mlua::Function)| {
            let mut result = Ok(());
            batch.borrow_mut().draw(&resources, true); // flush before changing framebuffer
            canvas.gl().using(|| {
                result = func.call::<()>(());
                batch.borrow_mut().draw(&resources, true);
            });
            result
        }
    });

    Ok(canvas_module)
}
