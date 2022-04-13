use std::borrow::Cow;
use PiCa::error::Error;
use PiCa::math::{Vertex, self};
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::utils;
use PiCa::wgpu_renderer::{WGPURenderer, Instance};
use glam::{Vec3, Quat};

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

fn create_instances(num_instances_per_row: u32, instance_displacement: Vec3) -> Vec<Instance> {
    let instances = (0..num_instances_per_row)
        .flat_map(|z| {
            (0..num_instances_per_row).map(move |x| {
                let x = x*4;
                let z = z*4;  
                let position = Vec3::new(x as f32, 0.0, z as f32) - instance_displacement;

                let rotation = if position.length_squared() as u32 == 0 {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly
                    Quat::from_axis_angle(Vec3::Z, 0.0_f32.to_radians())
                } else {
                    Quat::from_axis_angle(position.normalize(), 45.0_f32.to_radians())
                };
                Instance { position, rotation }
            })
        })
        .collect::<Vec<Instance>>(); // <- num_instances_per_row^2 instances
    instances
}


pub fn main() -> Result<(), Error> {
    let vertices = create_vertices();
    let indices = cube_indices();

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    let instance_displacement: Vec3 = Vec3::new(
        NUM_INSTANCES_PER_ROW as f32,
        0.5,
        NUM_INSTANCES_PER_ROW as f32,
    );

    let instances = create_instances(NUM_INSTANCES_PER_ROW, instance_displacement);

    let inputs = PiCa::wgpu_renderer::Inputs {
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../../assets/cube_face_color.wgsl"
        ))),
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None, //Some(wgpu::IndexFormat::Uint32),
        vertices: Some(vertices),
        indices: Some(indices),
        camera_position: (5.0, 5.0, 5.0),
        instances: Some(instances), 
    };

    let window_attributes = WindowAttributes::new()
        .with_title("Cube Color")
        .with_position(50, 50)
        .with_size(1200, 1600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    let mut wgpu_renderer = pollster::block_on(WGPURenderer::wgpu_init(window.as_ref(), inputs));

    const ANIMATION_SPEED: f32 = 1.5;
    const ROTATION_SPEED: f32 = 0.25 * std::f32::consts::PI / 60.0;

    // PiCa window rendering loop
    while window.pull() {
        // window.push();

        let dt = ANIMATION_SPEED * window.time.seconds;
        let model_mat =
            math::create_transforms([0.0, 1.0, 1.0], [dt.sin(), 0.0, dt.cos()], [0.15, 0.15, 0.15]);
        let mvp_mat = wgpu_renderer.project_mat * wgpu_renderer.view_mat * model_mat;
        let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
        wgpu_renderer.queue.write_buffer(
            &wgpu_renderer.uniform_buffer,
            0,
            utils::as_bytes(mvp_ref),
        );

        for instance in wgpu_renderer.instances.as_mut().unwrap() {
            let amount = Quat::from_rotation_y(ROTATION_SPEED);
            let current = instance.rotation;
            instance.rotation = amount.mul_quat(current);
        }
        let instance_data = wgpu_renderer
            .instances.as_ref().unwrap()
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        wgpu_renderer.queue.write_buffer(
            &wgpu_renderer.instance_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&instance_data),
        );


        wgpu_renderer.render().unwrap();
    }

    Ok(())
}
