use cstr::cstr;
use raylib::prelude::*;

// to easily switch between 32 and 64 bits
type FP = f64;

#[repr(C)]
pub struct Complex(FP, FP);

struct Camera {
    x: FP, y: FP,
    zoom: FP
}

impl Camera {
    fn transform(&self, mut pos: Vector2, w: i32, h: i32) -> Complex {
        pos /= Vector2::new(w as f32, h as f32);
        Complex(
            (pos.x - 0.5) as FP * self.zoom + self.x,
            ((pos.y - 0.5) * h as f32 / w as f32) as FP * self.zoom + self.y
        )
    }
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

#[derive(Clone, Copy)]
struct DoubleUniform {
    big: Uniform,
    small: Uniform
}

impl DoubleUniform {
    fn new(s: &Shader, name: &str) -> DoubleUniform {
        DoubleUniform{
            big: s.get_shader_location(&format!("{name}_big")),
            small: s.get_shader_location(&format!("{name}_small"))
        }
    }
}

struct FractalShader {
    s: Shader,
    resolution: Uniform,
    max_iter: Uniform,
    cam: DoubleUniform,
    animation: DoubleUniform,
    kind: Uniform
}

impl FractalShader {
    fn create(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        const SHADER: &str = include_str!("mandelbrot.glsl");
        let s = rl.load_shader_from_memory(thread, None, Some(SHADER));
        let resolution = s.get_shader_location("resolution");
        let max_iter = s.get_shader_location("max_iter");
        let cam = DoubleUniform::new(&s, "cam");
        let animation = DoubleUniform::new(&s, "animation");
        let kind = s.get_shader_location("kind");
        Self {
            s, resolution, max_iter, cam, animation, kind
        }
    }
}

/// You cannot directly pass an f64 to a shader, so you need to split it up into two f32s.
/// Doesn't work well with big values, but functions with small ones.
trait ShaderVDouble {
    fn set_shader_f64(self, s: &mut Shader, uniform: DoubleUniform);
}

macro_rules! impl_ShaderVDouble {
    ($N:literal) => {
        impl ShaderVDouble for [f64; $N] {
            fn set_shader_f64(self, s: &mut Shader, uniform: DoubleUniform) {
                let mut big = [0.0; $N];
                let mut small = [0.0; $N];
                for i in 0..$N {
                    big[i] = self[i] as f32;
                    small[i] = (self[i] - big[i] as f64) as f32;
                }

                s.set_shader_value(uniform.big, big);
                s.set_shader_value(uniform.small, small);
            }
        }
    }
}

impl ShaderVDouble for f64 {
    fn set_shader_f64(self, s: &mut Shader, uniform: DoubleUniform) {
        let big = self as f32;
        let small = (self - big as f64) as f32;
        s.set_shader_value(uniform.big, big);
        s.set_shader_value(uniform.small, small);
    }
}

impl_ShaderVDouble!(2);
impl_ShaderVDouble!(3);
impl_ShaderVDouble!(4);

fn render_fractal(d: &mut RaylibDrawHandle, thread: &RaylibThread, shader: &Shader, target: &mut RenderTexture2D) {
    let (w, h) = (target.width(), target.height());
    let mut d = d.begin_texture_mode(thread, target);
    {
        let mut d = d.begin_shader_mode(shader);
        d.draw_rectangle(0, 0, w, h, Color::WHITE);
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

    let mut animation = Complex(0.0, 0.0);
    let mut vary_julia = false;

    let mut kind = ShaderKind::Mandelbrot;

    let mut shader = FractalShader::create(&mut rl, &thread);
    shader.s.set_shader_value::<[f32; 2]>(shader.resolution, [640.0, 480.0]);
    shader.s.set_shader_value(shader.max_iter, max_iter);

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
            rerender = true;
        }

        vary_julia ^= rl.is_key_pressed(KeyboardKey::KEY_SPACE);

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

        let mut draw_selection_lines = false;
        if vary_julia {
            if rl.is_mouse_button_down(MouseButton::MOUSE_RIGHT_BUTTON) {
                let pos = rl.get_mouse_position();
                animation = cam.transform(pos, w, h);
                kind = ShaderKind::Julia;
                rerender = true;
            }
        } else {
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON) {
                kind = ShaderKind::Mandelbrot;
                rerender = true;
            }

            draw_selection_lines = rl.is_mouse_button_down(MouseButton::MOUSE_RIGHT_BUTTON);

            if rl.is_mouse_button_released(MouseButton::MOUSE_RIGHT_BUTTON) {
                let pos = rl.get_mouse_position();
                animation = cam.transform(pos, w, h);
                kind = ShaderKind::Julia;
                rerender = true;
            }
        }

        old_mouse = rl.get_mouse_position();

        let save_image = rl.is_key_pressed(KeyboardKey::KEY_S).then(|| {
            let (w, h) = (4960, 3508);
            rl.load_render_texture(&thread, w, h).unwrap()
        });

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::new(0, 0, 0, 0));
        if rerender {
            [animation.0, animation.1]
                .set_shader_f64(&mut shader.s, shader.animation);
            [cam.x, cam.y, cam.zoom]
                .set_shader_f64(&mut shader.s, shader.cam);
            shader.s.set_shader_value(shader.kind, kind as i32);

            render_fractal(&mut d, &thread, &shader.s, &mut target);
            
            rerender = false;
        }

        // save image
        if let Some(mut target) = save_image {
            shader.s.set_shader_value(shader.resolution, [target.width() as f32, target.height() as f32]);
            render_fractal(&mut d, &thread, &shader.s, &mut target);
            shader.s.set_shader_value(shader.resolution, [w as f32, h as f32]);
            let image = target.get_texture_data().unwrap();
            image.export_image("output.png");
        }

        d.draw_texture(&target, 0, 0, Color::WHITE);

        if draw_selection_lines {
            let pos = d.get_mouse_position();
            d.draw_line(pos.x as i32, 0, pos.x as i32, h, Color::RED);
            d.draw_line(0, pos.y as i32, w, pos.y as i32, Color::RED);
        }

        let new_kind = if d.gui_button(Rectangle::new(5.0, 5.0, 65.0, 20.0), Some(cstr!("Mandelbrot"))) {
            Some(ShaderKind::Mandelbrot)
        } else if d.gui_button(Rectangle::new(75.0, 5.0, 65.0, 20.0), Some(cstr!("Julia"))) {
            Some(ShaderKind::Julia)
        } else { None };

        if let Some(new_kind) = new_kind {
            kind = new_kind;
            rerender = true;
        }

        d.gui_panel(Rectangle::new(5.0, 30.0, 50.0, 20.0));
        vary_julia = d.gui_check_box(Rectangle::new(7.5, 32.5, 15.0, 15.0), Some(cstr!("Vary")), vary_julia);
    }

    rl.gui_disable();

}
