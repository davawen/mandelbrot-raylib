use std::{ffi::CString, f32::consts::PI, usize, mem::MaybeUninit, array};

use raylib::prelude::*;

// to easily switch between 32 and 64 bits
type FP = f32;

#[repr(C)]
pub struct Complex(FP, FP);

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

#[repr(C)]
#[derive(Clone, Copy)]
enum ShaderKind {
    Mandelbrot = 0,
    Julia
}

type Uniform = i32;

struct FractalShader {
    s: Shader,
    resolution: Uniform,
    max_iter: Uniform,
    cam: Uniform,
    animation: Uniform,
    kind: Uniform
}

impl FractalShader {
    fn create(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        const SHADER: &str = include_str!("mandelbrot.glsl");
        let s = rl.load_shader_from_memory(thread, None, Some(SHADER));
        let resolution = s.get_shader_location("resolution");
        let max_iter = s.get_shader_location("max_iter");
        let cam = s.get_shader_location("cam");
        let animation = s.get_shader_location("animation");
        let kind = s.get_shader_location("kind");
        Self {
            s, resolution, max_iter, cam, animation, kind
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

    rl.set_target_fps(60);

    let mut cam = Camera {
        x: -0.5, y: 0.0,
        zoom: 2.5
    };

    let mut rerender = true;

    let mut max_iter = 32;
    println!("change how many iterations there are with the number keys!");

    let mut animation = 0.0_f32;
    let mut animating = false;

    let mut kind = ShaderKind::Mandelbrot;

    let mut shader = FractalShader::create(&mut rl, &thread);
    shader.s.set_shader_value::<[f32; 2]>(shader.resolution, [640.0, 480.0]);
    shader.s.set_shader_value(shader.max_iter, max_iter);
    shader.s.set_shader_value(shader.kind, kind as i32);

    let mut target = rl.load_render_texture(&thread, 640, 480).unwrap();

    rl.gui_enable();

    let number_keys = number_keys();
    let mut old_mouse = rl.get_mouse_position();

    while !rl.window_should_close() {
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        if rl.is_window_resized() {
            shader.s.set_shader_value::<[f32; 2]>(shader.resolution, [w as f32, h as f32]);
            target = rl.load_render_texture(&thread, w as _, h as _).unwrap();
            rerender = true;
        }

        for key in number_keys {
            if rl.is_key_pressed(key.0) {
                max_iter = 2_i32.pow(key.1 + 2);
                shader.s.set_shader_value(shader.max_iter, max_iter);

                println!("{max_iter}");
                rerender = true;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            kind = match kind {
                ShaderKind::Mandelbrot => ShaderKind::Julia,
                ShaderKind::Julia => ShaderKind::Mandelbrot
            };
            shader.s.set_shader_value(shader.kind, kind as i32);
            rerender = true;
        }

        animating ^= rl.is_key_pressed(KeyboardKey::KEY_SPACE);
        if animating {
            animation += 0.02 / 60.0;
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

        if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            let delta = rl.get_mouse_position() - old_mouse;
            if delta != Vector2::zero() {
                cam.x -= cam.zoom*delta.x as FP / w as FP;
                cam.y -= cam.zoom*delta.y as FP / w as FP;
                rerender = true;
            }
        }

        old_mouse = rl.get_mouse_position();

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::new(0, 0, 0, 0));
        if rerender {
            {
                let mut d = d.begin_texture_mode(&thread, &mut target);
                {
                    shader.s.set_shader_value(shader.animation, [animation.cos()*0.7885, animation.sin()*0.7885]);
                    shader.s.set_shader_value(shader.cam, [cam.x, cam.y, cam.zoom]);

                    let mut d = d.begin_shader_mode(&shader.s);
                    d.draw_rectangle(0, 0, w, h, Color::WHITE);
                }
            }
            rerender = false;
        }

        d.draw_texture(&target, 0, 0, Color::WHITE);

        let new_kind = if d.gui_button(Rectangle::new(5.0, 5.0, 65.0, 20.0), Some(CString::new("Mandelbrot").unwrap().as_c_str())) {
            Some(ShaderKind::Mandelbrot)
        } else if d.gui_button(Rectangle::new(75.0, 5.0, 65.0, 20.0), Some(CString::new("Julia").unwrap().as_c_str())) {
            Some(ShaderKind::Julia)
        } else { None };

        if let Some(new_kind) = new_kind {
            kind = new_kind;
            shader.s.set_shader_value(shader.kind, kind as i32);
            rerender = true;
        }

        if let ShaderKind::Julia = kind {
            let new = d.gui_slider_bar(Rectangle::new(5.0, 30.0, 140.0, 20.0), None, None, animation, 0.0, 2.0*PI);
            if new != animation {
                animation = new as FP;
                rerender = true;
            }
        }

        drop(d);
    }

    rl.gui_disable();

}
