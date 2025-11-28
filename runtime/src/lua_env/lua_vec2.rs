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
    pub fn new(x: f32, y: f32) -> Self {
        Self([x, y])
    }
    pub fn x(&self) -> f32 {
        self.0[0]
    }
    pub fn y(&self) -> f32 {
        self.0[1]
    }
    pub fn with_x(self, x: f32) -> Self {
        Self([x, self.0[1]])
    }
    pub fn with_y(self, y: f32) -> Self {
        Self([self.0[0], y])
    }
    pub fn scale(self, k: f32) -> Self {
        self * k
    }
    pub fn cmul(self, other: Self) -> Self {
        Self::new(
            self.0[0] * other.0[0] - self.0[1] * other.0[1],
            self.0[1] * other.0[0] + self.0[0] * other.0[1],
        )
    }
    pub fn rotated(self, angle_rad: f32) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        self.cmul(Self::new(cos_a, sin_a))
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
        methods.add_method("normalized", |_, vec, ()| Ok(vec.normalized()));
        methods.add_method("round", |_, vec, (digits_of_precision,): (Option<u32>,)| {
            Ok(vec.round(digits_of_precision))
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
        lua.create_function(|lua, (x, y): (f32, f32)| {
            let data = mlua::Value::UserData(lua.create_userdata(Vec2::new(x, y))?);
            Ok(data)
        })?,
    )?;

    vec2_module.set("ZERO2", Vec2::zero())?;
    Ok(vec2_module)
}
