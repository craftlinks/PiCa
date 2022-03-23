use std::borrow::Cow;
use PiCa::error::Error;
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::wgpu_renderer::{Inputs, WGPURenderer};

pub fn main() -> Result<(), Error> {

    // Create a PiCa window
    let window_attributes = WindowAttributes::new()
        .with_title("Awesome PiCa Simulation")
        .with_position(50, 50)
        .with_size(800, 800);

    let mut window = Window::new_with_attributes(window_attributes)?;

    // Logging for wgpu
    env_logger::init();

    // Define the inputs for the WGPURenderer
    let inputs = Inputs {
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../assets/shader.wgsl"))),
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        vertices: None,
        indices: None,
    };

    // WGPURenderer initialization
    let mut wgpu_renderer = pollster::block_on(WGPURenderer::wgpu_init(window.as_ref(), inputs));

    // PiCa window rendering loop
    while window.pull() {
        window.push();
        
        // Paint to the window surface
        wgpu_renderer.render();
    }

    // All was well
    Ok(())
}