use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::error::Error;
use PiCa::wgpu_renderer::WGPURenderer;

use std::borrow::Cow;


pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("Triangles")
        .with_position(50, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    let mut primitive_type = "triangle_list";
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        primitive_type = &args[1];

    }

    let mut topology = wgpu::PrimitiveTopology::TriangleList;
    let mut strip_index_format = None;
    if primitive_type == "triangle-strip" {
        topology = wgpu::PrimitiveTopology::TriangleStrip;
        strip_index_format = Some(wgpu::IndexFormat::Uint32);
    }

    let inputs = PiCa::wgpu_renderer::Inputs{
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../assets/triangles.wgsl"))),
        topology,
        strip_index_format,
        vertices: None,
        
    };

    let mut wgpu_renderer = pollster::block_on( WGPURenderer::wgpu_init(window.as_ref(), inputs));

    // PiCa window rendering loop
    while window.pull() {
        window.push();
        
        //TODO: create a PiCa renderer trait, so this can be hidden behind the 'push()' function call
        wgpu_renderer.render(9);
    }

    Ok(())
}
