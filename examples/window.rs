use std::time::{Duration, Instant};

use pica::error::Error;
use pica::pica_mouse::{Button, Mouse};
use pica::pica_window::{Window, WindowAttributes, CTR, ALT, SHIFT, };

pub fn main() -> Result<(), Error> {
    let window_attributes = WindowAttributes::new()
        .with_title("Awesome pica Simulation")
        .with_position(50, 50)
        .with_size(800, 600);

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
                "Position: {:?}, Size: {:?}, Mouse: {:?}, delta_us: {:?}, ms: {:?}, tps: {:?}",
                window.window_attributes.position,
                window.window_attributes.size,
                window.mouse.position,
                window.time.delta_microseconds,
                window.time.milliseconds,
                window.time.ticks_per_second,

            );
            last_print_time = window.time.seconds;
        }
    }
    Ok(())
}
