/// The shaders used to draw shapes, images and text, and the vertex shader shared by the custom shaders of games.
pub const BUILTIN_SHADERS: &str = r#"
struct ColorVertex {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs_color(@location(0) position: vec2f, @location(1) color: vec4f) -> ColorVertex {
    return ColorVertex(vec4f(position, 0.0, 1.0), color);
}

@fragment
fn fs_color(vertex: ColorVertex) -> @location(0) vec4f {
    return vertex.color;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct TextureVertex {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) color: vec4f,
}

@vertex
fn vs_texture(@location(0) position: vec2f, @location(1) uv: vec2f, @location(2) color: vec4f) -> TextureVertex {
    return TextureVertex(vec4f(position, 0.0, 1.0), uv, color);
}

@fragment
fn fs_texture(vertex: TextureVertex) -> @location(0) vec4f {
    return textureSample(tex, tex_sampler, vertex.uv) * vertex.color;
}

@fragment
fn fs_font(vertex: TextureVertex) -> @location(0) vec4f {
    let coverage = textureSample(tex, tex_sampler, vertex.uv).r;
    if (coverage < 0.01) {
        discard;
    }
    return vec4f(vertex.color.rgb, coverage * vertex.color.a);
}

struct CustomVertex {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
}

@vertex
fn vs_custom(@location(0) position: vec2f, @location(1) uv: vec2f) -> CustomVertex {
    return CustomVertex(vec4f(position, 0.0, 1.0), uv);
}
"#;
