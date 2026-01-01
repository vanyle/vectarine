use std::{cell::RefCell, rc::Rc};

use mlua::UserDataMethods;
use mlua::{FromLua, IntoLua};
use noise::{NoiseFn, Simplex, Worley};

use crate::{
    game_resource::{self, image_resource::ImageResource},
    graphics::{batchdraw, shape::Quad},
    lua_env::{
        lua_image::ImageResourceId,
        lua_vec2::Vec2,
        lua_vec4::{Vec4, WHITE},
    },
};

#[derive(Clone, Debug)]
pub struct FastList {
    pub data: Vec<Vec2>,
}
impl IntoLua for FastList {
    #[inline(always)]
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.create_any_userdata(self).map(mlua::Value::UserData)
    }
}

impl FromLua for FastList {
    #[inline(always)]
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            // Note that we are taking and not cloning the data here.
            // This means that any function that takes a FastList as argument consumes it.
            // It is thus forbidden !!! Instead use Userdata as an argument and perform the borrowing manually.
            mlua::Value::UserData(ud) => Ok(ud.take::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "FastList".to_string(),
                message: Some("Expected FastList userdata".to_string()),
            }),
        }
    }
}

impl FastList {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn from_vec(data: Vec<Vec2>) -> Self {
        Self { data }
    }
}

impl Default for FastList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub enum FastListOrVec<'a> {
    List(&'a FastList),
    Vec(Vec2),
}

fn parse_fastlist_or_vec_with_cb<F, T>(ud: mlua::AnyUserData, f: F) -> mlua::Result<T>
where
    F: Fn(&FastListOrVec) -> mlua::Result<T>,
{
    if let Ok(vec) = ud.borrow::<Vec2>() {
        f(&FastListOrVec::Vec(*vec))
    } else if let Ok(list) = ud.borrow::<FastList>() {
        f(&FastListOrVec::List(&list))
    } else {
        Err(mlua::Error::RuntimeError(
            "Expected FastList or Vec2".to_string(),
        ))
    }
}

/// Helper function to apply a binary operation element-wise between a FastList and either another FastList or a Vec2
fn apply_binary_op<F>(this: &FastList, other: &FastListOrVec, op: F) -> FastList
where
    F: Fn(Vec2, Vec2) -> Vec2,
{
    let mut data = Vec::with_capacity(this.data.len());
    match other {
        FastListOrVec::List(other_list) => {
            let len = this.data.len().min(other_list.data.len());
            for i in 0..len {
                data.push(op(this.data[i], other_list.data[i]));
            }
        }
        FastListOrVec::Vec(vec) => {
            for v in &this.data {
                data.push(op(*v, *vec));
            }
        }
    }
    FastList::from_vec(data)
}

