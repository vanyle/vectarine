pub enum DrawInstruction {
    Rectangle { x: f32, y: f32, w: f32, h: f32 },
    Circle { x: f32, y: f32, radius: f32 },
    SetColor { r: u8, g: u8, b: u8 },
    Clear,
}
