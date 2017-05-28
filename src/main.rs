extern crate image;
extern crate imageproc;
extern crate rand;
extern crate time;

use rand::{Rng, ThreadRng};
use std::fs::File;
use std::fmt;
use std::path::Path;
use image::{ImageBuffer, Rgb, ImageRgb8, PNG, RgbImage};
use imageproc::drawing::draw_filled_circle_mut;
use imageproc::pixelops::interpolate;
use std::env;

struct Mountain {
    points: Vec<u32>,
}

impl Mountain {
    fn new(y_amp: (f64, f64), resolution: (u32, u32)) -> Mountain {
        let mut rng = rand::thread_rng();
        let step_max = rng.gen_range(0.9, 1.1);
        let step_change = rng.gen_range(0.15, 0.35);
        let (height_min, height_max) = y_amp;
        let (screen_x, screen_y) = resolution;        
        let mut height = rng.gen_range(0.0, height_max);
        let mut slope = rng.gen_range(0.0, step_max) * 2.0 - step_max;
        let mut points: Vec<u32> = Vec::new();

        for _ in 0..screen_x {
            height = height + slope;
            slope = slope + (rng.gen_range(0.0, step_change) * 2.0 - step_change);

            if slope > step_max {
                slope = step_max;
            } else if slope < -step_max {
                slope = -step_max;
            }

            if height > height_max {
                height = height_max;
                slope = slope * -1.0;
            } else if height < height_min {
                height = height_min;
                slope = slope * -1.0;
            }
            points.push(height as u32);
        }
        Mountain {
            points: points
        }
    }
    fn draw(&self, img: &mut RgbImage, color: Rgb<u8>, c_fog: Rgb<u8>, resolution: (u32, u32)) {
        let mut i = 0;
        let (screen_x, screen_y) = resolution;
        for &point in self.points.iter() {
            img.put_pixel(i, point, color);
            let mut k = 0;
            for j in point..screen_y {
                if k < 5 {
                    img.put_pixel(i, j, Rgb([0,0,0]));
                } else {
                    img.put_pixel(i, j, interpolate(c_fog, color, j as f32 / screen_y as f32));
                }
                k = k + 1;
            }
            i = i + 1;
        }
    }
}

fn rgb_rand(rng: &mut ThreadRng, r: (u8, u8), g: (u8, u8), b: (u8, u8)) -> Rgb<u8> {
    Rgb([rng.gen_range(r.0, r.1), rng.gen_range(g.0, g.1), rng.gen_range(b.0, b.1)])
}

fn main() {
    let mut rng = rand::thread_rng();
    let args: Vec<_> = env::args().collect();
    let mut screen_x = 4000;
    let mut screen_y = 3000;
    if args.len() > 2 {
        screen_x = args[1].parse::<u32>().unwrap();
        screen_y = args[2].parse::<u32>().unwrap();
    }
    
    let resolution = (screen_x, screen_y);
    let c_sky = match rng.gen_range(1, 4) {
        1 => rgb_rand(&mut rng, (1, 40), (1, 40), (1, 40)),
        2 => rgb_rand(&mut rng, (215, 225), (215, 225), (230, 255)),
        _ => rgb_rand(&mut rng, (200, 255), (200, 255), (200, 255)),
    };
    let c_fog = rgb_rand(&mut rng, (1, 255), (1, 255), (1, 255));
    let mut img = ImageBuffer::from_pixel(screen_x, screen_y, c_sky);

    if rng.gen_weighted_bool(1) {
        let x = rng.gen_range(screen_x / 10, screen_x * 9 / 10) as i32;
        let y = rng.gen_range(screen_y / 10, screen_y / 3) as i32;
        let rad = rng.gen_range(screen_y / 6, screen_y / 3) as i32;
        let c_planet = interpolate(rgb_rand(&mut rng, (1, 255), (1, 255), (1, 255)), c_sky, 0.5);
        draw_filled_circle_mut(&mut img, (x, y), rad, Rgb([0,0,0]));
        draw_filled_circle_mut(&mut img, (x, y), rad - 5, c_planet);
    }

    for (_, y, pixel) in img.enumerate_pixels_mut() {
        let divisor = screen_y * 2;
        *pixel = interpolate(c_fog, *pixel, y as f32 / divisor as f32);
    }
    
    let mountain_count: u32 = rng.gen_range(4, 7);
    let c_mountain = rgb_rand(&mut rng, (1, 255), (1, 255), (1, 255));
    for i in 0..mountain_count {
        let c = interpolate(c_mountain, c_sky, (i + 1) as f32 / mountain_count as f32);
        let min_y = screen_y * 5 / 6;
        let y_amp = ( (min_y - screen_y / 2 / mountain_count * (mountain_count - i)) as f64, min_y as f64);
        Mountain::new(y_amp, resolution).draw(&mut img, c, c_fog,resolution);
    }

    let _ = ImageRgb8(img).save(&mut File::create(&Path::new("images/export.png")).unwrap(), PNG);
}