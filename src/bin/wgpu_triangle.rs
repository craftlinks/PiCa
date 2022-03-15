use PiCa::error::Error;
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::wgpu_renderer::WGPURenderer;
use wgpu::RenderPipeline;

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

    env_logger::init();

    // WGPU initialization
    let mut wgpu_renderer = pollster::block_on(WGPURenderer::wgpu_init(window.as_ref()));

    // WGPU application dependent code
    let pipeline_layout = wgpu_renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[], // placeholder, vertext and color data are written directly in the shader
        push_constant_ranges: &[],
    });

    let render_pipeline = wgpu_renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &wgpu_renderer.shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &wgpu_renderer.shader,
            entry_point: "fs_main",
            targets: &[wgpu_renderer.texture_format.into()],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // PiCa window rendering loop
    while window.pull() {
        window.push();
        render(&mut wgpu_renderer, &render_pipeline);
    }

    Ok(())
}

pub fn render(wgpu_renderer: &mut WGPURenderer, render_pipeline: &RenderPipeline) {
    // Later should just take a closure
    let frame = wgpu_renderer.surface.get_current_texture().unwrap();
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        wgpu_renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.062,
                        b: 0.08,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(render_pipeline);
        rpass.draw(0..3, 0..1);
    }
    wgpu_renderer.queue.submit(Some(encoder.finish()));
    frame.present();

}
