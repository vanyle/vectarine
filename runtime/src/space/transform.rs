use std::ops;

use crate::lua_env::lua_vec2::Vec2;

// TODO: generalize everything to 3D

#[derive(PartialEq, Clone, Copy)]
pub struct Transform2 {
    position: Vec2,
    rotation: f32,
}

impl Transform2 {
    pub fn position(self) -> Vec2 {
        self.position
    }

    pub fn rotation(self) -> f32 {
        self.rotation
    }

    #[inline(always)]
    pub const fn new(position: Vec2, rotation: f32) -> Self {
        Self { position, rotation }
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Self {
            position: Vec2::zero(),
            rotation: 0.0,
        }
    }

    #[inline]
    pub fn apply(&self, pos: Vec2) -> Vec2 {
        (pos + self.position).rotated(self.rotation)
    }

    #[inline]
    pub fn applied(&self) -> Vec2 {
        self.apply(Vec2::zero())
    }
}

// applying (t1 + t2) is the same as applying t1 then t2
// (t1 + t2) + t3 == t1 + (t2 + t3) but it is not commutative
impl ops::Add for Transform2 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            position: (self.position + (other.position.rotated(-self.rotation))),
            rotation: other.rotation + self.rotation,
        }
    }
}

// applying (t1 - t1) is the same as the identity
// (t1 + t2) - t1 == t1 + (t2 - t1)
// but in general t1 + t2 - t1 != t2
impl ops::Sub for Transform2 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        self + Self {
            position: other.position.scale(-1.0).rotated(other.rotation),
            rotation: -other.rotation,
        }
    }
}

impl ops::Neg for Transform2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Transform2::zero() - self
    }
}
