use std::borrow::Cow;

use crate::pica_window::{self, Window};

pub struct WGPURenderer {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub texture_format: wgpu::TextureFormat,
    pub queue: wgpu::Queue,
    pub shader: wgpu::ShaderModule,
}

impl WGPURenderer {
    pub async fn wgpu_init(window: &Window) -> WGPURenderer {
        let size = window.window_attributes.size;
        let instance = wgpu::Instance::new(wgpu::Backends::DX12);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        let texture_format = surface.get_preferred_format(&adapter).unwrap();
        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &config);

        // Load the shaders from disk
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../assets/shader.wgsl"
            ))),
        });

        let wgpu_renderer = WGPURenderer{
            device,
            surface,
            texture_format,
            queue,
            shader,
        };

        wgpu_renderer
    }

}
