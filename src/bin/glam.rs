use glam::{Mat4, Vec3, Vec4, Vec3A};

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
    let rot_mat_z = Mat4::from_rotation_z(
        f32::to_radians(20.0),
    );

    let rad_45 = f32::to_radians(45.0); 

    println!("rad_45: {} ; cos(rad_45): {} ; sin(rad_45): {}", rad_45, rad_45.cos(), rad_45.sin());


    // get total rotation matrix after another rotation around the z axis by 25 degrees
    let rot_mat =  rot_mat_z * Mat4::from_rotation_z(
            f32::to_radians(25.0),
        );

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
    
}
