pub mod math;
pub mod renderer;

/// Module for creating and managing a PiCa window
pub mod pica_window {

    use crate::{
        pica_mouse::{Button, Mouse},
        pica_time::Time,
        utils::*, renderer::{D3D12},
    };
    use std::{ffi::c_void, mem::size_of};
    use windows::Win32::{
        Devices::HumanInterfaceDevice::MOUSE_MOVE_RELATIVE,
        Foundation::{
            GetLastError, SetLastError, HWND, LPARAM, LRESULT, POINT, PSTR, PWSTR, RECT, WPARAM,
        },
        Globalization::{WideCharToMultiByte, CP_ACP},
        Graphics::Gdi::{ClientToScreen, GetDC, HDC},
        System::{
            LibraryLoader::GetModuleHandleW,
            Performance::QueryPerformanceCounter,
            Threading::{ConvertThreadToFiber, CreateFiber, SwitchToFiber},
        },
        UI::{
            Input::{GetRawInputData, RAWINPUT, RAWINPUTHEADER, RID_INPUT, RIM_TYPEMOUSE, KeyboardAndMouse::GetKeyboardState, HRAWINPUT},
            WindowsAndMessaging::{
                AdjustWindowRect, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
                GetCursorPos, GetWindowLongPtrW, LoadCursorW, PeekMessageW, RegisterClassW,
                SetTimer, SetWindowLongPtrW, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
                CW_USEDEFAULT, GWLP_USERDATA, IDC_CROSS, MSG, PM_REMOVE, RI_MOUSE_LEFT_BUTTON_DOWN,
                RI_MOUSE_LEFT_BUTTON_UP, RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP,
                RI_MOUSE_WHEEL, WHEEL_DELTA, WM_CHAR, WM_DESTROY, WM_INPUT, WM_SIZE, WM_TIMER,
                WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
            },
        },
    };

    /// Wrapper type around [`Error`]
    use crate::error::Error;
    pub type Result<T> = std::result::Result<T, crate::error::Error>;

