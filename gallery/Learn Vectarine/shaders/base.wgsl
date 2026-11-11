// A shader computes the color of every pixel of a canvas when the canvas is drawn.
// `uv` is the position of the pixel on the canvas: (0, 0) is its bottom left, (1, 1) its top right.

@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
    // Generate a red-green gradient
    return vec4f(uv.x, uv.y, 0.0, 1.0);
}
