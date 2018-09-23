extern crate piston;
extern crate piston_window;
extern crate rand;
extern crate rusty_chip;

use piston::input::{Button, Key, PressEvent, ReleaseEvent};
use piston_window::{clear, rectangle, Event, OpenGL, PistonWindow, WindowSettings};
use rusty_chip::cpu;
use std::env;
use std::fs::File;
use std::io::Read;

static SCALE: usize = 8;

fn main() {
    let rom_path = env::args()
        .nth(1)
        .expect("Please specify the path to a ROM as the 1st arg");
    // Specify anything as the 2nd arg to enable debug mode
    let debug_enabled = if let Some(_) = env::args().nth(2) {
        true
    } else {
        false
    };

    let mut allow_next_step = !debug_enabled;

    let rom_bytes = read_rom(&rom_path);

    let mut cpu = cpu::CPU::new();
    cpu.load_rom(&rom_bytes);

    // Might as well free the memory now that it's been copied,
    // otherwise this would be alive until the end of the game
    // Same thing should be possible by just using a local scope { ... }
    std::mem::drop(rom_bytes);

    let mut window_wrapper = WindowWrapper::new();

    while let Some(e) = window_wrapper.window.next() {
        if let Some(b) = e.press_args() {
            if let Button::Keyboard(key) = b {
                if let Key::Return = key {
                    allow_next_step = true;
                }
            }
            WindowWrapper::process_input(&b, &mut cpu.keypad, 1);
        }

        if let Some(b) = e.release_args() {
            WindowWrapper::process_input(&b, &mut cpu.keypad, 0);
        }

        if allow_next_step {
            if debug_enabled {
                cpu.emulate_cycle();
                println!("{}", cpu);
            } else {
                for _ in 0..5 {
                    cpu.emulate_cycle();
                }
            }
        }

        if debug_enabled {
            allow_next_step = false;
        }

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

    fn process_input(b: &Button, keypad: &mut Vec<u8>, new_value: u8) {
        if let &Button::Keyboard(key) = b {
            match key {
                Key::D0 => keypad[0] = new_value,
                Key::D1 => keypad[1] = new_value,
                Key::D2 => keypad[2] = new_value,
                Key::D3 => keypad[3] = new_value,
                Key::Q => keypad[4] = new_value,
                Key::W => keypad[5] = new_value,
                Key::E => keypad[6] = new_value,
                Key::R => keypad[7] = new_value,
                Key::A => keypad[8] = new_value,
                Key::S => keypad[9] = new_value,
                Key::D => keypad[10] = new_value,
                Key::F => keypad[11] = new_value,
                Key::Y => keypad[12] = new_value,
                Key::X => keypad[13] = new_value,
                Key::C => keypad[14] = new_value,
                Key::V => keypad[15] = new_value,
                _ => (),
            }
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
