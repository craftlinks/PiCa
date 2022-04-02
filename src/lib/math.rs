use std::f32::consts::PI;

use glam::{Mat4, Vec3};

pub const OPENGL_TO_WGPU_MATRIX: &[f32;16] = &[
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
];

/// Combine transformations into a single 4x4 transformation matrix
pub fn create_transforms(translation: [f32; 3], rotation: [f32; 3], scaling: [f32; 3]) -> Mat4 {
    // create individual transformation matrices
    let trans_mat =
        Mat4::from_translation(Vec3::new(translation[0], translation[1], translation[2]));
    let rotate_mat_x = Mat4::from_rotation_x(rotation[0]);
    let rotate_mat_y = Mat4::from_rotation_y(rotation[1]);
    let rotate_mat_z = Mat4::from_rotation_z(rotation[2]);
    let scale_mat = Mat4::from_scale(Vec3::new(scaling[0], scaling[1], scaling[2]));
    // combine all transformation matrices together to form a final transform matrix: model matrix
    let model_mat = trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat;
    // return final model matrix
    model_mat
}

pub enum ProjectionType {
    PERSPECTIVE,
    ORTHO,
}

pub struct ViewProjectionMat {
    pub view_mat: Mat4,
    pub project_mat: Mat4,
    pub view_project_mat: Mat4,
}

pub fn create_view_projection(
    camera_position: Vec3,
    look_direction: Vec3,
    up_direction: Vec3,
    aspect: f32,
    is_perspective: ProjectionType,
) -> ViewProjectionMat {
    let view_mat = Mat4::look_at_rh(camera_position, look_direction, up_direction);

    let project_mat = match is_perspective {
        ProjectionType::PERSPECTIVE => {
            Mat4::from_cols_array(OPENGL_TO_WGPU_MATRIX) * Mat4::perspective_rh(2.0 * PI / 5.0, aspect, 0.1, 100.0)
        }
        ProjectionType::ORTHO => {
            Mat4::from_cols_array(OPENGL_TO_WGPU_MATRIX) * Mat4::orthographic_rh(-4.0, 4.0, -3.0, 3.0, -1.0, 6.0)
        }
    };

    // construct view-projection matrix
    let view_project_mat = project_mat * view_mat;

    ViewProjectionMat {
        view_mat,
        project_mat,
        view_project_mat,
    }
}

pub fn create_view(
    camera_position: Vec3,
    look_direction: Vec3,
    up_direction: Vec3,
) -> Mat4 {
    Mat4::look_at_rh(camera_position, look_direction, up_direction)
}
pub fn create_projection(aspect: f32, is_perspective: ProjectionType) -> Mat4 {
    let project_mat = match is_perspective {
        ProjectionType::PERSPECTIVE => {
            Mat4::from_cols_array(OPENGL_TO_WGPU_MATRIX) * Mat4::perspective_rh(2.0 * PI / 5.0, aspect, 0.1, 100.0)
        }
        ProjectionType::ORTHO => {
            Mat4::from_cols_array(OPENGL_TO_WGPU_MATRIX) * Mat4::orthographic_rh(-4.0, 4.0, -3.0, 3.0, -1.0, 6.0)
        }
    };
    project_mat
}

#[repr(C)]
#[derive(Clone, Copy)]
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
