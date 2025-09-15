#[derive(Debug, Clone)]
pub enum DrawInstruction {
    Rectangle {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        color: [f32; 4],
    },
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        resource_id: u32,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        color: [f32; 4],
        font_size: f32,
        font_resource_id: u32,
    },
    Clear {
        color: [f32; 4],
    },
}
