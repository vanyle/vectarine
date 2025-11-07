use std::ops;

#[derive(Clone, Debug, Default, PartialEq, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl mlua::FromLua for Vec2 {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Vec2".to_string(),
                message: Some("expected Vec2 userdata".to_string()),
            }),
        }
    }
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn with_x(self, x: f32) -> Self {
        Self { x, y: self.y }
    }
    pub fn with_y(self, y: f32) -> Self {
        Self { x: self.x, y }
    }
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    pub fn scale(self, k: f32) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
        }
    }
    pub fn cmul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x - self.y * other.y,
            y: self.y * other.x + self.x * other.y,
        }
    }
    pub fn rotated(self, angle_rad: f32) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        self.cmul(Self { x: cos_a, y: sin_a })
    }
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self { x: 0.0, y: 0.0 }
        } else {
            self.scale(1.0 / len)
        }
    }
}

impl ops::Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::Mul for Vec2 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl mlua::UserData for Vec2 {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, v| Ok(v.x));
        fields.add_field_method_get("y", |_, v| Ok(v.y));
        fields.add_field_method_set("x", |_, vec, v| {
            vec.x = v;
            Ok(())
        });
        fields.add_field_method_set("y", |_, vec, v| {
            vec.y = v;
            Ok(())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("length", |_, vec, ()| {
            Ok((vec.x * vec.x + vec.y * vec.y).sqrt())
        });
        methods.add_method("scale", |_, vec, (k,): (f32,)| {
            Ok(Vec2 {
                x: vec.x * k,
                y: vec.y * k,
            })
        });
        methods.add_method("cmul", |_, vec, (other,): (Vec2,)| {
            Ok(Vec2 {
                x: vec.x * other.x - vec.y * other.y,
                y: vec.y * other.x + vec.x * other.y,
            })
        });
        methods.add_method("normalized", |_, vec, ()| {
            let len = (vec.x * vec.x + vec.y * vec.y).sqrt();
            if len == 0.0 {
                // We pick an arbitrary direction in this case
                return Ok(Vec2 { x: 1.0, y: 0.0 });
            }
            Ok(Vec2 {
                x: vec.x / len,
                y: vec.y / len,
            })
        });
        methods.add_method("round", |_, vec, (digits_of_precision,): (Option<u32>,)| {
            let factor = 10f32.powi(digits_of_precision.unwrap_or(0) as i32);
            Ok(Vec2 {
                x: (vec.x * factor).round() / factor,
                y: (vec.y * factor).round() / factor,
            })
        });
        methods.add_meta_function(mlua::MetaMethod::Add, |_, (vec, other): (Vec2, Vec2)| {
            Ok(Vec2 {
                x: vec.x + other.x,
                y: vec.y + other.y,
            })
        });
        methods.add_meta_function(mlua::MetaMethod::Sub, |_, (vec, other): (Vec2, Vec2)| {
            Ok(Vec2 {
                x: vec.x - other.x,
                y: vec.y - other.y,
            })
        });
        methods.add_meta_function(mlua::MetaMethod::Mul, |_, (vec, other): (Vec2, Vec2)| {
            Ok(Vec2 {
                x: vec.x * other.x,
                y: vec.y * other.y,
            })
        });
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, vec, _any: mlua::Value| {
            Ok(format!("V2({}, {})", vec.x, vec.y))
        });
    }
}

pub fn setup_vec_api(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let vec2_module = lua.create_table()?;
    vec2_module.set(
        "V2",
        lua.create_function(|lua, (x, y): (f32, f32)| {
            let data = mlua::Value::UserData(lua.create_userdata(Vec2 { x, y })?);
            Ok(data)
        })?,
    )?;

    vec2_module.set("ZERO2", Vec2 { x: 0.0, y: 0.0 })?;

    Ok(vec2_module)
}
