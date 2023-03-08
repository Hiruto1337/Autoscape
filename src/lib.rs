pub mod image {
    extern crate scrap;
    use scrap::Capturer;
    use std::cmp;
    use std::io::ErrorKind::WouldBlock;
    use std::thread;
    use std::time::Duration;

    pub struct Color {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
        pub tolerance: u8,
    }

    pub struct Ranges {
        red_low: u8,
        red_high: u8,
        green_low: u8,
        green_high: u8,
        blue_low: u8,
        blue_high: u8,
    }

    pub fn get_buffer(capturer: &mut Capturer) -> Vec<u8> {
        let one_second = Duration::new(1, 0);
        let one_frame = one_second / 60;

        loop {
            let buffer = match capturer.frame() {
                Ok(buffer) => buffer,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        // Keep spinning.
                        thread::sleep(one_frame);
                        continue;
                    } else {
                        panic!("Error: {}", error);
                    }
                }
            };

            return buffer.to_owned();
        }
    }

    pub fn get_pixels(
        rgb: (u8, u8, u8),
        tolerance: u8,
        capturer: &mut Capturer,
    ) -> Vec<(u32, u32)> {
        let (w, h) = (capturer.width(), capturer.height());

        println!("width: {w}\nheight: {h}");

        let color: Color = Color {
            red: rgb.0,
            green: rgb.1,
            blue: rgb.2,
            tolerance,
        };

        let ranges: Ranges = Ranges {
            red_low: cmp::max(0, (color.red as i16) - (color.tolerance as i16)) as u8,
            red_high: cmp::min(255, (color.red as i16) + (color.tolerance as i16)) as u8,
            green_low: cmp::max(0, (color.green as i16) - (color.tolerance as i16)) as u8,
            green_high: cmp::min(255, (color.green as i16) + (color.tolerance as i16)) as u8,
            blue_low: cmp::max(0, (color.blue as i16) - (color.tolerance as i16)) as u8,
            blue_high: cmp::min(255, (color.blue as i16) + (color.tolerance as i16)) as u8,
        };

        let buffer = get_buffer(capturer);
        let stride = buffer.len() / h;

        let mut matching_pixels: Vec<(u32, u32)> = Vec::new();

        for y in 0..h {
            for x in 0..w {
                let i = stride * y + 4 * x;
                // If a matching pixel is found
                if ranges.red_low < (buffer[i + 2] as u8)
                    && (buffer[i + 2] as u8) < ranges.red_high
                    && ranges.green_low < (buffer[i + 1] as u8)
                    && (buffer[i + 1] as u8) < ranges.green_high
                    && ranges.blue_low < (buffer[i] as u8)
                    && (buffer[i] as u8) < ranges.blue_high
                {
                    matching_pixels.push((x as u32, y as u32));
                }
            }
        }

        return matching_pixels;
    }

    pub fn get_color(pixel: (u32, u32), capturer: &mut Capturer) -> Color {
        let h = capturer.height();

        let buffer = get_buffer(capturer);

        let stride = buffer.len() / h;
        let i = stride * (pixel.1 as usize) + 4 * (pixel.0 as usize);
        return Color {
            red: buffer[i + 2],
            green: buffer[i + 1],
            blue: buffer[i],
            tolerance: 0,
        };
    }
}

pub mod input {
    use dialoguer::{theme::ColorfulTheme, Select};

    pub fn get_input() -> (usize, usize, usize) {
        let system_select = &["🍎 16:10 (MacBook)", "🖥️  16:9 (Windows)"];

        let system = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your aspect-ratio:")
            .default(0)
            .items(&system_select[..])
            .interact()
            .unwrap();

        let target_select = &["🔩 Iron", "🔥 Coal"];
        let target = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your target:")
            .default(0)
            .items(&target_select[..])
            .interact_opt()
            .unwrap()
            .unwrap();
        
        let auto_empty_select = &["💰 No", "🗑️  Yes"];
        let auto_empty = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Auto-empty last slot?")
            .default(0)
            .items(&auto_empty_select[..])
            .interact_opt()
            .unwrap()
            .unwrap();

        (system, target, auto_empty)
    }
}

pub mod movement {
    use autopilot::{
        geometry::Point,
        mouse::{self},
    };

    use std::{thread::sleep, time::Duration};

    pub fn move_smoothly(pixel: (u32, u32), center_pixel: (u32, u32), ratio: f64) -> () {
        let mut x_sign: i8 = 1;
        let mut y_sign: i8 = 1;

        if pixel.0 < center_pixel.0 {
            x_sign = -1;
        }

        if pixel.1 < center_pixel.1 {
            y_sign = -1;
        }

        for offset in 0..6 {
            let point: Point;

            point = Point::from_pixel(
                (pixel.0 as i32 + (x_sign * (offset - 5)) as i32) as f64,
                (pixel.1 as i32 + (y_sign * (offset - 5)) as i32) as f64,
                ratio,
            );

            mouse::move_to(point).expect("Couldn't move to point");
            sleep(Duration::from_millis(10));
        }
    }

    pub fn empty_inventory(center_pixel: (u32, u32), ratio: f64) -> () {
        let x_coor: u32;
        let y_coor: u32;
        // let y_drop_normal: usize;
        let y_drop_bottom: u32;

        if center_pixel == (720, 450) {
            x_coor = 1216 + 3 * 42;
            y_coor = 626 + 6 * 36;
            // y_drop_normal = y_coor + 45;
            y_drop_bottom = y_coor + 21;
        } else {
            x_coor = 1640 + 3 * 53;
            y_coor = 740 + 6 * 46;
            // y_drop_normal = y_coor + 52;
            y_drop_bottom = y_coor + 16;
        }

        move_smoothly((x_coor, y_coor), center_pixel, ratio);
        mouse::click(mouse::Button::Right, Some(0));
        sleep(Duration::from_millis(750));

        move_smoothly((x_coor, y_drop_bottom), center_pixel, ratio);
        mouse::click(mouse::Button::Left, Some(0));
        sleep(Duration::from_millis(500));
    }
}

pub mod sound {
    use rodio::Sink;
    use std::io::BufReader;
    pub fn toggle(run_program: &mut bool) -> () {
        *run_program = !*run_program;
    
        let sound: Sink;
    
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    
        if *run_program {
            let file = std::fs::File::open("sound/activate.wav").unwrap();
            sound = stream_handle.play_once(BufReader::new(file)).unwrap();
        } else {
            let file = std::fs::File::open("sound/deactivate.wav").unwrap();
            sound = stream_handle.play_once(BufReader::new(file)).unwrap();
        }
    
        sound.play();
    
        sound.sleep_until_end();
    
        println!("Running: {run_program}");
    }
}
