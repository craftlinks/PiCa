use pica::pica_window::{WindowAttributes, Window};
use pica::error::Error;

pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("WebGPU Mesh")
        .with_position(50, 50)
        .with_size(1600, 2000);

    let mut window = Window::new_with_attributes(window_attributes)?;

    while window.pull() {
    
    }

    Ok(())

}
