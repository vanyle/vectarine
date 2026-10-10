// The darkness of the explorium, drawn over the world (see scripts/lighting.luau and world_view.luau).
// Each pixel is turned back into a position in the world (in tiles), and darkened according to the light
// that reaches it. lighting.lightAt computes the same light in Lua, for gameplay: keep them in sync.
precision highp float;
in vec2 uv;
uniform sampler2D tex;

#define MAX_LIGHTS 10

uniform vec2 camera;        // top-left corner of the view, in tiles
uniform vec2 visible;       // size of the view, in tiles
uniform float darkness;     // 0 = bright, 1 = pitch black
uniform vec4 shadowColor;
uniform vec4 halo;          // the player's light: x, y, radius, 1 if the flashlight is on
uniform vec2 facing;        // where the flashlight points
uniform float lightCount;
uniform vec4 lights[MAX_LIGHTS]; // x, y, radius, unused
uniform float haze;         // sandstorm: 0 = clear, 1 = thick sand
uniform vec4 hazeColor;
uniform float hazeRadius;   // how far the player sees through the sand, in tiles
out vec4 frag_color;

float falloff(float distance, float radius) {
    float t = distance / radius;
    return t >= 1.0 ? 0.0 : 1.0 - t * t;
}

void main() {
    // uv goes up, the world goes down. The light is snapped to the pixels of the sprites (16 per tile).
    vec2 world = camera + vec2(uv.x, 1.0 - uv.y) * visible;
    world = (floor(world * 16.0) + 0.5) / 16.0;

    float light = 0.0;
    for (int i = 0; i < MAX_LIGHTS; i++) {
        if (float(i) >= lightCount) {
            break;
        }
        light = max(light, falloff(length(world - lights[i].xy), lights[i].z));
    }
    vec2 offset = world - halo.xy;
    float distance = length(offset);
    light = max(light, falloff(distance, halo.z));
    // The flashlight: a cone in front of the player
    if (halo.w > 0.5 && distance > 0.01) {
        float alignment = dot(offset, facing) / distance;
        float cone = falloff(distance, 6.5) * clamp((alignment - 0.8) / 0.1, 0.0, 1.0);
        light = max(light, cone * 0.95);
    }
    // Never completely black: shapes stay visible in the dark
    float alpha = min(0.88, darkness * (1.0 - light));
    // The sand of a storm hides everything beyond a few steps from the player, lights included
    float sand = haze * (1.0 - falloff(distance, hazeRadius)) * 0.94;
    float total = 1.0 - (1.0 - alpha) * (1.0 - sand);
    vec3 color = (shadowColor.rgb * alpha * (1.0 - sand) + hazeColor.rgb * sand) / max(total, 0.001);
    frag_color = vec4(color, total);
}
