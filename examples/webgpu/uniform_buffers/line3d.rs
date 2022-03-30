use std::borrow::Cow;
use PiCa::error::Error;
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::wgpu_renderer::{Vertex, WGPURenderer};

fn create_vertices() -> [Vertex; 1200]{
    let mut vertices = [Vertex {
        position: [0.0, 0.0, 0.0, 1.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }; 1200];
    for i in 0..1200 {
        let t = i as f32 / 150.0;
        let x = (-t).exp() * (30.0 * t).sin();
        let z = (-t).exp() * (30.0 * t).cos();
        let y = 2.0 * (-t).sin() - 1.0;
        vertices[i] = Vertex {
            position: [x, y, z, 1.0],
            color: [0.5, 0.5, 0.8, 1.0],
        };
    }
    vertices
}

pub fn main() -> Result<(), Error> {
    
    let vertices = create_vertices();
    
    let inputs = PiCa::wgpu_renderer::Inputs {
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../../assets/line3d.wgsl"
        ))),
        topology: wgpu::PrimitiveTopology::LineStrip,
        strip_index_format: Some(wgpu::IndexFormat::Uint32),
        vertices: Some(vertices.to_vec()),
        indices: None,
        camera_position: (1.5, 1.0, 3.0),
    };
    
    let window_attributes = WindowAttributes::new()
        .with_title("Line3D")
        .with_position(50, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    let mut wgpu_renderer = pollster::block_on(WGPURenderer::wgpu_init(window.as_ref(), inputs));

    // PiCa window rendering loop
    while window.pull() {
        window.push();

        // TODO: create a PiCa renderer trait, so this can be hidden behind the 'push()' function call
        wgpu_renderer.render();
    }

    Ok(())
}
