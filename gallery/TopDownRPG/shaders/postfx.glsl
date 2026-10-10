// Post-processing applied to the whole screen (see scripts/postfx.luau).
// - aberration: splits the red and blue channels, for impacts and rewinds
// - flash: tints the whole screen with a color, for explosions and deaths
// - desaturate: removes the colors, when the player dies
// - iris: a circle that closes and opens around a point, for transitions
// - danger, thirst, cold: the edges of the screen turn red when health is low, yellow when water is low, frosty
//   white-blue when the player is freezing
precision mediump float;
in vec2 uv;
uniform sampler2D tex;
uniform float aberration;
uniform vec4 flash;        // rgb = color, a = strength
uniform float desaturate;
uniform float iris;        // 1 = fully open, 0 = closed
uniform vec2 irisPos;      // center of the iris, in uv coordinates
uniform vec2 resolution;   // size of the screen in pixels
uniform vec4 danger;       // rgb = tint (multiplied with the colors) of the edges when health is low, a = strength
uniform vec4 thirst;       // rgb = tint of the edges when water is low, a = strength
uniform vec4 cold;         // rgb = color of the frost on the edges when the player is cold, a = strength
out vec4 frag_color;

void main() {
    float shift = aberration * 0.008;
    vec3 color = vec3(
        texture(tex, uv + vec2(shift, 0.0)).r,
        texture(tex, uv).g,
        texture(tex, uv - vec2(shift, 0.0)).b
    );

    float gray = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(color, vec3(gray) * vec3(0.9, 0.85, 1.0), desaturate);
    color = mix(color, flash.rgb, flash.a);

    // Edges of the screen: a soft band along the 4 sides, rounded in the corners
    vec2 q = abs(uv - 0.5) * 2.0;
    float edge = smoothstep(0.55, 1.05, pow(pow(q.x, 4.0) + pow(q.y, 4.0), 0.25));
    color = mix(color, color * thirst.rgb, edge * thirst.a);
    // Frost creeps in from the edges: it lightens and blues the colors
    color = mix(color, cold.rgb * (0.55 + 0.45 * gray), edge * cold.a);
    color = mix(color, color * danger.rgb + vec3(0.06, 0.0, 0.0), edge * danger.a);

    float aspect = resolution.x / resolution.y;
    float radius = iris * 1.2 * max(aspect, 1.0);
    float distance = length((uv - irisPos) * vec2(aspect, 1.0));
    color *= smoothstep(radius, radius - 0.01, distance);

    frag_color = vec4(color, 1.0);
}
