/// Module for creating and managing a PiCa window
pub mod pica_window {

    use crate::{pica_time::Time, utils::*};
    use std::ffi::c_void;
    use windows::Win32::{
        Foundation::{GetLastError, SetLastError, HWND, LPARAM, LRESULT, PWSTR, RECT, WPARAM},
        Graphics::Gdi::GetDC,
        System::{
            LibraryLoader::GetModuleHandleW,
            Performance::QueryPerformanceCounter,
            Threading::{ConvertThreadToFiber, CreateFiber, SwitchToFiber},
        },
        UI::WindowsAndMessaging::{
            AdjustWindowRect, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetWindowLongPtrW,
            LoadCursorW, PeekMessageW, RegisterClassW, SetTimer, SetWindowLongPtrW,
            TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_CROSS, MSG,
            PM_REMOVE, WM_DESTROY, WM_QUIT, WM_SIZE, WM_TIMER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
            WS_VISIBLE,
        },
    };

    /// Wrapper type around [`Error`]
    use crate::error::Error;
    pub type Result<T> = std::result::Result<T, crate::error::Error>;

    /// Window Attributes for creating a new PiCa window.
    #[derive(Debug)]
    pub struct WindowAttributes {
        pub title: String,
        pub position: (i32, i32),
        pub size: (i32, i32),
        pub resized: bool,
    }

    impl WindowAttributes {
        /// Create default window attributes
        pub fn new() -> Self {
            Self {
                title: "PiCa Window".to_owned(),
                position: (0, 0),
                size: (0, 0),
                resized: false,
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

    #[derive(Debug)]
    struct Win32 {
        win32_window_handle: isize,
        win32_device_context: isize,
        main_fiber: *mut c_void,
        message_fiber: *mut c_void,
    }

    #[derive(Debug)]
    pub struct Window {
        window_attributes: WindowAttributes,
        win32: Win32,
        pub time: Time,
        quit: bool,
    }

    impl Window {
        // Create window with default window attributes.
        pub fn new() -> Result<Self> {
            let window_attributes = WindowAttributes::new();
            Self::new_with_attributes(window_attributes)
        }

        // // Create window with provided window attributes.
        pub fn new_with_attributes(window_attributes: WindowAttributes) -> Result<Self> {
            let instance = unsafe { GetModuleHandleW(None) };
            let window_class_name = "pica".to_wide();

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

            println!("window size {:?}", window_size);

            let window_position: (i32, i32) = match window_attributes.position {
                (0, 0) => (CW_USEDEFAULT, CW_USEDEFAULT),
                _ => window_attributes.position,
            };

            let window_class = {
                unsafe {
                    WNDCLASSW {
                        hCursor: LoadCursorW(None, IDC_CROSS),
                        hInstance: instance,
                        lpszClassName: PWSTR(window_class_name),

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
                    PWSTR(window_class_name),
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
            if win32_window_handle == 0 {
                println!("Failed to create window, with error code {}", unsafe {
                    GetLastError()
                })
            }
            debug_assert!(win32_window_handle != 0);

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
                time: Time::new(),
                quit: false,
            };

            unsafe {
                SetLastError(0);
                if SetWindowLongPtrW(
                    pica_window.win32.win32_window_handle,
                    GWLP_USERDATA,
                    (&mut pica_window) as *mut Self as isize,
                ) == 0
                    && GetLastError() != 0
                {
                    let error = GetLastError();
                    println!(
                        "Error settting userdata for window handle {}, error code: {}",
                        pica_window.win32.win32_window_handle, error
                    );
                }
            };

            pica_window.win32.message_fiber = unsafe {
                CreateFiber(
                    0,
                    Some(Self::message_fiber_proc),
                    &mut pica_window as *const Window as *const c_void,
                )
            };
            assert!(!pica_window.win32.message_fiber.is_null());

            pica_window.pull();
            Ok(pica_window)
        }

        pub fn pull(&mut self) -> bool {
            self.window_pull();
            self.time_pull();
            !self.quit

        }

        fn window_pull(&mut self) {
            println!("Window Pull");
            unsafe {
                SwitchToFiber(self.win32.message_fiber);
            }
        }

        fn time_pull(&mut self) {
            let mut current_ticks: i64 = 0;
            println!("TIME PULL");
            unsafe {
                if !QueryPerformanceCounter(&mut current_ticks).as_bool() {
                    let error = GetLastError();
                    println!("Error getting performance count: {}", error);
                } 
            }
            
            // Calculate ticks
            self.time.delta_ticks = (current_ticks - self.time.initial_ticks) - self.time.ticks;
            self.time.ticks = current_ticks - self.time.initial_ticks;

            self.time.delta_nanoseconds = (1000 * 1000 * 1000 * self.time.delta_ticks) / self.time.ticks_per_second;
            self.time.delta_microseconds = self.time.delta_nanoseconds / 1000;
            self.time.delta_milliseconds = self.time.delta_microseconds / 1000;
            self.time.seconds = self.time.delta_ticks as f32 / self.time.ticks_per_second as f32;

            self.time.nanoseconds =
                (1000 * 1000 * 1000 * self.time.ticks) / self.time.ticks_per_second;
            self.time.microseconds = self.time.nanoseconds / 1000;
            self.time.milliseconds = self.time.microseconds / 1000;
            self.time. seconds = self.time.ticks as f32 / self.time.ticks_per_second as f32;

            println!("   {:?}", self);
            
        }

        // Win32 message handling
        extern "system" fn wndproc(
            window_handle: HWND,
            message: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            unsafe {
                let pica_window = GetWindowLongPtrW(window_handle, GWLP_USERDATA) as *mut Self;
                if pica_window.is_null() {
                    return DefWindowProcW(window_handle, message, wparam, lparam);
                }
                match message {
                    WM_DESTROY => {
                        (*pica_window).quit = true;
                        println!("WM_DESTROY");
                        0
                    }

                    WM_TIMER => {
                        println!("WM_TIME");
                        SwitchToFiber((*pica_window).win32.main_fiber);
                        0
                    }

                    WM_SIZE => {
                        (*pica_window).window_attributes.resized = true;
                        println!("WM_SIZE");
                        0
                    }

                    _ => DefWindowProcW(window_handle, message, wparam, lparam),
                }
            }
        }

        // Win32 message loop
        extern "system" fn message_fiber_proc(data: *mut c_void) {

            // data is actually a pointer to our initialized pica_window::Window struct
            let pica_window: *mut Self = unsafe { std::mem::transmute(data) };
            assert!(!pica_window.is_null());
            unsafe {
                SetTimer((*pica_window).win32.win32_window_handle, 1, 1, None)
            };
            loop {
                unsafe {
                    let mut message = MSG::default();
                    while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).into() {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                    println!("MESS_FIBER -> MAIN_FIBER");
                    println!("  {:?}", (*pica_window));
                    println!("  Main Fiber pointer: {:?}", (*pica_window).win32.main_fiber);
                    SwitchToFiber((*pica_window).win32.main_fiber);
                }
            }
        }
    }
}

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
