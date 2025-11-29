use crate::math::Vect;

pub type Vec4 = Vect<4>;

impl mlua::FromLua for Vec4 {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Vec4".to_string(),
                message: Some("expected Vec4 userdata".to_string()),
            }),
        }
    }
}

impl Vec4 {
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w])
    }
    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.0[0]
    }
    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.0[1]
    }
    #[inline(always)]
    pub fn z(&self) -> f32 {
        self.0[2]
    }
    #[inline(always)]
    pub fn w(&self) -> f32 {
        self.0[3]
    }
    #[inline]
    pub fn with_x(self, x: f32) -> Self {
        Self([x, self.0[1], self.0[2], self.0[3]])
    }
    #[inline]
    pub fn with_y(self, y: f32) -> Self {
        Self([self.0[0], y, self.0[2], self.0[3]])
    }
    #[inline]
    pub fn with_z(self, z: f32) -> Self {
        Self([self.0[0], self.0[1], z, self.0[3]])
    }
    #[inline]
    pub fn with_w(self, w: f32) -> Self {
        Self([self.0[0], self.0[1], self.0[2], w])
    }
    #[inline]
    pub fn scale(self, k: f32) -> Self {
        self * k
    }
    #[inline]
    pub fn cmul(self, other: Self) -> Self {
        // Quaternion multiplication
        Self::new(
            self.0[0] * other.0[0]
                - self.0[1] * other.0[1]
                - self.0[2] * other.0[2]
                - self.0[3] * other.0[3],
            self.0[0] * other.0[1] + self.0[1] * other.0[0] + self.0[2] * other.0[3]
                - self.0[3] * other.0[2],
            self.0[0] * other.0[2] - self.0[1] * other.0[3]
                + self.0[2] * other.0[0]
                + self.0[3] * other.0[1],
            self.0[0] * other.0[3] + self.0[1] * other.0[2] - self.0[2] * other.0[1]
                + self.0[3] * other.0[0],
        )
    }
}

impl mlua::UserData for Vec4 {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, v| Ok(v.0[0]));
        fields.add_field_method_get("y", |_, v| Ok(v.0[1]));
        fields.add_field_method_get("z", |_, v| Ok(v.0[2]));
        fields.add_field_method_get("w", |_, v| Ok(v.0[3]));

        // Field aliases for colors
        fields.add_field_method_get("r", |_, v| Ok(v.0[0]));
        fields.add_field_method_get("g", |_, v| Ok(v.0[1]));
        fields.add_field_method_get("b", |_, v| Ok(v.0[2]));
        fields.add_field_method_get("a", |_, v| Ok(v.0[3]));

        fields.add_field_method_set("x", |_, vec, v| {
            vec.0[0] = v;
            Ok(())
        });
        fields.add_field_method_set("y", |_, vec, v| {
            vec.0[1] = v;
            Ok(())
        });
        fields.add_field_method_set("z", |_, vec, v| {
            vec.0[2] = v;
            Ok(())
        });
        fields.add_field_method_set("w", |_, vec, v| {
            vec.0[3] = v;
            Ok(())
        });

        // Field aliases for colors
        fields.add_field_method_set("r", |_, vec, v| {
            vec.0[0] = v;
            Ok(())
        });
        fields.add_field_method_set("g", |_, vec, v| {
            vec.0[1] = v;
            Ok(())
        });
        fields.add_field_method_set("b", |_, vec, v| {
            vec.0[2] = v;
            Ok(())
        });
        fields.add_field_method_set("a", |_, vec, v| {
            vec.0[3] = v;
            Ok(())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("length", |_, vec, ()| Ok(vec.length()));
        methods.add_method("lengthSq", |_, vec, ()| Ok(vec.length_sq()));
        methods.add_method("distance", |_, vec, (other,): (Vec4,)| {
            Ok((*vec - other).length())
        });
        methods.add_method("scale", |_, vec, (k,): (f32,)| Ok(*vec * k));
        methods.add_method("cmul", |_, vec, (other,): (Vec4,)| Ok(vec.cmul(other)));
        methods.add_method("dot", |_, vec, (other,): (Vec4,)| Ok(vec.dot(&other)));
        methods.add_method("normalized", |_, vec, ()| Ok(vec.normalized()));
        methods.add_method("round", |_, vec, (digits_of_precision,): (Option<u32>,)| {
            Ok(vec.round(digits_of_precision))
        });
        methods.add_method("floor", |_, vec, ()| Ok(vec.floor()));
        methods.add_method("ceil", |_, vec, ()| Ok(vec.ceil()));
        methods.add_method("max", |_, vec, (other,): (Vec4,)| Ok(vec.max(other)));
        methods.add_method("min", |_, vec, (other,): (Vec4,)| Ok(vec.min(other)));
        methods.add_method("lerp", |_, vec, (other, k): (Vec4, f32)| {
            Ok(vec.lerp(other, k))
        });
        methods.add_method("sign", |_, vec, ()| Ok(vec.sign()));
        methods.add_meta_function(mlua::MetaMethod::Add, |_, (vec, other): (Vec4, Vec4)| {
            Ok(vec + other)
        });
        methods.add_meta_function(mlua::MetaMethod::Sub, |_, (vec, other): (Vec4, Vec4)| {
            Ok(vec - other)
        });
        methods.add_meta_function(mlua::MetaMethod::Mul, |_, (vec, other): (Vec4, Vec4)| {
            Ok(vec * other)
        });
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, vec, _any: mlua::Value| {
            Ok(format!(
                "V4({}, {}, {}, {})",
                vec.0[0], vec.0[1], vec.0[2], vec.0[3]
            ))
        });
    }
}

