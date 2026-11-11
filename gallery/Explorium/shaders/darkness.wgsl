// The darkness of the explorium, drawn over the world (see scripts/lighting.luau and world_view.luau).
// Each pixel is turned back into a position in the world (in tiles), and darkened according to the light
// that reaches it. lighting.lightAt computes the same light in Lua, for gameplay: keep them in sync.
const MAX_LIGHTS = 10;

struct Uniforms {
    camera: vec2f,        // top-left corner of the view, in tiles
    visible: vec2f,       // size of the view, in tiles
    darkness: f32,        // 0 = bright, 1 = pitch black
    shadowColor: vec4f,
    halo: vec4f,          // the player's light: x, y, radius, 1 if the flashlight is on
    facing: vec2f,        // where the flashlight points
    lightCount: f32,
    lights: array<vec4f, MAX_LIGHTS>, // x, y, radius, unused
    haze: f32,            // sandstorm: 0 = clear, 1 = thick sand
    hazeColor: vec4f,
    hazeRadius: f32,      // how far the player sees through the sand, in tiles
    maxDark: f32,         // the darkest the dark gets: 0.88 (shapes stay visible), 1 in a lightless place
}
@group(1) @binding(0) var<uniform> u: Uniforms;

fn falloff(dist: f32, radius: f32) -> f32 {
    let t = dist / radius;
    if (t >= 1.0) {
        return 0.0;
    }
    return 1.0 - t * t;
}

@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
    // uv goes up, the world goes down. The light is snapped to the pixels of the sprites (16 per tile).
    var world = u.camera + vec2f(uv.x, 1.0 - uv.y) * u.visible;
    world = (floor(world * 16.0) + 0.5) / 16.0;

    var light = 0.0;
    for (var i = 0; i < MAX_LIGHTS; i++) {
        if (f32(i) >= u.lightCount) {
            break;
        }
        light = max(light, falloff(length(world - u.lights[i].xy), u.lights[i].z));
    }
    let offset = world - u.halo.xy;
    let dist = length(offset);
    light = max(light, falloff(dist, u.halo.z));
    // The flashlight: a cone in front of the player
    if (u.halo.w > 0.5 && dist > 0.01) {
        let alignment = dot(offset, u.facing) / dist;
        let cone = falloff(dist, 6.5) * clamp((alignment - 0.8) / 0.1, 0.0, 1.0);
        light = max(light, cone * 0.95);
    }
    // Never completely black (shapes stay visible in the dark), except in the black of a lightless place
    let alpha = min(u.maxDark, u.darkness * (1.0 - light));
    // The sand of a storm hides everything beyond a few steps from the player, lights included
    let sand = u.haze * (1.0 - falloff(dist, u.hazeRadius)) * 0.94;
    let total = 1.0 - (1.0 - alpha) * (1.0 - sand);
    let color = (u.shadowColor.rgb * alpha * (1.0 - sand) + u.hazeColor.rgb * sand) / max(total, 0.001);
    return vec4f(color, total);
}
