extern crate scrap;

use device_query::{DeviceQuery, DeviceState, Keycode};

use std::{thread::sleep, time::Duration};

use scrap::{Capturer, Display};

use autoscape::{
    input,
    movement::{move_smoothly, empty_inventory},
    image::{get_color, get_pixels, Color},
    sound::toggle
};

use autopilot::mouse::{Button, click};

struct Settings {
    auto_empty: usize,
    target_pixel: (u8, u8, u8),
    center_pixel: (u32, u32),
    inventory_pixel: (u32, u32),
    ratio: f64
}

impl Settings {
    fn new(screen_dimensions: (&u32, &u32), autopilot_dimensions: (&u32, &u32)) -> Settings {
        let (system, target, auto_empty) = input::get_input();

        Settings {
            auto_empty,
            target_pixel: [(50, 33, 25), (42, 42, 28)][target],
            center_pixel: (screen_dimensions.0 / 2, screen_dimensions.1 / 2),
            inventory_pixel: [(1344, 846), (1797, 1010)][system],
            ratio: (screen_dimensions.0.to_owned() as f64)/(autopilot_dimensions.0.to_owned() as f64)
        }
    }

    fn inv_empty(&self, capturer: &mut Capturer) -> bool {
        let color: Color = get_color(self.inventory_pixel, capturer);

        color.red == 62 && color.green == 53 && color.blue == 41
    }
}

fn main() {
    println!( r"
               _        _____                      
    /\        | |      / ____|                     
   /  \  _   _| |_ ___| (___   ___ __ _ _ __   ___ 
  / /\ \| | | | __/ _ \\___ \ / __/ _` | '_ \ / _ \
 / ____ \ |_| | || (_) |___) | (_| (_| | |_) |  __/
/_/    \_\__,_|\__\___/_____/ \___\__,_| .__/ \___|
                                       | |         
                                       |_|         
    ");

    let display = Display::primary().expect("Couldn't find primary display.");

    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    let screen_dimensions: (&u32, &u32) = (&(capturer.width() as u32), &(capturer.height() as u32));

    let autopilot_dimensions: (&u32, &u32) = (&(autopilot::screen::size().width as u32), &(autopilot::screen::size().height as u32));

    let settings: Settings = Settings::new(screen_dimensions, autopilot_dimensions);

    println!("All set! Hold >Delete< to start running!");

    let device_state = DeviceState::new();

    let mut run_program: bool = false;

    loop {
        // If delete is held, toggle run status
        if device_state.get_keys().contains(&Keycode::Delete) {
            toggle(&mut run_program);
        }

        // If program is running
        if run_program {
            // ... and the inventory is full
            if !settings.inv_empty(&mut capturer) {
                // ... and auto-empty is enabled
                if settings.auto_empty == 1 {
                    // ... empty the last slot
                    empty_inventory(settings.center_pixel, settings.ratio);
                } else {
                    // ... stop running program
                    toggle(&mut run_program);
                    continue;
                }
            }

            // Get all pixels matching pixels
            let pixels = get_pixels(settings.target_pixel, 2, &mut capturer);

            // Find the matching pixel closest to the center of the screen
            let closest_pixel = get_closest_pixel(pixels, settings.center_pixel);

            println!("Closest pixel: {:?}", closest_pixel);
            if closest_pixel.0 != 0 && closest_pixel.1 != 0 {
                move_smoothly(closest_pixel, settings.center_pixel, settings.ratio);
                click(Button::Left, Some(0));
            }
        }
        sleep(Duration::from_secs(3));
    }
}

fn get_closest_pixel(pixels: Vec<(u32, u32)>, center_pixel: (u32, u32)) -> (u32, u32) {
    let mut closest_pixel: (u32, u32) = (0, 0);
    let mut current_dist: f32 = 10000.0;

    for point in pixels.iter() {
        let x_diff = (point.0 as i32 - center_pixel.0 as i32) as f32;
        let y_diff = (point.1 as i32 - center_pixel.1 as i32) as f32;

        let x_diff_sqr = f32::powf(x_diff, 2.0);
        let y_diff_sqr = f32::powf(y_diff, 2.0);

        let dist = f32::sqrt(x_diff_sqr + y_diff_sqr);
        if dist < current_dist {
            closest_pixel = (point.0, point.1);
            current_dist = dist;
        }
    }

    closest_pixel
}
