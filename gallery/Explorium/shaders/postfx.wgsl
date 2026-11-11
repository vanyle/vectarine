// Post-processing applied to the whole screen (see scripts/postfx.luau).
// - aberration: splits the red and blue channels, for impacts and rewinds
// - flash: tints the whole screen with a color, for explosions and deaths
// - desaturate: removes the colors, when the player dies
// - iris: a circle that closes and opens around a point, for transitions
// - danger, thirst, cold: the edges of the screen turn red when health is low, yellow when water is low, frosty
//   white-blue when the player is freezing
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    aberration: f32,
    flash: vec4f,        // rgb = color, a = strength
    desaturate: f32,
    iris: f32,           // 1 = fully open, 0 = closed
    irisPos: vec2f,      // center of the iris, in uv coordinates
    resolution: vec2f,   // size of the screen in pixels
    danger: vec4f,       // rgb = tint (multiplied with the colors) of the edges when health is low, a = strength
    thirst: vec4f,       // rgb = tint of the edges when water is low, a = strength
    cold: vec4f,         // rgb = color of the frost on the edges when the player is cold, a = strength
}
@group(1) @binding(0) var<uniform> u: Uniforms;

@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
    let shift = u.aberration * 0.008;
    var color = vec3f(
        textureSample(tex, tex_sampler, uv + vec2f(shift, 0.0)).r,
        textureSample(tex, tex_sampler, uv).g,
        textureSample(tex, tex_sampler, uv - vec2f(shift, 0.0)).b,
    );

    let gray = dot(color, vec3f(0.299, 0.587, 0.114));
    color = mix(color, vec3f(gray) * vec3f(0.9, 0.85, 1.0), u.desaturate);
    color = mix(color, u.flash.rgb, u.flash.a);

    // Edges of the screen: a soft band along the 4 sides, rounded in the corners
    let q = abs(uv - 0.5) * 2.0;
    let edge = smoothstep(0.55, 1.05, pow(pow(q.x, 4.0) + pow(q.y, 4.0), 0.25));
    color = mix(color, color * u.thirst.rgb, edge * u.thirst.a);
    // Frost creeps in from the edges: it lightens and blues the colors
    color = mix(color, u.cold.rgb * (0.55 + 0.45 * gray), edge * u.cold.a);
    color = mix(color, color * u.danger.rgb + vec3f(0.06, 0.0, 0.0), edge * u.danger.a);

    let aspect = u.resolution.x / u.resolution.y;
    let radius = u.iris * 1.2 * max(aspect, 1.0);
    let dist = length((uv - u.irisPos) * vec2f(aspect, 1.0));
    color *= 1.0 - smoothstep(radius - 0.01, radius, dist);

    return vec4f(color, 1.0);
}
