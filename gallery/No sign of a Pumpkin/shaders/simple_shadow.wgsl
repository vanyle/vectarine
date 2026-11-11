@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    lightZone: f32, // size of the lighted area
    lightPosX: f32,
    lightPosY: f32,
    windowWidth: f32,
    windowHeight: f32,
}
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

fn calculateLightFalloff(closeness: f32, lightZoneSize: f32) -> f32 {
    return (1.1 * atan((closeness - (1.2 - lightZoneSize)) * 12.0) / 3.14159265) + 0.5;
}

@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
    let windowSize = vec2f(uniforms.windowWidth, uniforms.windowHeight);
    // Convert to pixel coordinates for window-independent calculation
    let pixelCoord = uv * windowSize;
    let lightPixelPos = vec2f(uniforms.lightPosX + 0.5, uniforms.lightPosY + 0.5) * windowSize;

    // Calculate distance in pixels, then normalize to fixed radius
    let pixelDistance = distance(pixelCoord, lightPixelPos);
    let normalizedDistance = pixelDistance / (uniforms.lightZone * 700.0); // Fixed pixel radius
    let closenessToCenter = max(0.0, 1.0 - normalizedDistance);

    let lightMultiplier = calculateLightFalloff(closenessToCenter, uniforms.lightZone);
    return textureSample(tex, tex_sampler, uv) * vec4f(lightMultiplier, lightMultiplier, lightMultiplier, 1.0);
}
