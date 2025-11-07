use std::{cell::RefCell, ops::Deref, rc::Rc};

use mlua::{AnyUserData, UserData};

use crate::{
    game_resource::{self, ResourceId, shader_resource::ShaderResource},
    graphics::{
        batchdraw, glframebuffer,
        gltexture::ImageAntialiasing,
        gluniforms::{UniformValue, Uniforms},
    },
    io,
    lua_env::{add_fn_to_table, lua_vec2::Vec2},
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

    add_fn_to_table(lua, &canvas_module, "setUniform", {
        let resources = resources.clone();
        move |_lua, (canvas, name, value): (RcFramebuffer, String, f32)| {
            let shader_id = canvas.current_shader();
            let Some(shader_id) = shader_id else {
                return Ok(()); // no op if no shader is set
            };
            let shader = resources.get_by_id::<ShaderResource>(shader_id);
            let Ok(shader) = shader else {
                return Ok(()); // no op if shader resource is not loaded
            };
            let mut shader = shader.shader.borrow_mut();
            let shader = shader.as_mut();
            let Some(shader) = shader else {
                return Ok(()); // no op if shader is not compiled
            };
            shader.shader.use_program();
            let mut uniforms = Uniforms::new();
            uniforms.add(&name, UniformValue::Float(value));
            shader.shader.set_uniforms(&uniforms);
            Ok(())
        }
    });

    add_fn_to_table(lua, &canvas_module, "getSize", {
        let resources = resources.clone();
        move |_lua, (canvas_or_image,): (AnyUserData,)| {
            let maybe_canvas = canvas_or_image.borrow::<RcFramebuffer>();
            if let Ok(canvas) = maybe_canvas {
                let size = Vec2::new(canvas.gl().width() as f32, canvas.gl().height() as f32);
                return Ok(size);
            }
            let maybe_image = canvas_or_image.borrow::<ResourceId>();
            let Ok(resource_id) = maybe_image else {
                return Err(mlua::Error::FromLuaConversionError {
                    from: "unknown",
                    to: "Canvas | ImageResource".to_string(),
                    message: Some("Expected Canvas or ImageResource userdata".to_string()),
                });
            };
            let image_resource =
                resources.get_by_id::<game_resource::image_resource::ImageResource>(*resource_id);
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

    Ok(canvas_module)
}
