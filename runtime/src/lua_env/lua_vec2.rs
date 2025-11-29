use crate::math::Vect;

pub type Vec2 = Vect<2>;

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
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self([x, y])
    }
    #[inline(always)]
    pub const fn x(&self) -> f32 {
        self.0[0]
    }
    #[inline(always)]
    pub const fn y(&self) -> f32 {
        self.0[1]
    }
    #[inline]
    pub const fn with_x(self, x: f32) -> Self {
        Self([x, self.0[1]])
    }
    #[inline]
    pub const fn with_y(self, y: f32) -> Self {
        Self([self.0[0], y])
    }
    #[inline]
    pub fn scale(self, k: f32) -> Self {
        self * k
    }
    #[inline]
    pub fn cmul(self, other: Self) -> Self {
        Self::new(
            self.0[0] * other.0[0] - self.0[1] * other.0[1],
            self.0[1] * other.0[0] + self.0[0] * other.0[1],
        )
    }
    #[inline]
    pub fn rotated(self, angle_rad: f32) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        self.cmul(Self::new(cos_a, sin_a))
    }
    #[inline]
    pub fn angle(self) -> f32 {
        self.0[1].atan2(self.0[0])
    }
    #[inline]
    pub fn from_angle(angle_rad: f32) -> Self {
        Self::new(angle_rad.cos(), angle_rad.sin())
    }
}

impl mlua::UserData for Vec2 {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, v| Ok(v.0[0]));
        fields.add_field_method_get("y", |_, v| Ok(v.0[1]));
        fields.add_field_method_set("x", |_, vec, v| {
            vec.0[0] = v;
            Ok(())
        });
        fields.add_field_method_set("y", |_, vec, v| {
            vec.0[1] = v;
            Ok(())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("length", |_, vec, ()| Ok(vec.length()));
        methods.add_method("scale", |_, vec, (k,): (f32,)| Ok(*vec * k));
        methods.add_method("cmul", |_, vec, (other,): (Vec2,)| Ok(vec.cmul(other)));
        methods.add_method("dot", |_, vec, (other,): (Vec2,)| Ok(vec.dot(&other)));
        methods.add_method("lengthSq", |_, vec, ()| Ok(vec.length_sq()));
        methods.add_method("normalized", |_, vec, ()| Ok(vec.normalized()));
        methods.add_method("round", |_, vec, (digits_of_precision,): (Option<u32>,)| {
            Ok(vec.round(digits_of_precision))
        });
        methods.add_method("angle", |_, vec, ()| Ok(vec.angle()));
        methods.add_method("floor", |_, vec, ()| Ok(vec.floor()));
        methods.add_method("lerp", |_, vec, (other, k)| Ok(vec.lerp(other, k)));
        methods.add_method("ceil", |_, vec, ()| Ok(vec.ceil()));
        methods.add_method("max", |_, vec, (other,): (Vec2,)| Ok(vec.max(other)));
        methods.add_method("min", |_, vec, (other,): (Vec2,)| Ok(vec.min(other)));
        methods.add_method("sign", |_, vec, ()| Ok(vec.sign()));
        methods.add_method("distance", |_, vec, (other,): (Vec2,)| {
            Ok((*vec - other).length())
        });
        methods.add_meta_function(mlua::MetaMethod::Add, |_, (vec, other): (Vec2, Vec2)| {
            Ok(vec + other)
        });
        methods.add_meta_function(mlua::MetaMethod::Sub, |_, (vec, other): (Vec2, Vec2)| {
            Ok(vec - other)
        });
        methods.add_meta_function(mlua::MetaMethod::Mul, |_, (vec, other): (Vec2, Vec2)| {
            Ok(vec * other)
        });
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, vec, _any: mlua::Value| {
            Ok(format!("V2({}, {})", vec.0[0], vec.0[1]))
        });
    }
}

pub fn setup_vec_api(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let vec2_module = lua.create_table()?;
    vec2_module.set(
        "V2",
        lua.create_function(|_lua, (x, y): (f32, f32)| Ok(Vec2::new(x, y)))?,
    )?;

    vec2_module.set(
        "fromAngle",
        lua.create_function(|_lua, (angle_rad, length): (f32, Option<f32>)| {
            let v2 = Vec2::from_angle(angle_rad);
            let scaled = v2 * length.unwrap_or(1.0);
            Ok(scaled)
        })?,
    )?;

    vec2_module.set("ZERO2", Vec2::zero())?;
    Ok(vec2_module)
}
