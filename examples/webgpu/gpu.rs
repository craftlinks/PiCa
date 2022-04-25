use pica::{pica_window::{WindowAttributes, Window}, error::Error};

pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("GPU INFO")
        .with_position(900, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;
    
    let instance = wgpu::Instance::new(wgpu::Backends::DX12);
    // let available_adapters = instance.enumerate_adapters(wgpu::Backends::all());
    // for adapter in available_adapters {
    //     let adapter_info = adapter.get_info();
    //     println!("{:?}", adapter_info );
    // }

    let surface = unsafe { instance.create_surface(window.as_ref()) };
    let adapter = 
        pollster::block_on(instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference:: HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })).expect("Failed to find an appropriate adapter");
    let adapter_info = adapter.get_info();
    println!("device: {:?}\nbackend: {:?}", adapter_info.name, adapter_info.backend );

    let (device, queue) = pollster::block_on(adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device Features"),
                    // https://docs.rs/wgpu/0.12.0/wgpu/struct.Features.html
                    features: wgpu::Features::empty(),
                    // https://docs.rs/wgpu/0.12.0/wgpu/struct.Limits.html
                    limits: if cfg!(target_arch = "wasm32") {
                        // This is a set of limits that is lower even than the [downlevel_defaults()],
                        // configured to be low enough to support running in the browser using WebGL2.
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
        ).expect("Failed to create device");

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: window.window_attributes.size.0 as u32,
        height: window.window_attributes.size.1 as u32,
        // https://docs.rs/wgpu/0.12.0/wgpu/enum.PresentMode.html
        present_mode: wgpu::PresentMode::Immediate,
    };
    // main window swap chain
    surface.configure(&device, &config);

    // TODO: Zig also defines a buffer_pool, texture_pool, and render_pipeline_pool
    // See C:\Users\CraftLinks\Repos\tmp\trait-test for easiest way to accomplish this in Rust

    while window.pull() {}

    Ok(())
}
