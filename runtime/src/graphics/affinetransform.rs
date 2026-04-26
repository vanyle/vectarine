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

    /// The translation is applied first, then the scale and finally the rotation.
    pub fn new(translation: Vec2, scale: Vec2, rotation: f32) -> Self {
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        let a = cos_r * scale.x();
        let b = sin_r * scale.x();
        let c = -sin_r * scale.y();
        let d = cos_r * scale.y();
        Self {
            a,
            b,
            c,
            d,
            tx: a * translation.x() + c * translation.y(),
            ty: b * translation.x() + d * translation.y(),
        }
    }

    pub fn apply(&self, v: &Vec2) -> Vec2 {
        Vec2::new(
            self.a * v.x() + self.c * v.y() + self.tx,
            self.b * v.x() + self.d * v.y() + self.ty,
        )
    }

    /// Returns the translation component of the affine transform.
    pub fn translation(&self) -> Vec2 {
        let det = self.a * self.d - self.b * self.c;
        if det == 0.0 {
            return Vec2::new(0.0, 0.0);
        }
        Vec2::new(
            (self.d * self.tx - self.c * self.ty) / det,
            (-self.b * self.tx + self.a * self.ty) / det,
        )
    }

    /// Returns the scale component of the affine transform.
    pub fn scale(&self) -> Vec2 {
        Vec2::new(
            (self.a.powi(2) + self.b.powi(2)).sqrt(),
            (self.c.powi(2) + self.d.powi(2)).sqrt(),
        )
    }

    /// Returns the rotation component of the affine transform in radians.
    pub fn rotation(&self) -> f32 {
        self.b.atan2(self.a)
    }

    pub fn inverse_apply(&self, v: &Vec2) -> Vec2 {
        let det = self.a * self.d - self.b * self.c;
        if det == 0.0 {
            return Vec2::new(0.0, 0.0);
        }
        let inv_a = self.d / det;
        let inv_b = -self.b / det;
        let inv_c = -self.c / det;
        let inv_d = self.a / det;
        let inv_tx = (self.c * self.ty - self.d * self.tx) / det;
        let inv_ty = (self.b * self.tx - self.a * self.ty) / det;

        Vec2::new(
            inv_a * v.x() + inv_c * v.y() + inv_tx,
            inv_b * v.x() + inv_d * v.y() + inv_ty,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    fn vec2_approx_eq(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x(), b.x()) && approx_eq(a.y(), b.y())
    }

    fn assert_vec2_approx_eq(a: Vec2, b: Vec2) {
        assert!(
            vec2_approx_eq(a, b),
            "Expected {:?} to be approximately equal to {:?}",
            a,
            b
        );
    }

    #[test]
    fn new_recovers_components() {
        let translation = Vec2::new(3.0, 4.0);
        let scale = Vec2::new(2.0, 3.0);
        let rotation = PI / 4.0;
        let t = AffineTransform::new(translation, scale, rotation);

        assert_vec2_approx_eq(t.translation(), translation);
        assert_vec2_approx_eq(t.scale(), scale);
        assert!(approx_eq(t.rotation(), rotation));
    }

    #[test]
    fn apply() {
        let t = AffineTransform::new(Vec2::new(1.0, 2.0), Vec2::new(2.0, 1.0), 0.0);

        assert_vec2_approx_eq(t.apply(&Vec2::new(0.0, 0.0)), Vec2::new(2.0, 2.0));
        assert_vec2_approx_eq(t.apply(&Vec2::new(1.0, 1.0)), Vec2::new(4.0, 3.0));
    }

    #[test]
    fn combine() {
        let t1 = AffineTransform::new(Vec2::new(0.0, 1.0), Vec2::new(4.0, 2.0), 3.0);
        let t2 = AffineTransform::new(Vec2::new(1.0, 0.0), Vec2::new(1.0, -1.0), 1.0);

        // (t1 combine t2).apply(v) == t1.apply(t2.apply(v))
        let combined = t1.combine(&t2);

        let v1 = Vec2::new(3.0, 4.0);
        let v2 = Vec2::new(0.0, 0.0);

        assert_vec2_approx_eq(combined.apply(&v1), t1.apply(&t2.apply(&v1)));
        assert_vec2_approx_eq(combined.apply(&v2), t1.apply(&t2.apply(&v2)));
    }
}
