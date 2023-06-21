use crate::keyboard::char_to_index;
use crate::platform::Platform;
use crate::platform::PlatformContext;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::collections::HashMap;
use std::time::Duration;

lazy_static! {
    pub(crate) static ref VALID_KEYS_TO_CHAR: HashMap<Keycode, char> = HashMap::from([
        (Keycode::Num0, '0'),
        (Keycode::Num1, '1'),
        (Keycode::Num2, '2'),
        (Keycode::Num3, '3'),
        (Keycode::Num4, '4'),
        (Keycode::Num5, '5'),
        (Keycode::Num6, '6'),
        (Keycode::Num7, '7'),
        (Keycode::Num8, '8'),
        (Keycode::Num9, '9'),
        (Keycode::A, 'A'),
        (Keycode::B, 'B'),
        (Keycode::C, 'C'),
        (Keycode::D, 'D'),
        (Keycode::E, 'E'),
        (Keycode::F, 'F'),
    ]);
}

pub(crate) struct SdlPlatform {
    running: bool,

    canvas: Canvas<Window>,
    event_pump: EventPump,

    keyboard_state: [u8; 16],
    width: u32,
    height: u32,
    pixels: Vec<Rect>,
}

impl Default for SdlPlatform {
    fn default() -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let w = 800;
        let h = 600;

        let window = video_subsystem
            .window("chip8", w, h)
            .position_centered()
            .build()
            .expect("Could not initialize SDL Video Subsystem");

        let canvas = window
            .into_canvas()
            .build()
            .expect("Could not make a canvas.");

        let event_pump = sdl.event_pump().unwrap();

        let mut pixels = Vec::<Rect>::new();
        for _ in 0..2048 {
            pixels.push(Rect::new(0, 0, 0, 0));
        }

        Self {
            running: false,
            canvas: canvas,
            event_pump: event_pump,
            keyboard_state: [0u8; 16],
            width: w,
            height: h,
            pixels: pixels,
        }
    }
}

impl Platform for SdlPlatform {
    fn start(&mut self, context: &PlatformContext) {
        self.running = true;

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.present();

        // platform loop
        while self.running {
            Platform::update(self, context);
            Platform::render(self, context);
            ::std::thread::sleep(Duration::new(0, 1_000_000u32 / 30));
        }
    }

    fn update(&mut self, context: &PlatformContext) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyDown { keycode, .. } => {
                    let key = keycode.unwrap();
                    if VALID_KEYS_TO_CHAR.contains_key(&key) {
                        let key_char = VALID_KEYS_TO_CHAR.get(&key).unwrap();

                        // update the platform keyboard state
                        self.keyboard_state[char_to_index(*key_char)] = 1;
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    let key = keycode.unwrap();
                    if VALID_KEYS_TO_CHAR.contains_key(&key) {
                        let keychar = VALID_KEYS_TO_CHAR.get(&key).unwrap();
                        let k = char_to_index(*keychar);
                        if self.keyboard_state[k] == 1 {
                            // previous state was 1 and now it's going to be zero
                            match context.single_key.try_send(*keychar) {
                                Ok(_) => (),
                                Err(_) => (),
                            }
                        }
                        self.keyboard_state[k] = 0;
                    }
                }
                Event::Quit { .. } => {
                    self.running = false;
                }
                _ => {}
            }
        }

        // send the platform keyboard state to the emulator
        match context.keyboard.try_send(self.keyboard_state) {
            Ok(_) => (),
            Err(_) => (),
        }
    }

    fn render(&mut self, context: &PlatformContext) {
        let sound_option = context.sound.try_recv();
        if sound_option.is_ok() {
            let sound = sound_option.unwrap();
            if sound {
                //let _audio = self.sdl_context.audio().unwrap();
                // play sound
            }
        }

        let pixels_option = context.display.try_recv();
        if pixels_option.is_ok() {
            let pixels = pixels_option.unwrap();
            for y in 0..32 {
                for x in 0..64 {
                    let idx = y * 64 + x;
                    if pixels[x][y] > 0 {
                        self.pixels[idx] = Rect::new(
                            (x as f32 * (self.width as f32 / 64.0)).round() as i32,
                            (y as f32 * (self.height as f32 / 32.0)).round() as i32,
                            (self.width as f32 / 64.0).round() as u32,
                            (self.height as f32 / 32.0).round() as u32,
                        );
                    } else {
                        self.pixels[idx] = Rect::new(0, 0, 0, 0);
                    }
                }
            }
        }

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        match self.canvas.fill_rects(&self.pixels) {
            Ok(_) => (),
            Err(_) => (),
        }

        // draw black line grid
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        for x in 0..64 {
            let xx = x as f32 * ((self.width as f32) / 64.0);
            match self.canvas.draw_line(
                Point::new(xx.round() as i32, 0),
                Point::new(xx.round() as i32, self.height as i32),
            ) {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        for y in 0..32 {
            let yy = y as f32 * ((self.height as f32) / 32.0);
            match self.canvas.draw_line(
                Point::new(0, yy.round() as i32),
                Point::new(self.width as i32, yy.round() as i32),
            ) {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        self.canvas.present();
    }
}
