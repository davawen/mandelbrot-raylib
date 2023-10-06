use itertools::Itertools;
use rayon::prelude::*;
use raylib::prelude::*;

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

fn number_keys() -> [(KeyboardKey, u32); 10] {
    use KeyboardKey as K;
    let nums = [
        K::KEY_ONE, K::KEY_TWO, K::KEY_THREE, K::KEY_FOUR, K::KEY_FIVE,
        K::KEY_SIX, K::KEY_SEVEN, K::KEY_EIGHT, K::KEY_NINE, K::KEY_ZERO
    ];
    std::array::from_fn(|i| (nums[i], i as _))
}

fn main() {

    // let opt = Opt::from_arg();
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Mandelbrot")
        .build();

    let mut cam = Camera {
        x: -0.5, y: 0.0,
        zoom: 2.5
    };

    let mut rerender = true;

    let mut values = vec![];
    let mut max_iter = 32;
    println!("change how many iterations there are with the number keys!");

    let mut target = rl.load_render_texture(&thread, 640, 480).unwrap();

    let keys = number_keys();
    while !rl.window_should_close() {
        for key in keys {
            if rl.is_key_pressed(key.0) {
                max_iter = 2_u16.pow(key.1 + 2);
                println!("{max_iter}");
                rerender = true;
            }
        }

        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        if rl.is_window_resized() {
            target = rl.load_render_texture(&thread, w as _, h as _).unwrap();
            rerender = true;
        }

        match rl.get_mouse_wheel_move() {
            y if y != 0.0 => {
                let Vector2 { x: mouse_x, y: mouse_y } = rl.get_mouse_position();
                println!("{mouse_x}, {mouse_y} | {w}, {h}");

                let delta = y as f64 / 10.0;
                cam.x += cam.zoom*delta*(mouse_x as f64 / w as f64 - 0.5);
                cam.y += cam.zoom*delta*(mouse_y as f64 / w as f64 - (h as f64 / w as f64/2.0));
                cam.zoom *= 1.0 - delta;
                rerender = true; 
            }
            _ => ()
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let delta = rl.get_mouse_delta();
            if delta != Vector2::zero() {
                cam.x -= cam.zoom*delta.x as f64 / w as f64;
                cam.y -= cam.zoom*delta.y as f64 / h as f64;
                rerender = true;
            }
        }


        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(0, 0, 0, 0));
        if rerender {
            let (w, h) = (w as usize, h as usize);
            (0..w*h).into_par_iter().map( |i| {
                let (x, y) = (i % w, i / w);
                let c = Complex(
                    (x as f64 / w as f64 - 0.5) * cam.zoom + cam.x,
                    (y as f64 / w as f64 - (h as f64/w as f64)/2.0) * cam.zoom - cam.y
                );

                mandelbrot(c, max_iter)
            }).collect_into_vec(&mut values);

            {
                let mut d = d.begin_texture_mode(&thread, &mut target);
                for (y, x) in (0..h).cartesian_product(0..w) {
                    let i = values[y*w + x] as u32;
                    let color = if i == max_iter as u32 {
                        Color::BLACK
                    }
                    else {
                        Color::new(
                            255 - (i*255/max_iter as u32) as u8,
                            128 - (i*128/max_iter as u32) as u8,
                            128,
                            255
                        )
                    };

                    d.draw_pixel(x as _, y as _, color);
                }
            }
            rerender = false;
        }

        d.draw_texture(&target, 0, 0, Color::WHITE);
    }
}
