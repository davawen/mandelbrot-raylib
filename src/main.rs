use std::sync::{Mutex, Arc};
use std::thread;
use itertools::Itertools;
use sdl2::pixels::Color;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::{Canvas, CanvasBuilder, RenderTarget};
use std::time::Duration;
use rayon::prelude::*;

pub struct Complex(f64, f64);

fn mandelbrot(c: Complex, max_iter: u16) -> u16 {
	let mut z = Complex(0.0, 0.0);

	for iteration in 0..max_iter {
		if z.0 >= 2.0 || z.1 >= 2.0 {
            return iteration;
       }

		// (a + bi)^2
		// a^2 + 2abi - b^2
		z = Complex(
			z.0*z.0 - z.1*z.1,
			2.0*z.0*z.1
		);
		z.0 += c.0;
		z.1 += c.1;
	}

    max_iter
}

struct Camera {
    x: f64, y: f64,
    zoom: f64
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut cam = Camera {
        x: -0.5, y: 0.0,
        zoom: 2.5
    };

    let mut values = vec![];

    let mut max_iter = 32;
    println!("change how many iterations there are with the number keys!");

    let mut rerender = true;

    'running: loop {
        let (mouse_x, mouse_y) = (event_pump.mouse_state().x(), event_pump.mouse_state().y());
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                Event::Window { win_event: WindowEvent::SizeChanged(..), .. } => rerender = true,
                Event::KeyDown { keycode: Some(keycode), .. } => 'nan: {
                    use Keycode as K;
                    let power = match keycode {
                        K::Num1 => 1,
                        K::Num2 => 2,
                        K::Num3 => 3,
                        K::Num4 => 4,
                        K::Num5 => 5,
                        K::Num6 => 6,
                        K::Num7 => 7,
                        K::Num8 => 8,
                        K::Num9 => 9,
                        K::Num0 => 10,
                        _ => break 'nan
                    };

                    max_iter = 2_u16.pow(power + 2);
                    println!("{max_iter}");
                    rerender = true;
                }
                Event::MouseWheel { y, .. } => {
                    let (w, h) = canvas.output_size().unwrap();
                    let delta = y as f64 / 10.0;
					cam.x += cam.zoom*delta*(mouse_x as f64 / w as f64 - 0.5);
					cam.y += cam.zoom*delta*(mouse_y as f64 / w as f64 - (h as f64 / w as f64/2.0));
					cam.zoom *= 1.0 - delta;
					rerender = true; 
                }
                _ => {}
            }
        }

        if rerender {
            canvas.clear();

            let (w, h) = canvas.output_size().unwrap();
            let (w, h) = (w as usize, h as usize);
            (0..w*h).into_par_iter().map( |i| {
                let (x, y) = (i % w, i / w);
                let c = Complex(
                    (x as f64 / w as f64 - 0.5) * cam.zoom + cam.x,
                    (y as f64 / w as f64 - (h as f64/w as f64)/2.0) * cam.zoom + cam.y
                );

                mandelbrot(c, max_iter)
            }).collect_into_vec(&mut values);

            for (y, x) in (0..h).cartesian_product(0..w) {
                let i = values[y*w + x] as u32;
                if i == max_iter as u32 {
                    canvas.set_draw_color(Color::BLACK);
                }
                else {
                    canvas.set_draw_color(Color::RGB(255 - (i*255/max_iter as u32) as u8, 128 - (i*128/max_iter as u32) as u8, 128));
                }
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
            canvas.present();
            rerender = false;
        }

        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
