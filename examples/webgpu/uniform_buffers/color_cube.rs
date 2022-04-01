use std::borrow::Cow;
use PiCa::error::Error;
use PiCa::math;
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::utils;
use PiCa::wgpu_renderer::{Vertex, WGPURenderer};

pub fn cube_positions() -> Vec<[i8; 3]> {
    [
        [-1, -1, 1],  // vertex a
        [1, -1, 1],   // vertex b
        [1, 1, 1],    // vertex c
        [-1, 1, 1],   // vertex d
        [-1, -1, -1], // vertex e
        [1, -1, -1],  // vertex f
        [1, 1, -1],   // vertex g
        [-1, 1, -1],  // vertex h
    ]
    .to_vec()
}

pub fn cube_colors() -> Vec<[i8; 3]> {
    [
        [0, 0, 1], // vertex a
        [1, 0, 1], // vertex b
        [1, 1, 1], // vertex c
        [0, 1, 1], // vertex d
        [0, 0, 0], // vertex e
        [1, 0, 0], // vertex f
        [1, 1, 0], // vertex g
        [0, 1, 0], // vertex h
    ]
    .to_vec()
}

fn cube_indices() -> Vec<u16> {
    [
        0, 1, 2, 2, 3, 0, // front
        1, 5, 6, 6, 2, 1, // right
        4, 7, 6, 6, 5, 4, // back
        0, 3, 7, 7, 4, 0, // left
        3, 2, 6, 6, 7, 3, // top
        0, 4, 5, 5, 1, 0, // bottom
    ]
    .to_vec()
}

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
    let indices = cube_indices();

    let inputs = PiCa::wgpu_renderer::Inputs {
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../../assets/cube_face_color.wgsl"
        ))),
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None, //Some(wgpu::IndexFormat::Uint32),
        vertices: Some(vertices),
        indices: Some(indices),
        camera_position: (3.0, 1.5, 3.0),
    };

    let window_attributes = WindowAttributes::new()
        .with_title("Cube Color")
        .with_position(50, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    let mut wgpu_renderer = pollster::block_on(WGPURenderer::wgpu_init(window.as_ref(), inputs));

    const ANIMATION_SPEED: f32 = 1.0;

    // PiCa window rendering loop
    while window.pull() {
        // window.push();

        let dt = ANIMATION_SPEED * window.time.seconds;
        let model_mat =
            math::create_transforms([0.0, 0.0, 0.0], [dt.sin(), 0.0, dt.cos()], [1.0, 1.0, 1.0]);
        let mvp_mat = wgpu_renderer.project_mat * wgpu_renderer.view_mat * model_mat;
        let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
        wgpu_renderer.queue.write_buffer(
            &wgpu_renderer.uniform_buffer,
            0,
            utils::as_bytes(mvp_ref),
        );

        wgpu_renderer.render().unwrap();
    }

    Ok(())
}
