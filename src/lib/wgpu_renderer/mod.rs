use crate::{math, utils};
#[cfg(target_arch = "x86_64")]
use crate::pica_window::Window;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use wgpu::{util::DeviceExt, IndexFormat, PrimitiveTopology, ShaderSource};
#[cfg(target_arch = "wasm32")]
use winit::{event::WindowEvent, window::Window};

pub mod camera;

pub struct Instance {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation))
                .to_cols_array_2d(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable )]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}


impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // 4 times a Vec4
                wgpu::VertexAttribute {
                    offset: 0,
                    // We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

use math::Vertex;
impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Inputs<'a> {
    pub source: ShaderSource<'a>,
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
    pub vertices: Option<Vec<Vertex>>,
    pub indices: Option<Vec<u16>>,
    pub camera_position: (f32, f32, f32),
    pub instances: Option<Vec<Instance>>,
}

pub struct WGPURenderer {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub clear_color: wgpu::Color,
    pub shader: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertices_len: usize,
    pub index_buffer: Option<wgpu::Buffer>,
    pub indices_len: usize,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub model_mat: Mat4,
    pub view_mat: Mat4,
    pub project_mat: Mat4,
    #[cfg(target_arch = "x86_64")]
    pub size: (i32, i32),
    #[cfg(target_arch = "wasm32")]
    pub size: winit::dpi::PhysicalSize<u32>,
    pub num_instances: Option<u32>,
    pub instances: Option<Vec<Instance>>,
    #[allow(dead_code)]
    pub instance_buffer: Option<wgpu::Buffer>,
}

impl<'a> WGPURenderer {
    pub async fn wgpu_init(window: &Window, inputs: Inputs<'_>) -> WGPURenderer {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                let size = window.window_attributes.size;
                let width = size.0;
                let height = size.1;
            } else if #[cfg(target_arch = "wasm32")] {
                let size = window.inner_size();
                let width = size.width;
                let height = size.height;
            }
        }
        let instance = wgpu::Instance::new(wgpu::Backends::all());
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
            .await
            .expect("Failed to create device");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: width as u32,
            height: height as u32,
            // https://docs.rs/wgpu/0.12.0/wgpu/enum.PresentMode.html
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&device, &config);

        let clear_color = wgpu::Color {
            r: 0.03,
            g: 0.01,
            b: 0.1,
            a: 1.0,
        };

        // Load the shaders from disk
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: inputs.source,
        });

        // uniform data
        let camera_position = inputs.camera_position.into();
        let look_direction = (0.0, 0.0, 0.0).into();
        let up_direction = Vec3::Y;
        let model_mat = math::create_transforms([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let vp_matrix = math::create_view_projection(
            camera_position,
            look_direction,
            up_direction,
            width as f32 / height as f32,
            math::ProjectionType::PERSPECTIVE,
        );
        let mvp_mat = vp_matrix.view_project_mat * model_mat;
        let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: crate::utils::as_bytes(mvp_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Uniform Bind Group Layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let mut num_instances = None;
        let mut instance_buffer = None;
        if let Some(instances) = inputs.instances.as_ref() {
            num_instances = Some(instances.len() as u32);
            let instance_data = instances
                .iter()
                .map(Instance::to_raw)
                .collect::<Vec<_>>();
            instance_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }));
        }
        

        // WGPU application dependent code
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: inputs.topology,
                strip_index_format: inputs.strip_index_format,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let mut vertex_buffer = None;
        let mut vertices_len: usize = 9;
        if let Some(vertices) = inputs.vertices {
            vertex_buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: crate::utils::as_bytes(vertices.as_slice()), // Everybody uses the Bytemuck crate here
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            );
            vertices_len = vertices.len();
        }

        let mut index_buffer = None;
        let mut indices_len = 0;

        if let Some(indices) = inputs.indices {
            index_buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: crate::utils::as_bytes(indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                }),
            );
            indices_len = indices.len();
        }

        let wgpu_renderer = WGPURenderer {
            device,
            surface,
            queue,
            shader,
            render_pipeline,
            config,
            vertex_buffer,
            vertices_len,
            index_buffer,
            indices_len,
            uniform_buffer,
            uniform_bind_group,
            model_mat,
            view_mat: vp_matrix.view_mat,
            project_mat: vp_matrix.project_mat,
            clear_color,
            size,
            num_instances,
            instance_buffer,
            instances: inputs.instances,
        };

        wgpu_renderer
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Later should just take a closure
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });
        {
            // <--mutable borrow of the command encoder needs to be dropped first before we can call encoder.finish()
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // This is what [[location(0)]] in the fragment shader targets
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);

            if let Some(vertex_buffer) = &self.vertex_buffer {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            }
            if let Some(instance_buffer) = &self.instance_buffer {
                render_pass.set_vertex_buffer(1,instance_buffer.slice(..));
            }
            if let Some(index_buffer) = &self.index_buffer {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.draw_indexed(0..self.indices_len as u32, 0, 0..self.num_instances.unwrap_or(1));
            } else {
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                // This is where [[builtin(vertex_index)]] comes from
                render_pass.draw(0..self.vertices_len as u32, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
                if new_size.width > 0 && new_size.height > 0 {
                    self.config.width = new_size.width;
                    self.config.height = new_size.height;
                    self.surface.configure(&self.device, &self.config);
                }
            }

            // Think about movinh this out if the renderer, as application specific code
            pub fn input(&mut self, event: &WindowEvent) -> bool {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        self.clear_color = wgpu::Color {
                            r: position.x as f64 / self.size.width as f64,
                            g: position.y as f64 / self.size.height as f64,
                            b: 1.0,
                            a: 1.0,
                        };
                        true
                    }
                    _ => false,
                }
            }

            const ANIMATION_SPEED: f32 = 5.0;

            pub fn update(&mut self, dt: std::time::Duration) {
                let dt = 5.0 * dt.as_secs_f32();
                let model_mat =
                    math::create_transforms([0.0, 0.0, 0.0], [dt.sin(), dt.sin() * dt.tanh(), dt.cos()], [1.0, 1.0, 1.0]);
                let mvp_mat = self.project_mat * self.view_mat * model_mat;
                let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
                self.queue.write_buffer(
                    &self.uniform_buffer,
                        0,
                        crate::utils::as_bytes(mvp_ref),
                );
            }
        }
    }
}
