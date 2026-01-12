use std::{cell::RefCell, ops::Deref, rc::Rc};

use vectarine_plugin_sdk::mlua::{AnyUserData, FromLua, IntoLua, UserDataMethods};

use crate::{
    auto_impl_lua_clone,
    console::print_warn,
    game_resource::{self, ResourceId, shader_resource::ShaderResource},
    graphics::{
        batchdraw, glframebuffer,
        gltexture::ImageAntialiasing,
        gluniforms::{UniformValue, Uniforms},
        shape::Quad,
    },
    io,
    lua_env::{
        add_fn_to_table,
        lua_coord::{get_pos_as_vec2, get_size_as_vec2},
        lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
        lua_vec2::Vec2,
    },
    make_resource_lua_compatible,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ShaderResourceId(ResourceId);
make_resource_lua_compatible!(ShaderResourceId);

#[derive(Clone)]
pub struct RcFramebuffer {
    buffer: Rc<glframebuffer::Framebuffer>,
    shader: RefCell<Option<ShaderResourceId>>,
}
auto_impl_lua_clone!(RcFramebuffer, Framebuffer);

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
        self.shader.borrow().map(|s| s.to_resource_id())
    }
}

pub fn setup_canvas_api(
    lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
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

    lua.register_userdata_type::<ShaderResourceId>(|registry| {
        register_resource_id_methods_on_type(resources, registry);
    })?;

    lua.register_userdata_type::<RcFramebuffer>(|registry| {
        registry.add_method(
            "setShader",
            |_, canvas, (shader,): (Option<ShaderResourceId>,)| {
                canvas.shader.replace(shader);
                Ok(())
            },
        );
        registry.add_method("paint", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_, canvas, (func,): (vectarine_plugin_sdk::mlua::Function,)| {
                let mut result = Ok(());
                batch.borrow_mut().draw(&resources, true); // flush before changing framebuffer
                canvas.gl().using(|| {
                    result = func.call::<()>(());
                    batch.borrow_mut().draw(&resources, true);
                });
                result
            }
        });

        registry.add_method("setUniform", {
            let resources = resources.clone();
            move |_lua, canvas, (name, value): (String, f32)| {
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
                let warnings = shader.shader.set_uniforms(&uniforms);
                for warning in warnings {
                    print_warn(format!(
                        "Uniform {} not found in shader, maybe it was unused and optimized out?",
                        warning.uniform_name
                    ));
                }
                Ok(())
            }
        });

        registry.add_method("getSize", {
            move |_lua, canvas, (): ()| {
                let size = Vec2::new(canvas.gl().width() as f32, canvas.gl().height() as f32);
                Ok(size)
            }
        });

        registry.add_method("draw", {
            let batch = batch.clone();
            let env = env_state.clone();
            move |_, canvas, (mpos, msize): (AnyUserData, AnyUserData)| {
                let pos = get_pos_as_vec2(mpos)?;
                let size = get_size_as_vec2(msize)?;
                let framebuffer = canvas.gl();
                let shader = canvas.current_shader();
                batch
                    .borrow_mut()
                    .draw_canvas(pos, size, framebuffer, shader, &env.borrow());
                Ok(())
            }
        });

        registry.add_method("drawPart", {
            let batch = batch.clone();
            let env_state = env_state.clone();
            move |_,
                  canvas,
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
                let framebuffer = canvas.gl();
                let shader = canvas.current_shader();
                batch.borrow_mut().draw_canvas_part(
                    Quad { p1, p2, p3, p4 },
                    framebuffer,
                    src_pos,
                    src_size,
                    shader,
                    &env_state.borrow(),
                );
                Ok(())
            }
        });
    })?;

    Ok(canvas_module)
}