pub fn setup_fastlist_api(
    lua: &mlua::Lua,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    lua.register_userdata_type::<FastList>(|registry| {
        registry.add_meta_method(mlua::MetaMethod::Len, |_, this, ()| Ok(this.data.len()));

        registry.add_method("get", |_, this, index: usize| {
            if index == 0 || index > this.data.len() {
                return Ok(None);
            }
            Ok(Some(this.data[index - 1]))
        });

        registry.add_method("concat", |_, this, other: FastList| {
            let mut new_data = this.data.clone();
            new_data.extend(other.data);
            Ok(FastList::from_vec(new_data))
        });

        registry.add_method("toTable", |_, this, ()| Ok(this.data.clone()));

        registry.add_method_mut("forEach", |_, this, func: mlua::Function| {
            for (i, vec) in this.data.iter_mut().enumerate() {
                // 1-indexed for Lua
                *vec = func.call::<Vec2>((*vec, i + 1))?;
            }
            Ok(())
        });

        registry.add_method("componentRepeated", |_, this, count: usize| {
            let mut new_data = Vec::with_capacity(this.data.len() * count);
            for vec in &this.data {
                for _ in 0..count {
                    new_data.push(*vec);
                }
            }
            Ok(FastList::from_vec(new_data))
        });

        registry.add_method("repeated", |_, this, count: usize| {
            let mut new_data = Vec::with_capacity(this.data.len() * count);
            for _ in 0..count {
                new_data.extend(&this.data);
            }
            Ok(FastList::from_vec(new_data))
        });

        registry.add_method("weave", |_, this, others: Vec<FastList>| {
            if others.is_empty() {
                return Ok(this.clone());
            }

            // Find the maximum length among all lists
            let mut max_len = this.data.len();
            for other in &others {
                max_len = max_len.max(other.data.len());
            }

            let total_lists = 1 + others.len();
            let mut new_data = Vec::with_capacity(max_len * total_lists);

            // Weave elements: take one from each list in order
            for i in 0..max_len {
                if i < this.data.len() {
                    new_data.push(this.data[i]);
                }
                for other in &others {
                    if i < other.data.len() {
                        new_data.push(other.data[i]);
                    }
                }
            }

            Ok(FastList::from_vec(new_data))
        });

        registry.add_method(
            "filterGtX",
            |_, this, (maybe_mask, threshold): (mlua::AnyUserData, f32)| {
                let mask = maybe_mask.borrow::<FastList>()?;
                let filtered_data: Vec<Vec2> = this
                    .data
                    .iter()
                    .zip(mask.data.iter())
                    .filter(|(_, mask)| mask.x() > threshold)
                    .map(|(vec, _)| *vec)
                    .collect();
                Ok(FastList::from_vec(filtered_data))
            },
        );

        registry.add_meta_method(
            mlua::MetaMethod::Add,
            |_, this, other: mlua::AnyUserData| {
                parse_fastlist_or_vec_with_cb(other, |parsed| {
                    Ok(apply_binary_op(this, parsed, |a, b| a + b))
                })
            },
        );
        registry.add_meta_method(
            mlua::MetaMethod::Sub,
            |_, this, other: mlua::AnyUserData| {
                parse_fastlist_or_vec_with_cb(other, |parsed| {
                    Ok(apply_binary_op(this, parsed, |a, b| a - b))
                })
            },
        );
        registry.add_meta_method(
            mlua::MetaMethod::Mul,
            |_, this, other: mlua::AnyUserData| {
                parse_fastlist_or_vec_with_cb(other, |parsed| {
                    Ok(apply_binary_op(this, parsed, |a, b| a * b))
                })
            },
        );

        registry.add_method("scale", |_, this, k: f32| {
            let data = this.data.iter().map(|v| *v * k).collect();
            Ok(FastList::from_vec(data))
        });

        registry.add_method("cmul", |_, this, other: mlua::AnyUserData| {
            parse_fastlist_or_vec_with_cb(other, |parsed| {
                Ok(apply_binary_op(this, parsed, |a, b| a.cmul(b)))
            })
        });

        registry.add_method("dot", |_, this, other: mlua::AnyUserData| {
            parse_fastlist_or_vec_with_cb(other, |parsed| {
                Ok(apply_binary_op(this, parsed, |a, b| {
                    let d = a.dot(&b);
                    Vec2::new(d, d)
                }))
            })
        });

        registry.add_method("normalized", |_, this, ()| {
            let data = this.data.iter().map(|v| v.normalized()).collect();
            Ok(FastList::from_vec(data))
        });

        registry.add_method("round", |_, this, digits: Option<u32>| {
            let data = this.data.iter().map(|v| v.round(digits)).collect();
            Ok(FastList::from_vec(data))
        });

        registry.add_method("floor", |_, this, ()| {
            let data = this.data.iter().map(|v| v.floor()).collect();
            Ok(FastList::from_vec(data))
        });

        registry.add_method("ceil", |_, this, ()| {
            let data = this.data.iter().map(|v| v.ceil()).collect();
            Ok(FastList::from_vec(data))
        });

        registry.add_method("max", |_, this, other: mlua::AnyUserData| {
            parse_fastlist_or_vec_with_cb(other, |parsed| {
                Ok(apply_binary_op(this, parsed, |a, b| a.max(b)))
            })
        });

        registry.add_method("min", |_, this, other: mlua::AnyUserData| {
            parse_fastlist_or_vec_with_cb(other, |parsed| {
                Ok(apply_binary_op(this, parsed, |a, b| a.min(b)))
            })
        });

        registry.add_method("toPolar", |_, this, ()| {
            let data = this.data.iter().map(|v| v.to_polar()).collect();
            Ok(FastList::from_vec(data))
        });

        registry.add_method("toCartesian", |_, this, ()| {
            let data = this.data.iter().map(|v| v.to_cartesian()).collect();
            Ok(FastList::from_vec(data))
        });

        registry.add_method("lerp", |_, this, (other, k): (mlua::AnyUserData, f32)| {
            parse_fastlist_or_vec_with_cb(other, |parsed| {
                Ok(apply_binary_op(this, parsed, |a, b| a.lerp(b, k)))
            })
        });

        registry.add_method("sign", |_, this, ()| {
            let data = this.data.iter().map(|v| v.sign()).collect();
            Ok(FastList::from_vec(data))
        });

        let simplex = Simplex::new(noise::Simplex::DEFAULT_SEED);
        registry.add_method("noise", move |_, this, ()| {
            let noise_values: Vec<Vec2> = this
                .data
                .iter()
                .map(|vec| {
                    let noise_val = simplex.get([vec.x() as f64, vec.y() as f64]) as f32;
                    Vec2::new(noise_val, noise_val)
                })
                .collect();
            Ok(FastList::from_vec(noise_values))
        });

        let worley = Worley::new(noise::Worley::DEFAULT_SEED);
        registry.add_method("worleyNoise", move |_, this, ()| {
            let noise_values: Vec<Vec2> = this
                .data
                .iter()
                .map(|vec| {
                    let noise_val = worley.get([vec.x() as f64, vec.y() as f64]) as f32;
                    Vec2::new(noise_val, noise_val)
                })
                .collect();
            Ok(FastList::from_vec(noise_values))
        });

        registry.add_method("drawRects", {
            let batch = batch.clone();
            move |_, this: &FastList, ()| {
                let mut batch = batch.borrow_mut();
                for chunk in this.data.chunks(4) {
                    if chunk.len() < 4 {
                        break;
                    }
                    let pos = chunk[0];
                    let size = chunk[1];
                    let c1 = chunk[2];
                    let c2 = chunk[3];
                    let color = [c1.x(), c1.y(), c2.x(), c2.y()];
                    batch.draw_rect(pos.x(), pos.y(), size.x(), size.y(), color);
                }
                Ok(())
            }
        });

        registry.add_method("drawQuads", {
            let batch = batch.clone();
            move |_, this: &FastList, ()| {
                let mut batch = batch.borrow_mut();
                for chunk in this.data.chunks(6) {
                    if chunk.len() < 6 {
                        break;
                    }
                    let p1 = chunk[0];
                    let p2 = chunk[1];
                    let p3 = chunk[2];
                    let p4 = chunk[3];
                    let c1 = chunk[4];
                    let c2 = chunk[5];
                    let color = [c1.x(), c1.y(), c2.x(), c2.y()];
                    batch.draw_polygon([p1, p2, p3, p4].into_iter(), color);
                }
                Ok(())
            }
        });

        registry.add_method("drawImages", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_, this: &FastList, (image_id, color): (ImageResourceId, Option<Vec4>)| {
                let image = resources.get_by_id::<ImageResource>(image_id.0);
                let Ok(image) = image else {
                    return Ok(());
                };
                let binding = image.texture.borrow();
                let Some(tex) = binding.as_ref() else {
                    return Ok(());
                };

                let mut batch = batch.borrow_mut();
                for chunk in this.data.chunks(2) {
                    if chunk.len() < 2 {
                        break;
                    }
                    let pos = chunk[0];
                    let size = chunk[1];
                    batch.draw_image(
                        pos.x(),
                        pos.y(),
                        size.x(),
                        size.y(),
                        tex,
                        color.unwrap_or(WHITE).0,
                    );
                }
                Ok(())
            }
        });

        registry.add_method("drawImageParts", {
            let batch = batch.clone();
            let resources = resources.clone();
            move |_, this: &FastList, (image_id, color): (ImageResourceId, Option<Vec4>)| {
                let image = resources.get_by_id::<ImageResource>(image_id.0);
                let Ok(image) = image else {
                    return Ok(());
                };
                let binding = image.texture.borrow();
                let Some(tex) = binding.as_ref() else {
                    return Ok(());
                };

                let mut batch = batch.borrow_mut();
                for chunk in this.data.chunks(6) {
                    if chunk.len() < 6 {
                        break;
                    }
                    let p1 = chunk[0];
                    let p2 = chunk[1];
                    let p3 = chunk[2];
                    let p4 = chunk[3];
                    let src_pos = chunk[4];
                    let src_size = chunk[5];
                    let quad = Quad { p1, p2, p3, p4 };
                    batch.draw_image_part(quad, tex, src_pos, src_size, color.unwrap_or(WHITE).0);
                }
                Ok(())
            }
        });
    })?;

    let fastlist_module = lua.create_table()?;

    fastlist_module.set(
        "newLinspace",
        lua.create_function(|_lua, (min, max, step): (Vec2, Vec2, Vec2)| {
            if step.x() <= 0.0 || step.y() <= 0.0 {
                return Err(mlua::Error::RuntimeError(
                    "step components must be positive".to_string(),
                ));
            }
            let estimated_size = ((max.x() - min.x()) / step.x()) as usize
                * ((max.y() - min.y()) / step.y()) as usize;
            let mut data = Vec::with_capacity(estimated_size);

            let mut y = min.y();
            while y <= max.y() {
                let mut x = min.x();
                while x <= max.x() {
                    data.push(Vec2::new(x, y));
                    x += step.x();
                }
                y += step.y();
            }
            data.shrink_to_fit();
            Ok(FastList::from_vec(data))
        })?,
    )?;

    fastlist_module.set(
        "fromTable",
        lua.create_function(|_, data: Vec<Vec2>| Ok(FastList::from_vec(data)))?,
    )?;

    fastlist_module.set(
        "zeros",
        lua.create_function(|_, size: usize| {
            let data = vec![Vec2::zero(); size];
            Ok(FastList::from_vec(data))
        })?,
    )?;

    fastlist_module.set(
        "fromValue",
        lua.create_function(|_, (value, size): (Vec2, usize)| {
            let data = vec![value; size];
            Ok(FastList::from_vec(data))
        })?,
    )?;

    Ok(fastlist_module)
}
