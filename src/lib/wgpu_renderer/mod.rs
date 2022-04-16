use std::borrow::Cow;

use crate::pica_window::Window;
use crate::utils;
use crate::{math, wgpu_renderer::camera::Camera};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3, Vec4};
use wgpu::ShaderModule;
use wgpu::{util::DeviceExt, IndexFormat, PrimitiveTopology, ShaderSource};

pub mod camera;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = Vec4::from((camera.position, 0.0)).to_array(); // Check if this is correct
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).to_cols_array_2d();
    }
}

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
#[derive(Copy, Clone, Pod, Zeroable)]
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn vertex(p: [i8; 3], c: [i8; 3]) -> Vertex {
        Vertex {
            position: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
            color: [c[0] as f32, c[1] as f32, c[2] as f32, 1.0],
        }
    }
}

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

pub struct RendererAttributes {
    pub source: &'static str,
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
    pub vertices: Option<Vec<Vertex>>,
    pub indices: Option<Vec<u16>>,
    pub camera_position: Vec3,
    pub instances: Option<Vec<Instance>>,
}

impl Default for RendererAttributes {
    fn default() -> Self {
        Self {
            source: include_str!(
            "../../../assets/cube_face_color.wgsl"
        ),
            topology: Default::default(),
            strip_index_format: Default::default(),
            vertices: Default::default(),
            indices: Default::default(),
            camera_position: Vec3::new(0.5, 0.5, 0.5),
            instances: Default::default(),
        }
    }
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
    pub size: (i32, i32),
    pub num_instances: Option<u32>,
    pub instances: Option<Vec<Instance>>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub camera: Camera,
    pub projection: camera::Projection,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl WGPURenderer {
    pub async fn new(window: &Window) -> WGPURenderer {
        let attributes = RendererAttributes::default();
        WGPURenderer::new_with_attributes(window, attributes).await
    }

    pub async fn new_with_attributes(
        window: &Window,
        renderer_attributes: RendererAttributes,
    ) -> WGPURenderer {
        let size = window.window_attributes.size;
        let width = size.0;
        let height = size.1;
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

        // LOAD SHADER ///////////////////////////////////////////////////////////////////////////////////////
        
        let shader_source = wgpu::ShaderSource::Wgsl(Cow::Borrowed(renderer_attributes.source));
        
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: shader_source,
        });


        
        ///////////////////////////////////////////////////////////////////////////////////////////////////////
        // VERTEX BUFFER

        let mut vertex_buffer = None;
        let mut vertices_len: usize = 9;
        if let Some(vertices) = renderer_attributes.vertices {
            vertex_buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: crate::utils::as_bytes(vertices.as_slice()), // Everybody uses the Bytemuck crate here
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            );
            vertices_len = vertices.len();
        }


        ////////////////////////////////////////////////////////////////////////////////////////////////////////
        // INDEX BUFFER

        let mut index_buffer = None;
        let mut indices_len = 0;

        if let Some(indices) = renderer_attributes.indices {
            index_buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: crate::utils::as_bytes(indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                }),
            );
            indices_len = indices.len();
        }

        ///////////////////////////////////////////////////////////////////////////////////////////////////////////
        // INSTANCE BUFFER
        let mut num_instances = None;
        let mut instance_buffer = None;
        if let Some(instances) = renderer_attributes.instances.as_ref() {
            num_instances = Some(instances.len() as u32);
            let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            instance_buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        /////////////////////////////////////////////////////////////////////////////////////////////////////
        // uniform buffer - global transformation
        let model_mat = math::create_transforms([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: crate::utils::as_bytes(model_mat.as_ref()),
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



        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // uniform buffer - Camera
        let camera = Camera::new(
            renderer_attributes.camera_position,
            -90.0_f32.to_radians(),
            -20.0_f32.to_radians(),
            0.1,
            0.001,
        );

        let projection = camera::Projection::new(
            config.width,
            config.height,
            45.0_f32.to_radians(),
            0.1,
            100.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // RENDER PIPELINE
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &camera_bind_group_layout],
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
                topology: renderer_attributes.topology,
                strip_index_format: renderer_attributes.strip_index_format,
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
            clear_color,
            size,
            num_instances,
            instance_buffer,
            instances: renderer_attributes.instances,
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
        };

        wgpu_renderer
    }

    pub fn write_uniform(&mut self, data:&[f32; 16]) {
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            utils::as_bytes(data),
        );
    }

    pub fn write_instances(&mut self, data: Vec<InstanceRaw>) {
        self.queue.write_buffer(
            &self.instance_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&data),
        );
    }

    pub fn write_camera(&mut self, data: &[CameraUniform]) {
            self.queue.write_buffer(
            &self.camera_buffer,
            0,
            utils::as_bytes(data),
        );
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

            // RENDER PIPELINE
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            if let Some(vertex_buffer) = &self.vertex_buffer {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            }
            if let Some(instance_buffer) = &self.instance_buffer {
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            }
            if let Some(index_buffer) = &self.index_buffer {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(
                    0..self.indices_len as u32,
                    0,
                    0..self.num_instances.unwrap_or(1),
                );
            }
            
            // } else {
            //     render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            //     // This is where [[builtin(vertex_index)]] comes from
            //     render_pass.draw(0..self.vertices_len as u32, 0..1);
            // }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}
