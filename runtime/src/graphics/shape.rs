use crate::lua_env::lua_vec2::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Quad {
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
    pub p4: Vec2,
}

/// Positive if C is to the left of AB.
#[inline]
fn cross_2d(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b - a).cross(c - a)
}

impl Quad {
    pub fn inside(&self, p: Vec2) -> bool {
        let cross1 = cross_2d(self.p1, self.p2, p);
        let cross2 = cross_2d(self.p2, self.p3, p);
        let cross3 = cross_2d(self.p3, self.p4, p);
        let cross4 = cross_2d(self.p4, self.p1, p);
        (cross1 >= 0.0 && cross2 >= 0.0 && cross3 >= 0.0 && cross4 >= 0.0)
            || (cross1 <= 0.0 && cross2 <= 0.0 && cross3 <= 0.0 && cross4 <= 0.0)
    }
}
