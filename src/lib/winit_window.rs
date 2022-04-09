use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::borrow::Cow;

use crate::wgpu_renderer::WGPURenderer;
use crate::math::{Vertex, self};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub fn cube_positions() -> Vec<[i8; 3]> {
    [
        [-1, -1, 1],  // vertex a
        [1, -1, 1],   // vertex b
        [1, 1, 1],    // vertex c
        [-1, 1, 1],   // vertex d
        [-1, -1, -1], // vertex e
        [1, -1, -1],  // vertex f
        [1, 1, -1],   // vertex g
        [-1, 1, -1],  // vertex h
    ]
    .to_vec()
}

pub fn cube_colors() -> Vec<[i8; 3]> {
    [
        [0, 0, 1], // vertex a
        [1, 0, 1], // vertex b
        [1, 1, 1], // vertex c
        [0, 1, 1], // vertex d
        [0, 0, 0], // vertex e
        [1, 0, 0], // vertex f
        [1, 1, 0], // vertex g
        [0, 1, 0], // vertex h
    ]
    .to_vec()
}

fn cube_indices() -> Vec<u16> {
    [
        0, 1, 2, 2, 3, 0, // front
        1, 5, 6, 6, 2, 1, // right
        4, 7, 6, 6, 5, 4, // back
        0, 3, 7, 7, 4, 0, // left
        3, 2, 6, 6, 7, 3, // top
        0, 4, 5, 5, 1, 0, // bottom
    ]
    .to_vec()
}

fn create_vertices() -> Vec<Vertex> {
    let pos = cube_positions();
    let col = cube_colors();
    let mut data: Vec<Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(Vertex::vertex(pos[i], col[i]));
    }
    data.to_vec()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(1600, 1200));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.body()?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let vertices = create_vertices();
    let indices = cube_indices();

    let inputs = crate::wgpu_renderer::Inputs {
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../assets/cube_face_color.wgsl"
        ))),
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None, //Some(wgpu::IndexFormat::Uint32),
        vertices: Some(vertices),
        indices: Some(indices),
        camera_position: (3.0, 1.5, 3.0),
    };

    let mut wgpu_renderer = WGPURenderer::wgpu_init(&window, inputs).await;

    let render_start_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let now = instant::Instant::now();;
            let dt = now - render_start_time;
            
            wgpu_renderer.update(dt);
            match wgpu_renderer.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                // Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => if !wgpu_renderer.input(event) { match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                wgpu_renderer.resize(*physical_size);
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                wgpu_renderer.resize(**new_inner_size);
            },
            
            _ => {}
        }},
        _ => {}
    });
}
