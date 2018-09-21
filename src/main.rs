extern crate piston_window;

use piston_window::*;

fn main() {
    // let mut cpu = CPU::new();

    // loop {
    //     // emulate a cycle
    //     cpu.emulate_cycle();
    //     // Draw if the flag is set

    //     // Store keypress state
    // }

    let mut window: PistonWindow = PistonWindow::new(
    OpenGL::V3_3,
    0,
    WindowSettings::new("Hello World!", [64, 32])
        .opengl(OpenGL::V3_3)
        .srgb(false)
        .build()
        .unwrap(),
);
    
    
    while let Some(e) = window.next() {
        window.draw_2d(&e, |context, graphics| {
            clear([0.5, 1.0, 0.5, 1.0], graphics);
            rectangle([1.0, 0.0, 0.0, 1.0], // red
                      [0.0, 0.0, 5.0, 5.0],
                      context.transform,
                      graphics);
        });
    }
}
