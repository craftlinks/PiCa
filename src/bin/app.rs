use pica::{self, pica_window::WindowAttributes, PiCa};

pub fn main() {
    // Create a PiCa aplication instance
    let mut app = PiCa::new().expect("Failed to create PiCa application");

    // Optional: set windows attributes
    let window_attributes =
        WindowAttributes::new()
            .with_title("Cube Color")
            .with_position(50, 50)
            .with_size(1600, 2000);

    app.set_window_attributes(window_attributes);

    // Optional: set renderer attributes
    // TODO let renderer_attributes = Attributes::RendererAttributes(());
    // app.set_renderer_attributes(renderer_attributes);

    while app.pull() {}
}
