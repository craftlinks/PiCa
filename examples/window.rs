use PiCa::pica_mouse::{Mouse, Button};
use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::error::Error;

pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("Awesome PiCa Simulation")
        .with_position(50, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    while window.pull() {
        // print!("\rwindow position: {}, {}  ", window.window_attributes.position.0, window.window_attributes.position.1);
        // print!("window size: {}, {}  ", window.window_attributes.size.0, window.window_attributes.size.1);
        // print!("mouse position: {}, {}  ",window.mouse.position.0, window.mouse.position.1);
        match window.mouse {
            
            Mouse {left_button: Button {pressed: true, ..}, ..} => {println!("LEFT BUTTON PRESSED")},
            Mouse {right_button: Button {pressed: true, ..}, ..} => {println!("RIGHT BUTTON PRESSED")},
            Mouse {left_button: Button {released: true, ..}, ..} => {println!("LEFT BUTTON RELEASED")},
            Mouse {right_button: Button {released: true, ..}, ..} => {println!("RIGHT BUTTON RELEASED")},
            _ => {},
        }
        print!("\r{}",  std::str::from_utf8(&window.text).unwrap());
        // if window.mouse.wheel != 0 {
        //     print!("WHEEL: {}    ", window.mouse.wheel);
        // }
            


        // let variable = window.time.delta_nanoseconds;
        // dbg!("elapsed seconds: {}", variable );
    }
    Ok(())
}