    const MAX_KEYS: usize = 256;
    const MAX_TEXT: usize = 256;
    pub const ALT: usize = 0x12;
    pub const CTR: usize = 0x11;
    pub const SHIFT: usize = 0x10;

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
        main_fiber: *mut c_void,
        message_fiber: *mut c_void,
        win32_window_handle: HWND,
        win32_device_context: HDC,
    }

    #[derive(Debug)]
    pub struct Window {
        win32: Win32,
        d3d12: D3D12,
        pub window_attributes: WindowAttributes,
        pub mouse: Mouse,
        pub keys: [Button; 256],
        pub time: Time,
        pub text: [char; MAX_TEXT],
        pub text_length: usize,
        quit: bool,
    }

    unsafe impl raw_window_handle::HasRawWindowHandle for Window {
        fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
            let mut handle = raw_window_handle::Win32Handle::empty();
            handle.hwnd = self.win32.win32_window_handle.0 as *mut c_void;
            handle.hinstance = unsafe { GetModuleHandleW(None).0 } as *mut c_void;
            raw_window_handle::RawWindowHandle::Win32(handle)
        }
    }

    impl Window {
        // Create window with default window attributes.
        pub fn new() -> Result<Box<Self>> {
            let window_attributes = WindowAttributes::new();
            Self::new_with_attributes(window_attributes)
        }

        // // Create window with provided window attributes.
        pub fn new_with_attributes(window_attributes: WindowAttributes) -> Result<Box<Self>> {
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
            if win32_window_handle.0 == 0 {
                println!("Failed to create window, with error code {}", unsafe {
                    GetLastError()
                })
            }
            debug_assert!(win32_window_handle.0 != 0);

            // Note Geert: Unsure if I can get a valid device context here, or shouild wait after showing the window?
            let win32_device_context = unsafe { GetDC(win32_window_handle) };
            if win32_device_context.0 == 0 {
                return Err(Error::Window(
                    "Failed to get Device Context during window creation.".to_owned(),
                ));
            }

            let mouse = Mouse::new(win32_window_handle)?;
            
            // Direct3D12 Initialization
            let mut d3d12 = D3D12::new()?;
            d3d12.create_resources(win32_window_handle, window_attributes.size)?;

            // Create PiCa window struct
            let mut pica_window = Box::into_raw(Box::new(Self {
                window_attributes,
                win32: Win32 {
                    win32_window_handle: win32_window_handle,
                    win32_device_context: win32_device_context,
                    main_fiber,
                    message_fiber: 0 as *mut c_void,
                },
                mouse,
                keys: [Button::default(); MAX_KEYS],
                time: Time::new(),
                text: ['0'; MAX_TEXT],
                text_length: 0,
                quit: false,
                d3d12,
            }));

            unsafe {
                SetLastError(0);
                if SetWindowLongPtrW(
                    (*pica_window).win32.win32_window_handle,
                    GWLP_USERDATA,
                    pica_window as isize,
                ) == 0
                    && GetLastError() != 0
                {
                    let error = GetLastError();
                    println!(
                        "Error settting userdata for window handle {}, error code: {}",
                        (*pica_window).win32.win32_window_handle.0,
                        error
                    );
                }

                (*pica_window).win32.message_fiber = CreateFiber(
                    0,
                    Some(Self::message_fiber_proc),
                    pica_window as *const c_void,
                );

                assert!(!(*pica_window).win32.message_fiber.is_null());

                // Note Geert: Unfortunately this pointer aliasing is undefined behavior.
                // Should just return the raw pointer and live with undefined beh in application code...
                // However, this wil behave as expected with current Rust compiler version, so I chose ergonomics.
                let mut pica_window = Box::from_raw(pica_window);

                pica_window.pull();
                Ok(pica_window)
            }
        }

        pub fn pull(&mut self) -> bool {
            self.window_pull();
            self.time_pull();
            self.keyboard_pull();
            self.mouse_pull();
            !self.quit
        }

        fn window_pull(&mut self) {
            
            self.text[0] = '0';
            self.text_length = 0;
            
            self.window_attributes.resized = false;
            self.mouse.delta_position.0 = 0;
            self.mouse.delta_position.1 = 0;
            self.mouse.delta_wheel = 0;
            self.mouse.left_button.pressed = false;
            self.mouse.left_button.released = false;
            self.mouse.right_button.pressed = false;
            self.mouse.right_button.released = false;

            unsafe {
                SwitchToFiber(self.win32.message_fiber as *const c_void);
            }

            let mut client_rect = RECT::default();
            unsafe { GetClientRect(self.win32.win32_window_handle, &mut client_rect) };

            self.window_attributes.size.0 = client_rect.right - client_rect.left;
            self.window_attributes.size.1 = client_rect.bottom - client_rect.top;

            let mut window_position = POINT {
                x: client_rect.left,
                y: client_rect.top,
            };
            unsafe { ClientToScreen(self.win32.win32_window_handle, &mut window_position) };

            self.window_attributes.position.0 = window_position.x;
            self.window_attributes.position.1 = window_position.y;
        }

        fn time_pull(&mut self) {
            let mut current_ticks: i64 = 0;
            unsafe {
                if !QueryPerformanceCounter(&mut current_ticks).as_bool() {
                    let error = GetLastError();
                    println!("Error getting performance count: {}", error);
                }
            }

            // Calculate ticks
            self.time.delta_ticks = (current_ticks - self.time.initial_ticks) - self.time.ticks;
            self.time.ticks = current_ticks - self.time.initial_ticks;

            self.time.delta_nanoseconds =
                (1000 * 1000 * 1000 * self.time.delta_ticks) / self.time.ticks_per_second;
            self.time.delta_microseconds = self.time.delta_nanoseconds / 1000;
            self.time.delta_milliseconds = self.time.delta_microseconds / 1000;
            self.time.seconds = self.time.delta_ticks as f32 / self.time.ticks_per_second as f32;

            self.time.nanoseconds =
                (1000 * 1000 * 1000 * self.time.ticks) / self.time.ticks_per_second;
            self.time.microseconds = self.time.nanoseconds / 1000;
            self.time.milliseconds = self.time.microseconds / 1000;
            self.time.seconds = self.time.ticks as f32 / self.time.ticks_per_second as f32;
        }

        fn keyboard_pull(&mut self) {
            let keyboard_state:&mut [u8;256] = &mut [0;256];
            unsafe { GetKeyboardState(keyboard_state.as_mut_ptr())};
            for key in 0..256 {
                self.keys[key].update_button((keyboard_state[key] >> 7) == 1);
            }
        }

        fn mouse_pull(&mut self) {
            let mut mouse_position = POINT::default();
            unsafe {
                GetCursorPos(&mut mouse_position);
            }
            mouse_position.x -= self.window_attributes.position.0;
            mouse_position.y -= self.window_attributes.position.1;
            self.mouse.position.0 = mouse_position.x;
            self.mouse.position.1 = mouse_position.y;
        }

        pub fn push(&mut self) {
            self.d3d12.render();
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
                let mut pica_window = &mut *pica_window;
                match message {
                    WM_INPUT => {
                        let mut size: u32 = 0;
                        GetRawInputData(
                            HRAWINPUT(lparam.0),
                            RID_INPUT,
                            0 as *mut c_void,
                            &mut size as *mut u32,
                            size_of::<RAWINPUTHEADER>() as u32,
                        );
                        let mut buffer: Vec<u8> = vec![0; size as usize];
                        if GetRawInputData(
                            HRAWINPUT(lparam.0),
                            RID_INPUT,
                            buffer[..].as_mut_ptr() as *mut c_void,
                            &mut size as *mut u32,
                            size_of::<RAWINPUTHEADER>() as u32,
                        ) == size
                        {
                            let raw_input: RAWINPUT = *(buffer.as_ptr().cast::<RAWINPUT>());
                            if raw_input.header.dwType == RIM_TYPEMOUSE
                                && raw_input.data.mouse.usFlags == MOUSE_MOVE_RELATIVE as u16
                            {
                                pica_window.mouse.delta_position.0 += raw_input.data.mouse.lLastX;
                                pica_window.mouse.delta_position.1 += raw_input.data.mouse.lLastY;

                                let button_flags =
                                    raw_input.data.mouse.Anonymous.Anonymous.usButtonFlags;

                                let mut left_button_down = (pica_window).mouse.left_button.down;
                                if button_flags as u32 & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
                                    left_button_down = true
                                };
                                if button_flags as u32 & RI_MOUSE_LEFT_BUTTON_UP != 0 {
                                    left_button_down = false
                                };

                                pica_window
                                    .mouse
                                    .left_button
                                    .update_button(left_button_down);

                                let mut right_button_down = pica_window.mouse.right_button.down;
                                if button_flags as u32 & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
                                    right_button_down = true
                                };
                                if button_flags as u32 & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
                                    right_button_down = false
                                };

                                pica_window
                                    .mouse
                                    .right_button
                                    .update_button(right_button_down);

                                // Alternative syntax, no opinions on what's "cleanest"
                                // right_button_down = match button_flags {
                                //     _ if button_flags.into() & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 => true,
                                //     _ if button_flags.into() & RI_MOUSE_RIGHT_BUTTON_UP != 0 => false,
                                //     _ => right_button_down
                                // };

                                if button_flags as u32 & RI_MOUSE_WHEEL != 0 {
                                    pica_window.mouse.delta_wheel +=
                                        raw_input.data.mouse.Anonymous.Anonymous.usButtonData
                                            as i32
                                            / WHEEL_DELTA as i32;
                                    pica_window.mouse.wheel += pica_window.mouse.delta_wheel;
                                }
                            }
                        }

                        LRESULT(0)
                    }

                    WM_CHAR => {
                        let mut utf16_character: u16 = wparam.0 as u16;
                        let mut ascii_character: u8 = 0;
                        let ascii_length = WideCharToMultiByte(
                            CP_ACP,
                            0,
                            PWSTR(&mut utf16_character),
                            1,
                            PSTR(&mut ascii_character),
                            1,
                            PSTR(0 as *mut u8),
                            0 as *mut i32,
                        );
                        if ascii_length == 1 && pica_window.text_length + 1 < size_of::<[u8; MAX_TEXT]>() - 1 {
                            pica_window.text[pica_window.text_length] = ascii_character as char;
                            pica_window.text[pica_window.text_length + 1] = '0';
                            pica_window.text_length += ascii_length as usize; 

                        }
                        LRESULT(0)
                    }

                    WM_DESTROY => {
                        pica_window.quit = true;
                        println!("WM_DESTROY");
                        LRESULT(0)
                    }
                    
                    /* WM_PAINT |*/ WM_TIMER => {
                        // Required to break out recursive message loops, so our main thread gets time to run!
                        SwitchToFiber(pica_window.win32.main_fiber);
                        LRESULT(0)
                    }

                    WM_SIZE => {
                        pica_window.window_attributes.resized = true;
                        // println!("WM_SIZE");
                        LRESULT(0)
                    }

                    _ => DefWindowProcW(window_handle, message, wparam, lparam),
                }
            }
        }

        // Win32 message loop
        extern "system" fn message_fiber_proc(data: *mut c_void) {
            // data is actually a pointer to our initialized pica_window::Window struct
            let pica_window: *mut Self = data.cast::<Self>();
            assert!(!pica_window.is_null());
            let pica_window: &mut Self = unsafe { pica_window.as_mut().unwrap() };
            println!(
                "First entry into message fiber: Main Fiber pointer: {:?}",
                (pica_window).win32.main_fiber
            );
            unsafe { SetTimer(pica_window.win32.win32_window_handle, 1, 1, None) };
            loop {
                unsafe {
                    let mut message = MSG::default();
                    while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).into() {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                    SwitchToFiber(pica_window.win32.main_fiber);
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

pub mod pica_mouse {
    use crate::error::Error;
    use std::mem::size_of;
    use windows::Win32::{UI::Input::{RegisterRawInputDevices, RAWINPUTDEVICE}, Foundation::HWND};
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
        pub fn new(win32_window_handle: HWND) -> Result<Self> {
            // TODO: Geert: You will need to Box this for sure!!
            let raw_input_device = Box::into_raw(Box::new(RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x02,
                dwFlags: 0,
                hwndTarget: win32_window_handle,
            }));
            if unsafe {
                !RegisterRawInputDevices(raw_input_device, 1, size_of::<RAWINPUTDEVICE>() as u32)
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
