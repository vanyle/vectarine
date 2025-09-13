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
    Clear {
        color: [f32; 4],
    },
}
