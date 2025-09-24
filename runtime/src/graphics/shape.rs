use crate::lua_env::lua_vec2::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Quad {
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
    pub p4: Vec2,
}
