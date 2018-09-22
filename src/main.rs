extern crate rusty_chip;
extern crate piston;
extern crate piston_window;
extern crate rand;

use rusty_chip::cpu;
use piston::input::{Button, Key, PressEvent};
use piston_window::{clear, rectangle, Event, OpenGL, PistonWindow, WindowSettings};
use std::env;
use std::fs::File;
use std::io::Read;

static SCALE: usize = 8;

fn main() {
    let mut allow_next_step = false;
    let rom_path = env::args().nth(1).unwrap();
    let rom_bytes = read_rom(&rom_path);

    let mut cpu = cpu::CPU::new();
    cpu.load_rom(&rom_bytes);

    let mut window_wrapper = WindowWrapper::new();

    while let Some(e) = window_wrapper.window.next() {
        if let Some(b) = e.press_args() {
            if let Some(key) = WindowWrapper::map_button(&b) {
                if key == 0 {
                    println!("{}", cpu);
                    allow_next_step = true;
                }
            }
        }

        if allow_next_step {
            cpu.emulate_cycle();
        }

        allow_next_step = false;

        window_wrapper.render(&e, &cpu.gfx);
    }
}

pub struct WindowWrapper {
    window: PistonWindow,
}

impl WindowWrapper {
    fn new() -> WindowWrapper {
        WindowWrapper {
            window: PistonWindow::new(
                OpenGL::V3_3,
                0,
                WindowSettings::new("RustyChip", [(64 * SCALE) as u32, (32 * SCALE) as u32])
                    .opengl(OpenGL::V3_3)
                    .srgb(false)
                    .build()
                    .unwrap(),
            ),
        }
    }

    fn map_button(b: &Button) -> Option<u8> {
        if let &Button::Keyboard(key) = b {
            match key {
                Key::Right => Some(0),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn render(&mut self, e: &Event, pixel_buffer: &Vec<u8>) {
        self.window.draw_2d(e, |context, graphics| {
            clear([0.5, 1.0, 0.5, 1.0], graphics);

            for y in 0..32 {
                for x in 0..64 {
                    let index = (y * 64 + x) as usize;

                    let color = pixel_buffer[index];

                    rectangle(
                        [color as f32, color as f32, color as f32, 1.0],
                        [
                            (x * SCALE) as f64,
                            (y * SCALE) as f64,
                            SCALE as f64,
                            SCALE as f64,
                        ],
                        context.transform,
                        graphics,
                    );
                }
            }
        });
    }
}

fn read_rom(path: &str) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut file_buf = Vec::new();
    let bytes_read = file.read_to_end(&mut file_buf).unwrap();
    println!("Read ROM with {} bytes", bytes_read);
    file_buf
}
