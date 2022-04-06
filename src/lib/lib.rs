use std::f32::consts::PI;

use glam::{Mat4, Vec3};

pub mod dx12_renderer;
pub mod math;
pub mod pica_window;
pub mod wgpu_renderer;

pub mod pica_time {
    use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
    #[derive(Default, Debug)]
    pub struct Time {
        pub delta_ticks: i64,
        pub delta_nanoseconds: i64,
        pub delta_microseconds: i64,
        pub delta_milliseconds: i64,
        pub delta_seconds: f32,

        pub ticks: i64,
        pub nanoseconds: i64,
        pub microseconds: i64,
        pub milliseconds: i64,
        pub seconds: f32,

        pub initial_ticks: i64,
        pub ticks_per_second: i64,
    }

    impl Time {
        pub fn new() -> Self {
            let mut ticks_per_second: i64 = 0;
            unsafe { QueryPerformanceFrequency(&mut ticks_per_second) };
            let mut initial_ticks: i64 = 0;
            unsafe { QueryPerformanceCounter(&mut initial_ticks) };
            Self {
                ticks_per_second,
                initial_ticks,
                ..Default::default()
            }
        }
    }
}

pub mod pica_mouse {
    use crate::error::Error;
    use std::mem::size_of;
    use windows::Win32::{
        Foundation::HWND,
        UI::Input::{RegisterRawInputDevices, RAWINPUTDEVICE, RAWINPUTDEVICE_FLAGS},
    };
    pub type Result<T> = std::result::Result<T, crate::error::Error>;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct Button {
        pub down: bool,     // current state
        pub pressed: bool,  // !down -> down
        pub released: bool, // down -> !down
    }

    impl Button {
        pub fn update_button(&mut self, is_down: bool) {
            let was_down = self.down;
            self.down = is_down;
            self.pressed = !was_down && is_down;
            self.released = was_down && !is_down;
        }
    }

    #[derive(Debug, Default)]
    pub struct Mouse {
        pub left_button: Button,
        pub right_button: Button,
        pub wheel: i32,
        pub delta_wheel: i32,
        pub position: (i32, i32),
        pub delta_position: (i32, i32),
    }

    impl Mouse {
        // what is going on with this RAWINPUTDEVICE_FLAGS??
        pub fn new(win32_window_handle: HWND) -> Result<Self> {
            let raw_input_device = &[RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x02,
                dwFlags: RAWINPUTDEVICE_FLAGS(0),
                hwndTarget: win32_window_handle,
            }];
            let raw_input_device = Box::new(raw_input_device);
            if unsafe {
                !RegisterRawInputDevices(*raw_input_device, size_of::<RAWINPUTDEVICE>() as u32)
                    .as_bool()
            } {
                return Err(Error::Mouse(
                    "Failed to register mouse as raw input device.".to_owned(),
                ));
            }
            Ok(Mouse::default())
        }
    }
}

pub mod error {
    use std::{error, fmt};

    #[derive(Debug)]
    pub enum Error {
        /// Win32 Error
        Win32Error(Win32Error),
        Window(String),
        Mouse(String),
    }
    /// The error type for when the OS cannot perform the requested operation.
    #[derive(Debug)]
    pub struct Win32Error {
        line: u32,
        file: &'static str,
        error: windows::core::Error,
    }

    impl Win32Error {
        #[allow(dead_code)]
        pub(crate) fn new(
            line: u32,
            file: &'static str,
            error: windows::core::Error,
        ) -> Win32Error {
            Win32Error { line, file, error }
        }
    }

    #[allow(unused_macros)]
    #[macro_export]
    macro_rules! win_error {
        ($error:expr) => {
            crate::error::Win32Error::new(line!(), file!(), $error)
        };
    }

    impl fmt::Display for Win32Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
            f.pad(&format!(
                "os error at {}:{}: {}",
                self.file, self.line, self.error
            ))
        }
    }

    // impl error::Error for Win32Error {}
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

