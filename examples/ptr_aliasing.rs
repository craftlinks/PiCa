use std::{ptr, alloc::{dealloc, Layout}};

// This program has UB as indicated by: cargo miri run --example ptr_aliasing
// The problems with ptr aliasing ("rebowering", or "stacked borrows") are discussed
// https://www.ralfj.de/blog/2018/08/07/stacked-borrows.html
// Note that the current Rust compiler won't give problems with this, yet.


pub struct Test{pub eee: *mut i32}

pub fn ub() {
    let y: *mut i32 = {
        let x = Box::into_raw(Box::new(42));
        x
    };

    let test = Test { eee: y };

    assert_eq!(unsafe { *y }, 42);

    let mut z = unsafe { Box::from_raw(y) };
    *z += 1;

    println!("{}", *z);

    unsafe { (*test.eee) += 1 };

    println!("{}", unsafe{*test.eee});
}

pub fn db() {
    let y: *mut i32 = {
        let x = Box::into_raw(Box::new(42));
        x
    };

    let test = Test { eee: y };

    assert_eq!(unsafe { *y }, 42);

    unsafe{*y += 1};

    println!("{}", unsafe{*y});

    unsafe { (*test.eee) += 1 };

    println!("{}", unsafe{*test.eee});

    // Converting the raw pointer back into a Box with Box::from_raw for automatic cleanup:
    unsafe { Box::from_raw(y) };

    // Or, Manual cleanup by explicitly running the destructor and deallocating the memory:
    // unsafe {
    //     ptr::drop_in_place(y);
    //     dealloc(y as *mut u8, Layout::new::<i32>());
    // }
}

pub fn main() {
    // ub(); --> Undefined Behavior detected when run with Miri.
    db();
}
