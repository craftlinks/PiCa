use PiCa::error::Error;
use PiCa::pica_window::{Window, WindowAttributes};

pub fn main() -> Result<(), Error> {
    let instances = wgpu::Instance::new(wgpu::Backends::all());
    for adapter in instances.enumerate_adapters(wgpu::Backends::all()) {
        println!("{:?}", adapter.get_info());
    }

    let window_attributes = WindowAttributes::new()
        .with_title("Awesome PiCa Simulation")
        .with_position(50, 50)
        .with_size(800, 800);

    let mut window = Window::new_with_attributes(window_attributes)?;
    while window.pull() {
        window.push();
    }

    Ok(())
}
