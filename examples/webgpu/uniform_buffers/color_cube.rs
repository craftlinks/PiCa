pub fn cube_positions() -> Vec<[i8; 3]> {
    [
        // front (0, 0, 1)
        [-1, -1, 1],
        [1, -1, 1],
        [-1, 1, 1],
        [-1, 1, 1],
        [1, -1, 1],
        [1, 1, 1],
        // right (1, 0, 0)
        [1, -1, 1],
        [1, -1, -1],
        [1, 1, 1],
        [1, 1, 1],
        [1, -1, -1],
        [1, 1, -1],
        // back (0, 0, -1)
        [1, -1, -1],
        [-1, -1, -1],
        [1, 1, -1],
        [1, 1, -1],
        [-1, -1, -1],
        [-1, 1, -1],
        // left (-1, 0, 0)
        [-1, -1, -1],
        [-1, -1, 1],
        [-1, 1, -1],
        [-1, 1, -1],
        [-1, -1, 1],
        [-1, 1, 1],
        // top (0, 1, 0)
        [-1, 1, 1],
        [1, 1, 1],
        [-1, 1, -1],
        [-1, 1, -1],
        [1, 1, 1],
        [1, 1, -1],
        // bottom (0, -1, 0)
        [-1, -1, -1],
        [1, -1, -1],
        [-1, -1, 1],
        [-1, -1, 1],
        [1, -1, -1],
        [1, -1, 1],
    ]
    .to_vec()
}

pub fn cube_colors() -> Vec<[i8; 3]> {
    [
        // front - blue
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        // right - red
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        // back - yellow
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        // left - aqua
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        // top - green
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        // bottom - fuchsia
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
    ]
    .to_vec()
}

use std::borrow::Cow;
use PiCa::error::Error;
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::wgpu_renderer::{Vertex, WGPURenderer};

fn vertex(p: [i8; 3], c: [i8; 3]) -> Vertex {
    Vertex {
        position: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
        color: [c[0] as f32, c[1] as f32, c[2] as f32, 1.0],
    }
}

fn create_vertices() -> Vec<Vertex> {
    let pos = cube_positions();
    let col = cube_colors();
    let mut data: Vec<Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(vertex(pos[i], col[i]));
    }
    data.to_vec()
}

pub fn main() -> Result<(), Error> {
    let vertices = create_vertices();

    let inputs = PiCa::wgpu_renderer::Inputs {
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../../assets/cube_face_color.wgsl"
        ))),
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None, //Some(wgpu::IndexFormat::Uint32),
        vertices: Some(vertices),
        indices: None,
        camera_position: (3.0, 1.5, 3.0),
    };

    let window_attributes = WindowAttributes::new()
        .with_title("Cube Color")
        .with_position(50, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    let mut wgpu_renderer = pollster::block_on(WGPURenderer::wgpu_init(window.as_ref(), inputs));

    // PiCa window rendering loop
    while window.pull() {
        window.push();

        // TODO: create a PiCa renderer trait, so this can be hidden behind the 'push()' function call
        wgpu_renderer.render().unwrap();
    }

    Ok(())
}
