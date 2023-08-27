extern crate clap;
extern crate glium;

use clap::Parser;
use glium::index::PrimitiveType;
use glium::VertexBuffer;
use glium::{implement_vertex, uniform, Surface};
use log::warn;
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
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

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

    let shader_path = if args.shader.ends_with(".glsl") {
        args.shader
    } else {
        format!("{}.glsl", args.shader)
    };

    let shader_code = std::fs::read_to_string(shader_path).expect("Could not read shader file");

    let mode = match gltr::gl::shader::Mode::new(&args.mode) {
        Ok(mode) => mode,
        Err(err) => {
            warn!("Could not create mode: {}", err);
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

    // Create the uniforms
    let uniforms = uniform! {
        from: &texture1,
        to: &texture2,
        progress: args.progress as f32,
        ratio: dimensions1.0 as f32 / dimensions1.1 as f32, // Add new uniforms
        smoothness: 1.0f32, // smoothness 값을 설정
        _fromR: dimensions1.0 as f32 / dimensions1.1 as f32,
        _toR: dimensions2.0 as f32 / dimensions2.1 as f32,
    };

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
