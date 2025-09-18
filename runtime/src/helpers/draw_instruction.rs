use crate::helpers::lua_env::vec2::Vec2;

#[derive(Debug, Clone)]
pub enum DrawInstruction {
    Rectangle {
        pos: Vec2,
        size: Vec2,
        color: [f32; 4],
    },
    Polygon {
        points: Vec<Vec2>,
        color: [f32; 4],
    },
    Circle {
        pos: Vec2,
        radius: f32,
        color: [f32; 4],
    },
    Image {
        pos: Vec2,
        size: Vec2,
        resource_id: u32,
    },
    ImagePart {
        p1: Vec2,
        p2: Vec2,
        p3: Vec2,
        p4: Vec2,
        resource_id: u32,
        uv_pos: Vec2,
        uv_size: Vec2,
    },
    Text {
        pos: Vec2,
        text: String,
        color: [f32; 4],
        font_size: f32,
        font_resource_id: u32,
    },
    Clear {
        color: [f32; 4],
    },
}
