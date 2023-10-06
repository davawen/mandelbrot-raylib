use raylib::prelude::*;

// to easily switch between 32 and 64 bits
type FP = f32;

#[repr(C)]
pub struct Complex(FP, FP);

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
    x: FP, y: FP,
    zoom: FP
}

fn number_keys() -> [(KeyboardKey, u32); 10] {
    use KeyboardKey as K;
    let nums = [
        K::KEY_ONE, K::KEY_TWO, K::KEY_THREE, K::KEY_FOUR, K::KEY_FIVE,
        K::KEY_SIX, K::KEY_SEVEN, K::KEY_EIGHT, K::KEY_NINE, K::KEY_ZERO
    ];
    std::array::from_fn(|i| (nums[i], i as _))
}

type Uniform = i32;

struct MandelbrotShader {
    s: Shader,
    resolution: Uniform,
    max_iter: Uniform,
    cam: Uniform
}

impl MandelbrotShader {
    fn create(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        const SHADER: &str = include_str!("mandelbrot.glsl");
        let s = rl.load_shader_from_memory(thread, None, Some(SHADER));
        let resolution = s.get_shader_location("resolution");
        let max_iter = s.get_shader_location("max_iter");
        let cam = s.get_shader_location("cam");
        Self {
            s, resolution, max_iter, cam
        }
    }
}

fn main() {

    // let opt = Opt::from_arg();
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Mandelbrot")
        .resizable()
        .build();

    let mut cam = Camera {
        x: -0.5, y: 0.0,
        zoom: 2.5
    };

    let mut rerender = true;

    let mut max_iter = 32;
    println!("change how many iterations there are with the number keys!");

    let mut shader = MandelbrotShader::create(&mut rl, &thread);
    shader.s.set_shader_value::<[f32; 2]>(shader.resolution, [640.0, 480.0]);
    shader.s.set_shader_value(shader.max_iter, max_iter);

    let mut target = rl.load_render_texture(&thread, 640, 480).unwrap();

    let keys = number_keys();
    while !rl.window_should_close() {
        for key in keys {
            if rl.is_key_pressed(key.0) {
                max_iter = 2_i32.pow(key.1 + 2);
                shader.s.set_shader_value(shader.max_iter, max_iter);

                println!("{max_iter}");
                rerender = true;
            }
        }

        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        if rl.is_window_resized() {
            shader.s.set_shader_value::<[f32; 2]>(shader.resolution, [w as f32, h as f32]);
            target = rl.load_render_texture(&thread, w as _, h as _).unwrap();
            rerender = true;
        }

        match rl.get_mouse_wheel_move() {
            y if y != 0.0 => {
                let Vector2 { x: mouse_x, y: mouse_y } = rl.get_mouse_position();

                let delta = y as FP / 10.0;
                cam.x += cam.zoom*delta*(mouse_x as FP / w as FP - 0.5);
                cam.y += cam.zoom*delta*(mouse_y as FP / w as FP - (h as FP / w as FP/2.0));
                cam.zoom *= 1.0 - delta;
                rerender = true; 
            }
            _ => ()
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let delta = rl.get_mouse_delta();
            if delta != Vector2::zero() {
                cam.x -= cam.zoom*delta.x as FP / w as FP;
                cam.y -= cam.zoom*delta.y as FP / w as FP;
                rerender = true;
            }
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::new(0, 0, 0, 0));
        if rerender {
            {
                let mut d = d.begin_texture_mode(&thread, &mut target);
                {
                    shader.s.set_shader_value(shader.cam, [cam.x, cam.y, cam.zoom]);
                    let mut d = d.begin_shader_mode(&shader.s);
                    d.draw_rectangle(0, 0, w, h, Color::WHITE);
                }
            }
            rerender = false;
        }

        d.draw_texture(&target, 0, 0, Color::WHITE);
    }
}
