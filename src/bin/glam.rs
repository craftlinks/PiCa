use std::f32::consts::FRAC_PI_6;

use glam::{Mat4, Vec3, Vec3A, Vec4};

pub fn main() {
    // SCALING
    println!("SCALING");
    // create original vector
    let my_vec = Vec4::new(1.0, 2.0, 3.0, 1.0);
    // create scale matrix
    let my_mat = Mat4::from_scale(Vec3::new(0.5, 0.5, 1.5));
    // get the scaled vector
    let scaled_vec = my_mat * my_vec;
    println!("\nOriginal vector: \n{:?}", my_vec);
    println!("Scaling matrix: \n{:?}", my_mat);
    println!(
        "Vector after scaling: scaled_vec = my_mat * my_vec = \n{:?}",
        scaled_vec
    );
    // two successive scaling transforms:
    // get total scaling matrix after another scaling transformation:
    let scaling_mat = my_mat * Mat4::from_scale(Vec3::new(1.0, 0.5, 0.3));
    // get final scaled vector
    let final_vec = scaling_mat * my_vec;
    println!("\nScaling matrix after two scalings: \n{:?}", scaling_mat);
    println!(
        "Vector after two scalings: scaled_vec = scaling_mat * my_vec = \n{:?}\n",
        final_vec
    );

    // TRANSLATION
    println!("TRANSLATION");
    // Create first translation matrix
    let my_mat = Mat4::from_translation(Vec3::new(2.0, 2.5, 3.0));

    // Get total translation matrix after another translation
    let trans_mat = my_mat * Mat4::from_translation(Vec3::new(-3.0, -2.0, -1.0));

    // Get final translated vector
    let trans_vec = trans_mat * my_vec;

    println!("\nOriginal vector: my_vec = \n{:?}", my_vec);
    println!(
        "Total translation matrix after two translations: trans_mat: \n{:?}",
        trans_mat
    );
    println!(
        "Vector after two translations: trans_vec = trans_mat * my_vec = \n{:?}\n ",
        trans_vec
    );

    // ROTATION
    println!("ROTATION");

    // create rotation matrix around z axis by 20 degrees
    let rot_mat_z = Mat4::from_rotation_z(f32::to_radians(20.0));

    let rad_45 = f32::to_radians(45.0);

    println!(
        "rad_45: {} ; cos(rad_45): {} ; sin(rad_45): {}",
        rad_45,
        rad_45.cos(),
        rad_45.sin()
    );

    // get total rotation matrix after another rotation around the z axis by 25 degrees
    let rot_mat = rot_mat_z * Mat4::from_rotation_z(f32::to_radians(25.0));

    // get final rotated vector
    let rot_vec = rot_mat * my_vec;

    println!("\nOriginal vector: my_vec = \n{:?}", my_vec);
    println!(
        "Total rotation matrix after two rotations: rot_mat = \n{:?}",
        rot_mat
    );
    println!(
        "Vector after two rotations: rot_vec = rot_mat * my_vec = \n{:?}\n",
        rot_vec
    );

    // TRANSFORMS
    println!("TRANSFORM");
    pub fn create_transforms(translation: [f32; 3], rotation: [f32; 3], scaling: [f32; 3]) -> Mat4 {
        // create individual transformation matrices
        let trans_mat =
            Mat4::from_translation(Vec3::new(translation[0], translation[1], translation[2]));
        let rotate_mat_x = Mat4::from_rotation_x(f32::to_radians(rotation[0]));
        let rotate_mat_y = Mat4::from_rotation_y(f32::to_radians(rotation[1]));
        let rotate_mat_z = Mat4::from_rotation_z(f32::to_radians(rotation[2]));
        let scale_mat = Mat4::from_scale(Vec3::new(scaling[0], scaling[1], scaling[2]));
        // combine all transformation matrices together to form a final transform matrix: model matrix
        let model_mat = trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat;
        // return final model matrix
        model_mat
    }

    let trans_mat = create_transforms([2.0, 3.0, 1.0], [20.0, 30.0, 45.0], [0.2, 0.5, 2.0]);
    let res = trans_mat * my_vec;
    println!("my_vec {} -> {}", my_vec, res);

    // Viewing Transform
    println!("VIEWING TRANSFORM");
    // position of the viewer
    let eye = Vec3::new(3.0, 4.0, 5.0);
    //point the viewer is looking at
    let center = Vec3::new(-3.0, -4.0, -5.0);
    // vector pointing up
    let up = Vec3::new(0.0, 1.0, 0.0);
    // construct view matrix:
    let view_mat = Mat4::look_at_rh(eye, center, up);
    println!("\nposition of viewer: {:?}", eye);
    println!("point the viewer is looking at: {:?}", center);
    println!("up direction: {:?}", up);
    println!("view matrix: {:?}\n ", view_mat);

    // Perspective Projection
    println!("VIEWING TRANSFORM");

    // frustum and perspective parameters
    let left = -3.0;
    let right = 3.0;
    let bottom = -5.0;
    let top = 5.0;
    let near = 1.0;
    let far = 100.0;
    let fovy = FRAC_PI_6;
    let aspect = 1.5;
    // construct the frustum matrix
    // let frustum_mat = frustum(left, right, bottom, top, near, far);
    // construct perspective projection matrix
    let persp_mat = Mat4::perspective_rh(fovy, aspect, near, far);
    // println!("\nfrustum matrix: {:?}\n ", frustum_mat);
    println!("perspective matrix: {:?}\n ", persp_mat);

    // construct orthographic projection matrix
    let ortho_mat = Mat4::orthographic_rh(left, right, bottom, top, near, far);
    println!("orthographic matrix: {:?}\n ", ortho_mat);


}
