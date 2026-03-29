use crate::{graphics::shape::Quad, lua_env::lua_vec2::Vec2};

// maps (x,y) -> (a*x + c*y + tx, b*x + d*y + ty)
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AffineTransform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    tx: f32,
    ty: f32,
}

impl AffineTransform {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
    pub fn new(translation: Vec2, scale: Vec2, rotation: f32) -> Self {
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        Self {
            a: cos_r * scale.x(),
            b: sin_r * scale.x(),
            c: -sin_r * scale.y(),
            d: cos_r * scale.y(),
            tx: translation.x(),
            ty: translation.y(),
        }
    }

    pub fn apply(&self, v: &Vec2) -> Vec2 {
        Vec2::new(
            self.a * v.x() + self.c * v.y() + self.tx,
            self.b * v.x() + self.d * v.y() + self.ty,
        )
    }

    pub fn apply_quad(&self, quad: &Quad) -> Quad {
        Quad {
            p1: self.apply(&quad.p1),
            p2: self.apply(&quad.p2),
            p3: self.apply(&quad.p3),
            p4: self.apply(&quad.p4),
        }
    }

    pub fn combine(&self, other: &AffineTransform) -> AffineTransform {
        AffineTransform {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            tx: self.a * other.tx + self.c * other.ty + self.tx,
            ty: self.b * other.tx + self.d * other.ty + self.ty,
        }
    }
}
