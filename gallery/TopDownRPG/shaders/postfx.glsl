// Post-processing applied to the whole screen (see scripts/postfx.luau).
// - aberration: splits the red and blue channels, for impacts and rewinds
// - flash: tints the whole screen with a color, for explosions and deaths
// - desaturate: removes the colors, when the player dies
// - iris: a circle that closes and opens around a point, for transitions
precision mediump float;
in vec2 uv;
uniform sampler2D tex;
uniform float aberration;
uniform vec4 flash;        // rgb = color, a = strength
uniform float desaturate;
uniform float iris;        // 1 = fully open, 0 = closed
uniform vec2 irisPos;      // center of the iris, in uv coordinates
uniform vec2 resolution;   // size of the screen in pixels
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

    float aspect = resolution.x / resolution.y;
    float radius = iris * 1.2 * max(aspect, 1.0);
    float distance = length((uv - irisPos) * vec2(aspect, 1.0));
    color *= smoothstep(radius, radius - 0.01, distance);

    frag_color = vec4(color, 1.0);
}
