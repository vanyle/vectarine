pub const COLOR_VERTEX_SHADER_SOURCE: &str = r#"
    layout (location = 0) in vec2 in_vert;
    layout (location = 1) in vec4 in_color;
    out vec4 color;
    void main() {
        color = in_color;
        gl_Position = vec4(in_vert.xy, 0.0, 1.0);
    }"#;

pub const COLOR_FRAG_SHADER_SOURCE: &str = r#"precision mediump float;
    in vec4 color;
    out vec4 frag_color;
    void main() {
        frag_color = color;
    }"#;

pub const TEX_VERTEX_SHADER_SOURCE: &str = r#"
    layout (location = 0) in vec2 in_vert;
    layout (location = 1) in vec2 in_uv;
    out vec2 uv;
    void main() {
        uv = in_uv;
        gl_Position = vec4(in_vert.xy, 0.0, 1.0);
    }"#;

pub const TEX_FRAG_SHADER_SOURCE: &str = r#"precision mediump float;
    in vec2 uv;
    uniform sampler2D tex;
    uniform vec4 tint_color;
    out vec4 frag_color;
    void main() {
        frag_color = texture(tex, uv) * tint_color;
    }"#;

pub const FONT_VERTEX_SHADER_SOURCE: &str = r#"
    layout (location = 0) in vec2 in_vert;
    layout (location = 1) in vec2 in_uv;
    out vec2 uv;
    void main() {
        uv = in_uv;
        gl_Position = vec4(in_vert.xy, 0.0, 1.0);
    }
"#;

pub const FONT_FRAG_SHADER_SOURCE: &str = r#"precision mediump float;
    in vec2 uv;
    uniform sampler2D tex;
    uniform vec4 text_color;
    out vec4 frag_color;
    void main() {
        float r = texture(tex, uv).r;
        if (r < 0.01) {
            discard;
        }
        frag_color = vec4(text_color.rgb, r * text_color.a);
    }"#;
