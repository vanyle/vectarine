use crate::helpers::lua_env::vec2::Vec2;

#[derive(Debug, Clone)]
pub enum DrawInstruction {
    Rectangle {
        pos: Vec2,
        size: Vec2,
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