pub const BLACK: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
pub const WHITE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

const RED: Vec4 = Vec4::new(1.0, 0.0, 0.0, 1.0);
const GREEN: Vec4 = Vec4::new(0.0, 1.0, 0.0, 1.0);
const BLUE: Vec4 = Vec4::new(0.0, 0.0, 1.0, 1.0);
const YELLOW: Vec4 = Vec4::new(1.0, 1.0, 0.0, 1.0);
const MAGENTA: Vec4 = Vec4::new(1.0, 0.0, 1.0, 1.0);
const CYAN: Vec4 = Vec4::new(0.0, 1.0, 1.0, 1.0);
const AZURE: Vec4 = Vec4::new(0.0, 0.5, 1.0, 1.0);
const ORANGE: Vec4 = Vec4::new(1.0, 0.5, 0.0, 1.0);
const PURPLE: Vec4 = Vec4::new(0.5, 0.0, 0.5, 1.0);
const SPRING: Vec4 = Vec4::new(0.0, 1.0, 0.5, 1.0);
const LIME: Vec4 = Vec4::new(0.5, 1.0, 0.0, 1.0);
const PINK: Vec4 = Vec4::new(1.0, 0.0, 1.0, 1.0);
const LIGHT_GRAY: Vec4 = Vec4::new(0.7, 0.7, 0.7, 1.0);
const DARK_GRAY: Vec4 = Vec4::new(0.3, 0.3, 0.3, 1.0);
const GRAY: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);
const DARK_RED: Vec4 = Vec4::new(0.5, 0.0, 0.0, 1.0);
const DARK_GREEN: Vec4 = Vec4::new(0.0, 0.5, 0.0, 1.0);
const DARK_BLUE: Vec4 = Vec4::new(0.0, 0.0, 0.5, 1.0);

pub fn setup_vec_api(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
    let vec4_module = lua.create_table()?;
    vec4_module.set(
        "V4",
        lua.create_function(|_lua, (x, y, z, w): (f32, f32, f32, f32)| Ok(Vec4::new(x, y, z, w)))?,
    )?;
    vec4_module.set(
        "createColor",
        lua.create_function(|_lua, (r, g, b, a): (f32, f32, f32, f32)| Ok(Vec4::new(r, g, b, a)))?,
    )?;

    vec4_module.set("ZERO4", Vec4::zero())?;

    // Default colors
    vec4_module.set("BLACK", BLACK)?;
    vec4_module.set("WHITE", WHITE)?;
    vec4_module.set("TRANSPARENT", TRANSPARENT)?;

    vec4_module.set("RED", RED)?;
    vec4_module.set("GREEN", GREEN)?;
    vec4_module.set("BLUE", BLUE)?;

    vec4_module.set("YELLOW", YELLOW)?;
    vec4_module.set("MAGENTA", MAGENTA)?;
    vec4_module.set("CYAN", CYAN)?;

    vec4_module.set("AZURE", AZURE)?;
    vec4_module.set("ORANGE", ORANGE)?;
    vec4_module.set("PURPLE", PURPLE)?;
    vec4_module.set("SPRING", SPRING)?;
    vec4_module.set("LIME", LIME)?;
    vec4_module.set("PINK", PINK)?;

    vec4_module.set("LIGHT_GRAY", LIGHT_GRAY)?;
    vec4_module.set("DARK_GRAY", DARK_GRAY)?;

    vec4_module.set("GRAY", GRAY)?;
    vec4_module.set("DARK_RED", DARK_RED)?;
    vec4_module.set("DARK_GREEN", DARK_GREEN)?;
    vec4_module.set("DARK_BLUE", DARK_BLUE)?;

    Ok(vec4_module)
}
