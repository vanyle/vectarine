use noise::{NoiseFn, Simplex, Worley};

use crate::lua_env::lua_vec2::Vec2;

#[derive(Clone)]
pub struct FastList {
    pub data: Vec<Vec2>,
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

impl mlua::FromLua for FastList {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => {
                let list = ud.borrow::<Self>()?;
                Ok(list.clone())
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "FastList".to_string(),
                message: Some("expected FastList userdata".to_string()),
            }),
        }
    }
}

impl mlua::UserData for FastList {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Len, |_, this, ()| Ok(this.data.len()));

        methods.add_method_mut("forEach", |_, this, func: mlua::Function| {
            for (i, vec) in this.data.iter_mut().enumerate() {
                // 1-indexed for Lua
                *vec = func.call::<Vec2>((*vec, i + 1))?;
            }
            Ok(())
        });

        methods.add_method(
            "group",
            |lua, this, (group_size, func): (usize, mlua::Function)| {
                if group_size == 0 {
                    return Err(mlua::Error::RuntimeError(
                        "groupSize must be greater than 0".to_string(),
                    ));
                }
                let mut result_data = Vec::new();
                for chunk in this.data.chunks(group_size) {
                    let input_table = lua.create_table()?;
                    for (i, vec) in chunk.iter().enumerate() {
                        input_table.set(i + 1, *vec)?;
                    }
                    let output_table: mlua::Table = func.call(input_table)?;
                    for pair in output_table.pairs::<usize, Vec2>() {
                        let (_, vec) = pair?;
                        result_data.push(vec);
                    }
                }

                Ok(FastList::from_vec(result_data))
            },
        );

        methods.add_method_mut(
            "linearMap",
            |_, this, (translation, scale, cmul): (Vec2, Vec2, Vec2)| {
                for vec in this.data.iter_mut() {
                    // Apply transformation: (v * scale + translation).cmul(cmul)
                    *vec = (*vec * scale + translation).cmul(cmul);
                }
                Ok(())
            },
        );

        methods.add_method_mut(
            "mapZip",
            |_, this, (other, func): (FastList, mlua::Function)| {
                this.data
                    .iter_mut()
                    .zip(other.data.iter())
                    .enumerate()
                    .for_each(|(i, (vec1, vec2))| {
                        let r = func.call::<Vec2>((*vec1, *vec2, i + 1));
                        if let Ok(r) = r {
                            *vec1 = r;
                        }
                    });
                Ok(())
            },
        );

        methods.add_method("copy", |_, this, ()| Ok(this.clone()));

        // add - element-wise addition with another FastList
        methods.add_method_mut("add", |_, this, other: FastList| {
            this.data
                .iter_mut()
                .zip(other.data.iter())
                .for_each(|(vec1, vec2)| {
                    *vec1 = *vec1 + *vec2;
                });
            Ok(())
        });

        // sub - element-wise subtraction with another FastList
        methods.add_method_mut("sub", |_, this, other: FastList| {
            this.data
                .iter_mut()
                .zip(other.data.iter())
                .for_each(|(vec1, vec2)| {
                    *vec1 = *vec1 - *vec2;
                });
            Ok(())
        });

        // noise - compute simplex noise for each Vec2, return new FastList with noise values
        let simplex = Simplex::new(noise::Simplex::DEFAULT_SEED);
        methods.add_method("noise", move |_, this, ()| {
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

        // worleyNoise - compute Worley noise for each Vec2, return new FastList with noise values
        let worley = Worley::new(noise::Worley::DEFAULT_SEED);
        methods.add_method("worleyNoise", move |_, this, ()| {
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

        methods.add_method_mut(
            "filterGtX",
            |_, this, (mask, threshold): (FastList, f32)| {
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
    }
}

pub fn setup_fastlist_api(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
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

    Ok(fastlist_module)
}
