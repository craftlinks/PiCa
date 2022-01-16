use std::ffi::c_void;

use windows::Win32::System::Threading::{ConvertThreadToFiber, CreateFiber, SwitchToFiber};

#[derive(Debug)]
struct InnerStruct {
    _title: String,
    _position: (i32, i32),
    main_fiber: *mut c_void,
    work_fiber: *mut c_void,
}

#[derive(Debug)]
struct Fiber {
    inner: InnerStruct,
    x: i32,
}

impl Fiber {
    pub fn new() -> Result<Box<Self>,()> {
        let x = 0;
        let main_fiber = unsafe { ConvertThreadToFiber(0 as *const c_void) };
        assert!(!main_fiber.is_null());
        let mut fiber_data = Self {
            inner: InnerStruct {
                _title: "TEST".to_owned(),
                _position: (100, 100),
                main_fiber,
                work_fiber: 0 as *mut c_void,
            },
            x,
        };

        let mut boxed_fiber = Box::new(fiber_data);
        let work_fiber = unsafe {
            CreateFiber(
                0,
                Some(worker_fiber_proc),
                boxed_fiber.as_mut() as *mut Fiber as *const c_void,
            )
        };
        assert!(!work_fiber.is_null());
        boxed_fiber.inner.work_fiber = work_fiber;
        Ok(boxed_fiber)
    }
}

pub fn main() -> Result<(), ()> {
    let fiber_data = Fiber::new()?;
    while fiber_data.x < 10 {
        
        unsafe {
            SwitchToFiber(fiber_data.inner.work_fiber as *const c_void)
        }
        println!("{}", fiber_data.x);
    }
    Ok(())
}

extern "system" fn worker_fiber_proc(data: *mut c_void) {
    let fiber_data = data.cast::<Fiber>();
    let fiber_data: &mut Fiber = unsafe { fiber_data.as_mut().unwrap() };
    loop {
        fiber_data.x = fiber_data.x + 1;
        unsafe {
            SwitchToFiber(fiber_data.inner.main_fiber);
        }
    }
}
