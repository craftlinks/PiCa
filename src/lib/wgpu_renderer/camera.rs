use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};

use crate::pica_window::Window;
use crate::pica_window::{SHIFT, SPACE};

pub const OPENGL_TO_WGPU_MATRIX: &[f32; 16] = &[
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
];

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    fn reset(&mut self) {
        self.amount_left = 0.0;
        self.amount_right = 0.0;
        self.amount_forward = 0.0;
        self.amount_backward = 0.0;
        self.amount_up = 0.0;
        self.amount_down = 0.0;
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        self.scroll = 0.0;
    }

    pub fn update(&mut self, window: &Window) {
        self.reset();
        // Process Keyboard
        if window.keys['W' as u8 as usize].down {
            self.amount_forward = 1.0;
            println!("forward");
        }
        if window.keys['S' as u8 as usize].down {
            self.amount_backward = 1.0;
            println!("backward");
        }
        if window.keys['D' as u8 as usize].down {
            self.amount_right = 1.0;
            println!("right");
        }
        if window.keys['A' as u8 as usize].down {
            self.amount_left = 1.0;
            println!("left");
        }
        if window.keys[SPACE].down {
            self.amount_up = 1.0;
            println!("up");
        }
        if window.keys[SHIFT].down {
            self.amount_up = 1.0;
            println!("down");
        }

        // Process Mouse
        if window.mouse.left_button.down {
            let mousex = window.mouse.delta_position.0 as f32;
            let mousey = window.mouse.delta_position.1 as f32;
            self.rotate_horizontal = mousex as f32;
            self.rotate_vertical = mousey as f32;
        }

        // Process Mouse Scroll
        // SCROLL is currently broken (values between 1.0 and 545), needs fixing
        self.scroll = -(window.mouse.delta_wheel as f32);
        if self.scroll != 0.0 {
            println!("{:?}", self.scroll);
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    yaw: f32,   //rads
    pitch: f32, //rads
    camera_controller: CameraController,
}

impl Camera {
    pub fn new(position: Vec3, yaw: f32, pitch: f32, speed: f32, sensitivity: f32) -> Self {
        let camera_controller = CameraController::new(speed, sensitivity);

        Self {
            position,
            yaw,
            pitch,
            camera_controller,
        }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Mat4::look_at_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }

    // Note that this is a WIN32 specific function at this stage...
    #[cfg(target_arch = "x86_64")]
    pub fn update_camera(&mut self, window: &mut Window) {
        self.camera_controller.update(window);
        
        // TODO Geert: update the Camera

        
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32, // rads
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::from_cols_array(OPENGL_TO_WGPU_MATRIX)
            * Mat4::perspective_rh_gl(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