// A Camera
pub mod camera {
    use crate::math::{self, Rad};
    use glam::{Mat4, Vec3, Vec4};
    use std::f32::consts::FRAC_PI_2;

    const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

    pub struct Camera {
        pub position: Vec3,
        yaw: math::Rad,   // horizontal rotation
        pitch: math::Rad, // vertical rotation
    }

    impl Camera {
        pub fn new<Pt: Into<Vec3>, Yaw: Into<Rad>, Pitch: Into<Rad>>(
            position: Pt,
            yaw: Yaw,
            pitch: Pitch,
        ) -> Self {
            Self {
                position: position.into(),
                yaw: yaw.into().to_radians(),
                pitch: pitch.into().to_radians(),
            }
        }

        pub fn view_mat(&self) -> Mat4 {
            let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
            let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
            Mat4::look_at_rh(
                self.position,
                Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
                Vec3::Y,
            )
        }
    }

    #[derive(Debug)]
    pub struct CameraController {
        pub amount_left: f32,
        pub amount_right: f32,
        pub amount_forward: f32,
        pub amount_backward: f32,
        pub amount_up: f32,
        pub amount_down: f32,
        pub rotate_horizontal: f32,
        pub rotate_vertical: f32,
        pub scroll: f32,
        pub speed: f32,
        pub sensitivity: f32,
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

        pub fn mouse_move(&mut self, mousex: f32, mousey: f32) {
            self.rotate_horizontal = mousex as f32;
            self.rotate_vertical = mousey as f32;
        }

        pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
            // Move forward/backward and left/right
            let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
            let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
            let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
            camera.position +=
                forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
            camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

            // Move in/out
            let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
            let scrollward =
                Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
            camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
            self.scroll = 0.0;

            // Move up/down.
            camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

            // Rotate
            camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
            camera.pitch += -self.rotate_vertical * self.sensitivity * dt;

            self.rotate_horizontal = 0.0;
            self.rotate_vertical = 0.0;

            // Keep the camera's angle from going too high/low.
            if camera.pitch < -SAFE_FRAC_PI_2 {
                camera.pitch = -SAFE_FRAC_PI_2;
            } else if camera.pitch > SAFE_FRAC_PI_2 {
                camera.pitch = SAFE_FRAC_PI_2;
            }
        }
    }

    #[repr(C)]
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CameraUniform {
        view_pos: Vec4,
        view_mat: Mat4,
    }
    impl CameraUniform {
        pub fn new() -> Self {
            Self {
                view_pos: Vec4::new(0.0,0.0,0.0,0.0),
                view_mat: Mat4::IDENTITY,
            }
        }
        pub fn update_view_project(&mut self, camera: &Camera, project_mat: Mat4) {
            self.view_pos = (camera.position, 0.0).into();
            self.view_mat = (project_mat * camera.view_mat()).into();
        }

        pub fn update_model_view_project(
            &mut self,
            camera: &Camera,
            project_mat: Mat4,
            model_mat: Mat4,
        ) {
            self.view_mat = (project_mat * camera.view_mat() * model_mat).into();
        }
    }
}

/// Some common utilities.
pub mod utils {

    /// Utility to convert Rust `&str` into wide UTF-16 string.
    pub trait ToWide {
        fn to_wide(&self) -> *mut u16;
    }

    impl ToWide for &str {
        fn to_wide(&self) -> *mut u16 {
            let mut result: Vec<u16> = self.encode_utf16().collect();
            result.push(0);
            result.as_ptr() as *mut u16
        }
    }

    pub fn as_bytes<T: ?Sized>(content: &T) -> &[u8] {
        let new_len = core::mem::size_of_val(content) / std::mem::size_of::<u8>();
        unsafe { core::slice::from_raw_parts(content as *const T as *const u8, new_len) }
    }
}
