@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    iTime: f32,
}
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
    // We can for example use a shader for deformations
    let time = uniforms.iTime;
    let offset = vec2f(cos((uv.y * 4.0 + time) * 10.0), sin((uv.x * 4.0 + time) * 10.0)) / 100.0;
    return textureSample(tex, tex_sampler, uv + offset);
}
