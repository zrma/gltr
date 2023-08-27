use std::io::Error;

pub const VERTEX_SHADER_SOURCE: &str = r#"
    #version 140
    in vec2 position;
    out vec2 uv;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        uv = vec2(0.5, 0.5) * (position + vec2(1.0, 1.0)); // Modify UV calculation
    }
"#;

pub enum Mode {
    Contain,
    Stretch,
    Cover,
}

impl Mode {
    pub fn new(mode: &str) -> Result<Self, Error> {
        match mode {
            "contain" => Ok(Mode::Contain),
            "stretch" => Ok(Mode::Stretch),
            "cover" => Ok(Mode::Cover),
            _ => Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid transition mode: {}", mode),
            )),
        }
    }
}

pub fn to_fragment_shader_source(mode: Mode, shader_body: &str) -> Result<String, Error> {
    match mode {
        Mode::Contain => Ok(format!(
            r#"
        #version 140
        uniform sampler2D from, to;
        uniform float progress;
        uniform float ratio, _fromR, _toR;
        in vec2 uv;
        out vec4 color;

        vec4 getFromColor(vec2 uv) {{
            return texture(from, .5 + (uv - .5) * vec2(max(ratio / _fromR, 1.), max(_fromR / ratio, 1.)));
        }}

        vec4 getToColor(vec2 uv) {{
            return texture(to, .5 + (uv - .5) * vec2(max(ratio / _toR, 1.), max(_toR / ratio, 1.)));
        }}

        {}

        void main() {{
            color = transition(uv);
        }}
    "#,
            shader_body
        )),

        Mode::Stretch => Ok(format!(
            r#"
        #version 140
        uniform sampler2D from, to;
        uniform float progress;
        in vec2 uv;
        out vec4 color;

        vec4 getFromColor(vec2 uv) {{
            return texture(from, uv);
        }}

        vec4 getToColor(vec2 uv) {{
            return texture(to, uv);
        }}

        {}

        void main() {{
            color = transition(uv);
        }}
    "#,
            shader_body
        )),

        Mode::Cover => Ok(format!(
            r#"
        #version 140
        uniform sampler2D from, to;
        uniform float progress;
        uniform float ratio, _fromR, _toR;
        in vec2 uv;
        out vec4 color;

        vec4 getFromColor(vec2 uv) {{
            return texture(from, .5 + (uv - .5) * vec2(min(ratio / _fromR, 1.), min(_fromR / ratio, 1.)));
        }}

        vec4 getToColor(vec2 uv) {{
            return texture(to, .5 + (uv - .5) * vec2(min(ratio / _toR, 1.), min(_toR / ratio, 1.)));
        }}

        {}

        void main() {{
            color = transition(uv);
        }}
    "#,
            shader_body
        )),
    }
}
