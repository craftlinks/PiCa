use std::borrow::Cow;

use wgpu::{ShaderSource, PrimitiveTopology, IndexFormat};
use crate::pica_window::Window;

pub struct Inputs<'a> {
    pub source: ShaderSource<'a>,
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
}

pub struct WGPURenderer {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub texture_format: wgpu::TextureFormat,
    pub queue: wgpu::Queue,
    pub shader: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl WGPURenderer {
    pub async fn wgpu_init(
        window: &Window,
        inputs: Inputs<'_>,
    ) -> WGPURenderer {
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
            source: inputs.source,
        });

        // WGPU application dependent code
        let pipeline_layout =
            device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[], // placeholder, vertext and color data are written directly in the shader
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[texture_format.into()],
                    }),
                    primitive: wgpu::PrimitiveState{
                        topology: inputs.topology,
                        strip_index_format: inputs.strip_index_format,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        let wgpu_renderer = WGPURenderer{
            device,
            surface,
            texture_format,
            queue,
            shader,
            render_pipeline,
        };

        wgpu_renderer
    }

    pub fn render(&mut self, num_vertices: usize) {
        // Later should just take a closure
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.03,
                            g: 0.01,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..num_vertices as u32, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
}
}
