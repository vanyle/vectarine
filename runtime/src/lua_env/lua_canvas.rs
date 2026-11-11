use std::{cell::RefCell, ops::Deref, rc::Rc};

use vectarine_plugin_sdk::mlua::{AnyUserData, FromLua, IntoLua, UserDataMethods};

use crate::{
    auto_impl_lua_clone,
    console::print_warn,
    game_resource::{self, ResourceId, shader_resource::ShaderResource},
    graphics::{
        batchdraw::{self, CanvasShader},
        customshader::{UniformError, UniformValue},
        framebuffer,
        shape::Quad,
        texture::ImageAntialiasing,
    },
    io,
    lua_env::{
        add_fn_to_table,
        lua_coord::{get_pos_as_vec2, get_size_as_vec2},
        lua_resource::{ResourceIdWrapper, register_resource_id_methods_on_type},
        lua_vec2::Vec2,
        lua_vec4::Vec4,
    },
    make_resource_lua_compatible,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ShaderResourceId(ResourceId);
make_resource_lua_compatible!(ShaderResourceId);

#[derive(Clone)]
pub struct RcFramebuffer {
    buffer: Rc<framebuffer::Framebuffer>,
    shader: RefCell<Option<ShaderResourceId>>,
}
auto_impl_lua_clone!(RcFramebuffer, Framebuffer);

impl RcFramebuffer {
    fn new(fb: framebuffer::Framebuffer) -> Self {
        RcFramebuffer {
            buffer: Rc::new(fb),
            shader: RefCell::new(None),
        }
    }
    pub fn framebuffer(&self) -> &framebuffer::Framebuffer {
        self.buffer.deref()
    }
    pub fn current_shader(&self) -> Option<ResourceId> {
        self.shader.borrow().map(|s| s.to_resource_id())
    }

    /// The shader used to draw the canvas, with the current values of its uniforms.
    fn shader_for_drawing(
        &self,
        resources: &game_resource::ResourceManager,
        env: &io::IoEnvState,
    ) -> Option<CanvasShader> {
        let id = self.current_shader()?;
        // TODO: We use Instant::now() here which is non-deterministic when testing.
        // We should instead store the start time of the current frame in the env and use that.
        let elapsed = std::time::Instant::now() - env.start_time;
        let uniforms = resources
            .get_by_id::<ShaderResource>(id)
            .ok()
            .and_then(|shader| {
                let shader = shader.shader.borrow();
                shader.as_ref()?.uniform_snapshot(elapsed.as_secs_f32())
            });
        Some(CanvasShader { id, uniforms })
    }
}

pub fn setup_canvas_api(
    lua: &vectarine_plugin_sdk::mlua::Lua,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
    let canvas_module = lua.create_table()?;

    add_fn_to_table(lua, &canvas_module, "createCanvas", {
        let batch = batch.clone();
        move |_lua, (width, height): (u32, u32)| {
            let canvas = RcFramebuffer::new(framebuffer::Framebuffer::new_rgba(
                batch.borrow().gfx().gpu(),
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
                let gfx = batch.borrow().gfx().clone();
                canvas.framebuffer().using(&gfx, || {
                    result = func.call::<()>(());
                    batch.borrow_mut().draw(&resources, true);
                });
                result
            }
        });

        registry.add_method("setUniform", {
            let resources = resources.clone();
            move |_lua, canvas, (name, value): (String, vectarine_plugin_sdk::mlua::Value)| {
                let shader_id = canvas.current_shader();
                let Some(shader_id) = shader_id else {
                    return Ok(()); // no op if no shader is set
                };
                let shader = resources.get_by_id::<ShaderResource>(shader_id);
                let Ok(shader) = shader else {
                    return Ok(()); // no op if shader resource is not loaded
                };
                let shader = shader.shader.borrow();
                let Some(shader) = shader.as_ref() else {
                    return Ok(()); // no op if shader is not compiled
                };
                let value = match value {
                    // In Lua code, integers and float are confused, so we treat them the same.
                    vectarine_plugin_sdk::mlua::Value::Number(n) => UniformValue::Float(n as f32),
                    vectarine_plugin_sdk::mlua::Value::Integer(n) => UniformValue::Float(n as f32),
                    vectarine_plugin_sdk::mlua::Value::UserData(ud) => {
                        if let Ok(vec2) = Vec2::from_userdata(&ud) {
                            UniformValue::Vec2([vec2.x(), vec2.y()])
                        } else if let Ok(vec4) = Vec4::from_userdata(&ud) {
                            UniformValue::Vec4([vec4.x(), vec4.y(), vec4.z(), vec4.w()])
                        } else {
                            return Err(vectarine_plugin_sdk::mlua::Error::external(
                                "Unable to set a uniform value to a value of this type",
                            ));
                        }
                    }
                    _ => {
                        return Err(vectarine_plugin_sdk::mlua::Error::external(
                            "Unable to set a uniform value to a value of this type",
                        ));
                    }
                };
                match shader.set_uniform(&name, value) {
                    Ok(()) => {}
                    Err(UniformError::NotFound) => print_warn(format!(
                        "Uniform {name} not found in shader: it needs to be a member of the uniform struct of the shader"
                    )),
                    Err(UniformError::WrongType { expected }) => print_warn(format!(
                        "Uniform {name} cannot receive this value, the shader declares `{expected}`"
                    )),
                    Err(UniformError::OutOfBounds { len }) => print_warn(format!(
                        "Uniform {name} is outside of its array, which has {len} elements"
                    )),
                }
                Ok(())
            }
        });

        registry.add_method("getSize", {
            move |_lua, canvas, (): ()| {
                let size = Vec2::new(canvas.framebuffer().width() as f32, canvas.framebuffer().height() as f32);
                Ok(size)
            }
        });

        registry.add_method("draw", {
            let batch = batch.clone();
            let env = env_state.clone();
            let resources = resources.clone();
            move |_, canvas, (mpos, msize): (AnyUserData, AnyUserData)| {
                let pos = get_pos_as_vec2(mpos)?;
                let size = get_size_as_vec2(msize)?;
                let framebuffer = canvas.framebuffer();
                let shader = canvas.shader_for_drawing(&resources, &env.borrow());
                batch
                    .borrow_mut()
                    .draw_canvas(pos, size, framebuffer, shader);
                Ok(())
            }
        });

        registry.add_method("drawPart", {
            let batch = batch.clone();
            let env_state = env_state.clone();
            let resources = resources.clone();
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
                let framebuffer = canvas.framebuffer();
                let shader = canvas.shader_for_drawing(&resources, &env_state.borrow());
                batch.borrow_mut().draw_canvas_part(
                    Quad { p1, p2, p3, p4 },
                    framebuffer,
                    src_pos,
                    src_size,
                    shader,
                );
                Ok(())
            }
        });
    })?;

    Ok(canvas_module)
}
