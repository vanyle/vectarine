use noise::{NoiseFn, Simplex, Worley};

use crate::math::Vect;

pub type Vec2 = Vect<2>;
use std::{
    cmp::{self, Ordering},
    ops,
};

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
    // hyper-area in n dimensions where n is 2
    pub fn area(self) -> f32 {
        self.x + self.y + self.x + self.y
    }
    // hyper-volume in n dimensions where n is 2
    pub fn volume(self) -> f32 {
        self.x * self.y
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

    #[inline]
    pub fn to_polar(self) -> Self {
        Self::new(self.length(), self.angle())
    }
    #[inline]
    pub fn to_cartesian(self) -> Self {
        let length = self.x();
        let angle = self.y();
        Self::from_angle(angle) * length
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self { x: 0.0, y: 0.0 }
        } else {
            self.scale(1.0 / len)
        }
    }
    pub fn min(self, other: Self) -> Self {
        Self {
            x: f32::min(self.x, other.x),
            y: f32::min(self.y, other.y),
        }
    }
    pub fn max(self, other: Self) -> Self {
        Self {
            x: f32::max(self.x, other.x),
            y: f32::max(self.y, other.y),
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

impl cmp::PartialOrd for Vec2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x < other.x && self.y < other.y {
            return Some(Ordering::Less);
        } else if self.x > other.x && self.y > other.y {
            return Some(Ordering::Greater);
        }
        return None;
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
        methods.add_method(
            "length",
            #[inline(always)]
            |_, vec, ()| Ok(vec.length()),
        );
        methods.add_method(
            "scale",
            #[inline(always)]
            |_, vec, (k,): (f32,)| Ok(*vec * k),
        );
        methods.add_method(
            "cmul",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| Ok(vec.cmul(other)),
        );
        methods.add_method(
            "dot",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| Ok(vec.dot(&other)),
        );
        methods.add_method(
            "lengthSq",
            #[inline(always)]
            |_, vec, ()| Ok(vec.length_sq()),
        );
        methods.add_method(
            "normalized",
            #[inline(always)]
            |_, vec, ()| Ok(vec.normalized()),
        );
        methods.add_method(
            "abs",
            #[inline(always)]
            |_, vec, ()| Ok(vec.abs()),
        );
        methods.add_method(
            "round",
            #[inline(always)]
            |_, vec, (digits_of_precision,): (Option<u32>,)| Ok(vec.round(digits_of_precision)),
        );
        methods.add_method(
            "angle",
            #[inline(always)]
            |_, vec, ()| Ok(vec.angle()),
        );
        methods.add_method(
            "floor",
            #[inline(always)]
            |_, vec, ()| Ok(vec.floor()),
        );
        methods.add_method(
            "lerp",
            #[inline(always)]
            |_, vec, (other, k)| Ok(vec.lerp(other, k)),
        );
        methods.add_method(
            "ceil",
            #[inline(always)]
            |_, vec, ()| Ok(vec.ceil()),
        );
        methods.add_method(
            "max",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| Ok(vec.max(other)),
        );
        methods.add_method(
            "min",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| Ok(vec.min(other)),
        );
        methods.add_method(
            "sign",
            #[inline(always)]
            |_, vec, ()| Ok(vec.sign()),
        );
        let simplex = Simplex::new(noise::Simplex::DEFAULT_SEED);
        methods.add_method(
            "noise",
            #[inline(always)]
            move |_, vec, ()| {
                let f64vec = [vec.0[0] as f64, vec.0[1] as f64];
                Ok(simplex.get(f64vec))
            },
        );
        let worley = Worley::new(noise::Worley::DEFAULT_SEED);
        methods.add_method(
            "worleyNoise",
            #[inline(always)]
            move |_, vec, ()| {
                let f64vec = [vec.0[0] as f64, vec.0[1] as f64];
                Ok(worley.get(f64vec))
            },
        );
        methods.add_method(
            "distance",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| Ok((*vec - other).length()),
        );
        methods.add_meta_function(
            mlua::MetaMethod::Add,
            #[inline(always)]
            |_, (vec, other): (Vec2, Vec2)| Ok(vec + other),
        );
        methods.add_meta_function(
            mlua::MetaMethod::Sub,
            #[inline(always)]
            |_, (vec, other): (Vec2, Vec2)| Ok(vec - other),
        );
        methods.add_meta_function(
            mlua::MetaMethod::Mul,
            #[inline(always)]
            |_, (vec, other): (Vec2, Vec2)| Ok(vec * other),
        );
        methods.add_meta_method(
            mlua::MetaMethod::ToString,
            #[inline(always)]
            |_, vec, _any: mlua::Value| Ok(format!("V2({}, {})", vec.0[0], vec.0[1])),
        );

        // In-place methods
        methods.add_method_mut(
            "add",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| {
                vec.0[0] += other.0[0];
                vec.0[1] += other.0[1];
                Ok(())
            },
        );
        methods.add_method_mut(
            "sub",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| {
                vec.0[0] -= other.0[0];
                vec.0[1] -= other.0[1];
                Ok(())
            },
        );
        methods.add_method_mut(
            "mul",
            #[inline(always)]
            |_, vec, (other,): (Vec2,)| {
                vec.0[0] *= other.0[0];
                vec.0[1] *= other.0[1];
                Ok(())
            },
        );
        methods.add_method_mut(
            "rescale",
            #[inline(always)]
            |_, vec, (k,): (f32,)| {
                vec.0[0] *= k;
                vec.0[1] *= k;
                Ok(())
            },
        );
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
