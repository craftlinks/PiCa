use std::ffi::c_void;

use windows::Win32::System::Threading::{ConvertThreadToFiber, CreateFiber, SwitchToFiber};

struct Fiber {
    pub main_fiber: *mut c_void,
    pub x: i32,
}

pub fn main() {
    let x = 0;

    let main_fiber = unsafe { ConvertThreadToFiber(0 as *const c_void) };

    let mut fiber_data = Fiber{main_fiber, x};

    assert!(!main_fiber.is_null());
    
    let work_fiber = unsafe { CreateFiber(
                    0,
                    Some(worker_fiber_proc),
                    &mut fiber_data as *mut Fiber as *const c_void ,
                )};
    
    while fiber_data.x <= 10 {
        println!("fiber_data: {}", fiber_data.x);
        unsafe {SwitchToFiber(work_fiber as *const c_void)}
    }
}

extern "system" fn worker_fiber_proc(data: *mut c_void) {
    let fiber_data = data.cast::<Fiber>();
    let fiber_data: &mut Fiber = unsafe{fiber_data.as_mut().unwrap()};
    loop {
        fiber_data.x = fiber_data.x + 1;
        unsafe {SwitchToFiber(fiber_data.main_fiber);}
    }
}
