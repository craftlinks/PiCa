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
    use std::f32::consts::PI;

    use glam::{Mat4, Vec3};

    pub struct Camera {
        pub position: Vec3,
        yaw: f32,
        pitch: f32,
    }

    impl Camera {
        pub fn new<Pt: Into<Vec3>, Yaw: Into<f32>, Pitch: Into<f32>>(
            position: Pt,
            yaw: Yaw,     // horizontal rotation
            pitch: Pitch, // vertical rotation
        ) -> Self {
            Self {
                position: position.into(),
                yaw: yaw.into().to_radians(), // is this in degrees or RAD!?
                pitch: pitch.into().to_radians(),
            }
        }

        pub fn view_mat(&self) -> Mat4 {
            Mat4::look_at_rh(
                self.position,
                Vec3::new(
                    self.pitch.cos() * self.yaw.cos(),
                    self.pitch.sin(),
                    self.pitch.cos() * self.yaw.sin(),
                )
                .normalize(),
                Vec3::Y,
            )
        }
    }

    pub struct CameraController {
        rotatex: f32,
        rotatey: f32,
        speed: f32,
    }

    impl CameraController {
        pub fn new(speed: f32) -> Self {
            Self {
                rotatex: 0.0,
                rotatey: 0.0,
                speed,
            }
        }

        pub fn mouse_move(&mut self, mousex: f64, mousey: f64) {
            self.rotatex = mousex as f32;
            self.rotatey = mousey as f32;
        }

        pub fn update_camera(&mut self, camera: &mut Camera) {
            camera.yaw += self.rotatex * self.speed;
            camera.pitch += self.rotatey * self.speed;
            self.rotatex = 0.0;
            self.rotatey = 0.0;
            if camera.pitch < -(89.0 * PI / 180.0) {
                camera.pitch = -(89.0 * PI / 180.0);
            } else if camera.pitch > (89.0 * PI / 180.0) {
                camera.pitch = 89.0 * PI / 180.0;
            }
        }
    }

    #[repr(C)]
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CameraUniform {
        view_mat: Mat4,
    }
    impl CameraUniform {
        pub fn new() -> Self {
            Self {
                view_mat: Mat4::IDENTITY,
            }
        }
        pub fn update_view_project(&mut self, camera: &Camera, project_mat: Mat4) {
            self.view_mat = (project_mat * camera.view_mat()).into()
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
