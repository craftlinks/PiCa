use PiCa::error::Error;
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::wgpu_renderer::{WGPURenderer, Vertex};
use std::borrow::Cow;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
];

pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("GPU Buffer - Triangle")
        .with_position(50, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    let inputs = PiCa::wgpu_renderer::Inputs {
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../../assets/buffered_triangle.wgsl"
        ))),
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        vertices: Some(VERTICES),
    };

    let mut wgpu_renderer = pollster::block_on(WGPURenderer::wgpu_init(window.as_ref(), inputs));

    // PiCa window rendering loop
    while window.pull() {
        window.push();

        // TODO: create a PiCa renderer trait, so this can be hidden behind the 'push()' function call
        wgpu_renderer.render(VERTICES.len());
    }

    Ok(())
}
