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

    #[cfg(target_arch = "x86_64")]
    pub fn update(&mut self, window: &Window) {
        self.reset();
        // Process Keyboard
        if window.keys['W' as u8 as usize].down {
            self.amount_forward = 0.1;
            println!("forward");
        }
        if window.keys['S' as u8 as usize].down {
            self.amount_backward = 0.1;
            println!("backward");
        }
        if window.keys['D' as u8 as usize].down {
            self.amount_right = 0.1;
            println!("right");
        }
        if window.keys['A' as u8 as usize].down {
            self.amount_left = 0.1;
            println!("left");
        }
        if window.keys[SPACE].down {
            self.amount_up = 0.1;
            println!("up");
        }
        if window.keys[CTR].down {
            self.amount_up = 0.1;
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

        use crate::pica_window::CTR;
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

        let dt = window.time.seconds;

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += forward
            * (self.camera_controller.amount_forward - self.camera_controller.amount_backward)
            * self.camera_controller.speed
            * dt;
        self.position += right
            * (self.camera_controller.amount_right - self.camera_controller.amount_left)
            * self.camera_controller.speed
            * dt;

        // Move in/out (aka. "zoom")
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let scrollward = Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.position += scrollward
            * self.camera_controller.scroll
            * self.camera_controller.speed
            * self.camera_controller.sensitivity
            * dt;
        self.camera_controller.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.position.y += (self.camera_controller.amount_up - self.camera_controller.amount_down)
            * self.camera_controller.speed
            * dt;

        // Rotate
        self.yaw +=
            self.camera_controller.rotate_horizontal * self.camera_controller.sensitivity * dt;
        self.pitch +=
            -self.camera_controller.rotate_vertical * self.camera_controller.sensitivity * dt;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -SAFE_FRAC_PI_2.to_radians() {
            self.pitch = -SAFE_FRAC_PI_2.to_radians();
        } else if self.pitch > SAFE_FRAC_PI_2.to_radians() {
            self.pitch = SAFE_FRAC_PI_2.to_radians();
        }
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
