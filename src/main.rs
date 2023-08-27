extern crate clap;
extern crate glium;

use std::collections::HashMap;
use std::io::Error;

use clap::Parser;
use glium::index::PrimitiveType;
use glium::uniforms::{AsUniformValue, Uniforms};
use glium::VertexBuffer;
use glium::{implement_vertex, Surface};
use log::warn;
use serde_json::{from_str, Value};
use simple_logger::SimpleLogger;

use gltr::gl::shader::{to_fragment_shader_source, VERTEX_SHADER_SOURCE};
use gltr::gl::texture::create_from;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    from: String,

    #[arg(short, long)]
    to: String,

    #[arg(short, long)]
    shader: String,

    #[arg(short, long)]
    mode: String,

    #[arg(short, long)]
    progress: f64,

    #[arg(long)]
    sampler2d: Option<String>,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

struct DynamicUniforms {
    data: HashMap<String, Box<dyn AsUniformValue>>,
}

impl DynamicUniforms {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn add<T: 'static + AsUniformValue>(&mut self, name: String, value: T) {
        self.data.insert(name, Box::new(value));
    }
}

impl Uniforms for DynamicUniforms {
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(
        &'a self,
        mut output: F,
    ) {
        for (key, value) in &self.data {
            output(key, value.as_uniform_value());
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let args = Args::parse();

    let wb = glium::glutin::window::WindowBuilder::new()
        .with_visible(false)
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(540, 960))
        .with_title("Hello world");

    let cb = glium::glutin::ContextBuilder::new().with_srgb(true);
    let events_loop = glium::glutin::event_loop::EventLoop::new();

    let display = match glium::Display::new(wb, cb, &events_loop) {
        Ok(display) => display,
        Err(err) => {
            warn!("Could not create display: {}", err);
            return;
        }
    };

    let (texture1, dimensions1) = match create_from(args.from, &display) {
        Ok((texture, dim)) => (texture, dim),
        Err(err) => {
            warn!("Could not load from image: {}", err);
            return;
        }
    };
    let (texture2, dimensions2) = match create_from(args.to, &display) {
        Ok((texture, dim)) => (texture, dim),
        Err(err) => {
            warn!("Could not load to image: {}", err);
            return;
        }
    };

    let shader = match load_shader(&args.shader) {
        Ok(shader) => shader,
        Err(err) => {
            warn!("Could not load shader: {}", err);
            return;
        }
    };

    let mut uniforms = DynamicUniforms::new();
    uniforms.add("from".to_string(), texture1);
    uniforms.add("to".to_string(), texture2);
    uniforms.add("progress".to_string(), args.progress as f32);
    uniforms.add(
        "ratio".to_string(),
        dimensions1.0 as f32 / dimensions1.1 as f32,
    );
    uniforms.add(
        "_fromR".to_string(),
        dimensions1.0 as f32 / dimensions1.1 as f32,
    );
    uniforms.add(
        "_toR".to_string(),
        dimensions2.0 as f32 / dimensions2.1 as f32,
    );

    if let Some(params) = shader["defaultParams"].as_object() {
        for (key, value) in params {
            match shader["paramsTypes"][key].as_str() {
                Some("bool") => {
                    let val: bool = match value.as_bool() {
                        Some(val) => val,
                        None => {
                            warn!("Could not parse defaultParams bool: {} {}", key, value);
                            return;
                        }
                    };
                    uniforms.add(key.to_string(), val);
                }
                Some("float") => {
                    let val: f32 = match value.as_f64() {
                        Some(val) => val as f32,
                        None => {
                            warn!("Could not parse defaultParams float: {} {}", key, value);
                            return;
                        }
                    };
                    uniforms.add(key.to_string(), val);
                }
                Some("int") => {
                    let val: i32 = match value.as_i64() {
                        Some(val) => val as i32,
                        None => {
                            warn!("Could not parse defaultParams int: {} {}", key, value);
                            return;
                        }
                    };
                    uniforms.add(key.to_string(), val);
                }
                Some("ivec2") => {
                    let arr: Vec<i32> = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_i64().unwrap() as i32)
                        .collect();
                    uniforms.add(key.to_string(), [arr[0], arr[1]]);
                }
                Some("vec2") => {
                    let arr: Vec<f32> = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_f64().unwrap() as f32)
                        .collect();
                    uniforms.add(key.to_string(), [arr[0], arr[1]]);
                }
                Some("vec3") => {
                    let arr: Vec<f32> = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_f64().unwrap() as f32)
                        .collect();
                    uniforms.add(key.to_string(), [arr[0], arr[1], arr[2]]);
                }
                Some("vec4") => {
                    let arr: Vec<f32> = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_f64().unwrap() as f32)
                        .collect();
                    uniforms.add(key.to_string(), [arr[0], arr[1], arr[2], arr[3]]);
                }
                Some("sampler2D") => {
                    if let Some(file_path) = args.sampler2d.clone() {
                        let texture = match create_from(file_path, &display) {
                            Ok((texture, _)) => texture,
                            Err(err) => {
                                warn!("Could not load texture: {}", err);
                                return;
                            }
                        };
                        uniforms.add(key.to_string(), texture);
                    } else {
                        warn!("No sampler2D file path provided");
                        return;
                    };
                }
                _ => {}
            }
        }
    }

    let mode = match gltr::gl::shader::Mode::new(&args.mode) {
        Ok(mode) => mode,
        Err(err) => {
            warn!("Could not create mode: {}", err);
            return;
        }
    };

    let shader_code = match serde_json::from_value::<String>(shader["glsl"].clone()) {
        Ok(code) => code,
        Err(err) => {
            warn!("Could not get shader code: {}", err);
            return;
        }
    };

    let fragment_shader_src = match to_fragment_shader_source(mode, &shader_code) {
        Ok(src) => src,
        Err(err) => {
            warn!("Could not create fragment shader: {}", err);
            return;
        }
    };

    let program =
        glium::Program::from_source(&display, VERTEX_SHADER_SOURCE, &fragment_shader_src, None)
            .unwrap();

    // Create a full-screen quad
    let vertex1 = Vertex {
        position: [-1.0, -1.0],
    };
    let vertex2 = Vertex {
        position: [1.0, -1.0],
    };
    let vertex3 = Vertex {
        position: [-1.0, 1.0],
    };
    let vertex4 = Vertex {
        position: [1.0, 1.0],
    };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(PrimitiveType::TriangleStrip);

    // Draw the frame
    let mut frame = display.draw();
    frame.clear_color(0.0, 0.0, 0.0, 1.0);

    // Draw the textures
    frame
        .draw(
            &vertex_buffer,
            indices,
            &program,
            &uniforms,
            &Default::default(),
        )
        .unwrap();

    frame.finish().unwrap();

    let image: glium::texture::RawImage2d<u8> = display.read_front_buffer().unwrap();
    let image =
        image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image).flipv();
    image.save("output.png").unwrap();
}

fn load_shader(target: &str) -> Result<Value, Error> {
    let shaders: Vec<Value> = match from_str(include_str!("../glsl/shaders.json")) {
        Ok(data) => data,
        Err(err) => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Could not parse shaders.json: {}", err),
            ));
        }
    };
    let shader = match shaders.iter().find(|shader| {
        match serde_json::from_value::<String>(shader["name"].clone()) {
            Ok(name) => name,
            _ => "".to_string(),
        }
        .eq(target.to_lowercase().as_str())
    }) {
        Some(shader) => shader,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Could not find shader: {}", target),
            ));
        }
    };

    Ok(shader.clone())
}
