in vec2 uv;
uniform sampler2D tex;
uniform float iTime;
out vec4 frag_color;

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    // We can for example use a shader for deformations
    fragColor = texture(tex, vec2(uv.x + cos((uv.y*4+iTime)*10)/100, uv.y + sin((uv.x*4+iTime)*10)/100));
}

void main() {
    mainImage(frag_color, uv);
}

