/// Module for creating and managing a PiCa window
pub mod pica_window {

    use crate::utils::*;
    use std::ffi::c_void;
    use windows::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, PWSTR, RECT, WPARAM},
        Graphics::Gdi::GetDC,
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{ConvertThreadToFiber, CreateFiber},
        },
        UI::WindowsAndMessaging::{
            AdjustWindowRect, CreateWindowExW, LoadCursorW, RegisterClassW, SetWindowLongPtrW,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_CROSS, WNDCLASSW,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE, GetWindowLongPtrW, DefWindowProcW,
        },
    };

    /// Wrapper type around [`Error`]
    use crate::error::Error;
    pub type Result<T> = std::result::Result<T, crate::error::Error>;

    /// Window Attributes for creating a new PiCa window.
    pub struct WindowAttributes {
        pub title: String,
        pub position: (i32, i32),
        pub size: (i32, i32),
    }

    impl WindowAttributes {
        /// Create default window attributes
        pub fn new() -> Self {
            Self {
                title: "PiCa Window".to_owned(),
                position: (0, 0),
                size: (0, 0),
            }
        }

        /// Set title for new PiCa window.
        pub fn with_title(mut self, title: &str) -> Self {
            self.title = title.to_owned();
            self
        }

        /// Set position for new PiCa window.
        pub fn with_position(mut self, x_pos: isize, y_pos: isize) -> Self {
            self.position = (x_pos as i32, y_pos as i32);
            self
        }

        /// Set size for new PiCa window.
        pub fn with_size(mut self, x_size: isize, y_size: isize) -> Self {
            self.size = (x_size as i32, y_size as i32);
            self
        }
    }

    struct Win32 {
        win32_window_handle: isize,
        win32_device_context: isize,
        main_fiber: *mut c_void,
        message_fiber: *mut c_void,
    }

    pub struct Window {
        window_attributes: WindowAttributes,
        win32: Win32,
    }

    impl Window {
        // Create window with default window attributes.
        pub fn new() -> Result<Self> {
            let window_attributes = WindowAttributes::new();
            Self::new_with_attributes(window_attributes)
        }

        // // Create window with provided window attributes.
        pub fn new_with_attributes(window_attributes: WindowAttributes) -> Result<Self> {
            let main_fiber = unsafe { ConvertThreadToFiber(0 as *const c_void) };
            assert!(!main_fiber.is_null());

            // Calculates the required size of the window rectangle, based on the desired client-rectangle size.
            // Returns default values when calculation fails.
            let window_size: (i32, i32) = if window_attributes.size != (0, 0) {
                let mut window_rectangle = RECT {
                    left: 0,
                    top: 0,
                    right: window_attributes.size.1,
                    bottom: window_attributes.size.0,
                };
                if unsafe {
                    AdjustWindowRect(&mut window_rectangle, WS_OVERLAPPEDWINDOW, None).as_bool()
                } {
                    let window_width = window_rectangle.right - window_rectangle.left;
                    let window_height = window_rectangle.bottom - window_rectangle.top;
                    (window_width, window_height)
                } else {
                    (CW_USEDEFAULT, CW_USEDEFAULT)
                }
            } else {
                (CW_USEDEFAULT, CW_USEDEFAULT)
            };

            let window_position: (i32, i32) = match window_attributes.position {
                (0, 0) => (CW_USEDEFAULT, CW_USEDEFAULT),
                _ => window_attributes.position,
            };

            let instance = unsafe { GetModuleHandleW(None) };

            let window_class = {
                unsafe {
                    WNDCLASSW {
                        hCursor: LoadCursorW(None, IDC_CROSS),
                        hInstance: instance,
                        lpszClassName: PWSTR("pica".to_wide()),

                        style: CS_HREDRAW | CS_VREDRAW,
                        lpfnWndProc: Some(Self::wndproc),
                        ..Default::default()
                    }
                }
            };

            if unsafe { RegisterClassW(&window_class) } == 0 {
                return Err(Error::Window(
                    "Failed to register win32 window class.".to_owned(),
                ));
            }

            let win32_window_handle = unsafe {
                CreateWindowExW(
                    Default::default(),
                    PWSTR("pica".to_wide()),
                    PWSTR((&window_attributes.title[..]).to_wide()),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    window_position.0,
                    window_position.1,
                    window_size.0,
                    window_size.1,
                    None,
                    None,
                    instance,
                    0 as *const c_void,
                )
            };
            // Note Geert: Unsure if I can get a valid device context here, or shouild wait after showing the window?
            let win32_device_context = unsafe { GetDC(win32_window_handle) };
            if win32_device_context == 0 {
                return Err(Error::Window(
                    "Failed to get Device Context during window creation.".to_owned(),
                ));
            }

            let mut pica_window = Self {
                window_attributes,
                win32: Win32 {
                    win32_window_handle,
                    win32_device_context,
                    main_fiber,
                    message_fiber: 0 as *mut c_void,
                },
            };

            unsafe {
                SetWindowLongPtrW(
                    pica_window.win32.win32_window_handle,
                    GWLP_USERDATA,
                    (&mut pica_window) as *mut Self as isize,
                )
            };

            pica_window.win32.message_fiber = unsafe {
                CreateFiber(
                    0,
                    Some(Self::message_fiber_proc),
                    &mut pica_window as *const _ as *const c_void,
                )
            };
            assert!(!pica_window.win32.message_fiber.is_null());

            Ok(pica_window)
        }

        // Win32 message handling
        extern "system" fn wndproc(
            window_handle: HWND,
            message: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            unsafe {
                println!("wndproc: {}",  GetWindowLongPtrW(window_handle, GWLP_USERDATA));
                let pica_window = GetWindowLongPtrW(window_handle, GWLP_USERDATA) as *mut Self;
                if !pica_window.is_null() {
                    println!("HURRAAI!");
                }

                // TODO Geert: Implement win32 message handling.
                // DefWindowProcW(window_handle, message, wparam, lparam)
                0
                
            }
        }

        // Win32 message loop
        extern "system" fn message_fiber_proc(data: *mut c_void) {
            // data is actually a pointer to our initialized pica_window::Window struct
            let pica_window: *mut Self = unsafe { std::mem::transmute(data) };
            assert!(!pica_window.is_null());
            // TODO Geert: Implement the win32 message loop
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

    impl error::Error for Win32Error {}
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
}
