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
        match window.mouse {
            
            Mouse {left_button: Button {pressed: true, ..}, ..} => {println!("LEFT BUTTON PRESSED")},
            Mouse {right_button: Button {pressed: true, ..}, ..} => {println!("RIGHT BUTTON PRESSED")},
            Mouse {left_button: Button {released: true, ..}, ..} => {println!("LEFT BUTTON RELEASED")},
            Mouse {right_button: Button {released: true, ..}, ..} => {println!("RIGHT BUTTON RELEASED")},
            _ => {},
        }
        if window.keys['A' as u8 as usize].pressed {
            println!("TrIggErED!!");
        }
    }
    Ok(())
}
