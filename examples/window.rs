use PiCa::pica_window::{Window, WindowAttributes};
use PiCa::error::Error;

pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("Awesome PiCa Simulation")
        .with_position(50, 50)
        .with_size(800, 600);

    let mut window = Window::new_with_attributes(window_attributes)?;

    while window.pull() {
        println!("window position: {}, {}", window.window_attributes.position.0, window.window_attributes.position.1);
        println!("window size: {}, {}", window.window_attributes.size.0, window.window_attributes.size.1);

        // let variable = window.time.delta_nanoseconds;
        // dbg!("elapsed seconds: {}", variable );
    }
    Ok(())
}
