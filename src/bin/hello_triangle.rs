use std::time::{Duration, Instant};

use PiCa::error::Error;
use PiCa::pica_mouse::{Button, Mouse};
use PiCa::pica_window::{Window, WindowAttributes, ALT, CTR, SHIFT};

pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("Awesome PiCa Simulation")
        .with_position(50, 50)
        .with_size(800, 800);

    let mut window = Window::new_with_attributes(window_attributes)?;
    let mut last_print_time: f32 = 0.0;
    while window.pull() {
        match window.mouse {
            Mouse {
                left_button: Button { pressed: true, .. },
                ..
            } => {
                println!("LEFT BUTTON PRESSED")
            }
            Mouse {
                right_button: Button { pressed: true, .. },
                ..
            } => {
                println!("RIGHT BUTTON PRESSED")
            }
            Mouse {
                left_button: Button { released: true, .. },
                ..
            } => {
                println!("LEFT BUTTON RELEASED")
            }
            Mouse {
                right_button: Button { released: true, .. },
                ..
            } => {
                println!("RIGHT BUTTON RELEASED")
            }
            _ => {}
        }
        if window.keys['A' as u8 as usize].pressed {
            println!("TrIggErED!!");
        }
        if window.keys[CTR].pressed {
            println!("Ctrl is pressed!");
        }

        if window.keys[ALT].pressed {
            println!("ALT is pressed!");
        }
        if window.keys[SHIFT].pressed {
            println!("SHIFT is pressed!");
        }
        // Test that we can capture all text input with this kind of API, we do!
        if window.text_length > 0 {
            println!("{:?}", &window.text[..window.text_length]);
        }

        if window.time.seconds - last_print_time > 1.0 {
            println!(
                "Position: {:?}, Size: {:?}, Mouse: {:?}, delta_us: {:?}, ms: {:?}",
                window.window_attributes.position,
                window.window_attributes.size,
                window.mouse.position,
                window.time.delta_microseconds,
                window.time.milliseconds,
            );
            last_print_time = window.time.seconds;
        }

        // Win32 DirectX12 Rendering!
        window.push();
    }
    Ok(())
}
