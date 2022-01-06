use PiCa::pica_window::{Window, WindowAttributes};

pub fn main() {
    let window_attributes = WindowAttributes::new()
        .with_title("Awesome PiCa Simulation")
        .with_position(50, 50)
        .with_size(800, 600);

    let window = Window::new_with_attributes(window_attributes);
}
