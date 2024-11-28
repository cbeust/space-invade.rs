use minifb::{Key, Window, WindowOptions};
use emulator::emulator::{HEIGHT, WIDTH};
use emulator::memory::{GRAPHIC_MEMORY_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};

const MAGNIFICATION: usize = 1;

const RED: u32 = 0x00ff0000;
const GREEN: u32 = 0x0000ff00;
const WHITE: u32 = 0xffffff;
const BLACK: u32 = 0;

pub fn run_minifb() {
    let shared_state = emulator::emulator::Emulator::start_emulator();

    let width = WIDTH as usize;
    let height = HEIGHT as usize;
    let mut buffer: Vec<u32> = vec![0; width * height * MAGNIFICATION * MAGNIFICATION];

    let mut options = WindowOptions::default();
    options.resize = true;
    let mut window = Window::new(
        "Test - ESC to exit",
        width * 2,
        height * 2,
        options
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    println!("Graphic memory is {SCREEN_WIDTH} x {SCREEN_HEIGHT} = {}", SCREEN_WIDTH * SCREEN_HEIGHT);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let graphic_memory: [u8; GRAPHIC_MEMORY_SIZE] = shared_state.lock().unwrap().graphic_memory();

        let mut i: usize = 0;
        for ix in 0..width {
            for iy in (0..height).step_by(8) {
                let mut byte = graphic_memory[i];
                i += 1;
                for b in 0..8 {
                    let x: i32 = ix as i32 * MAGNIFICATION as i32;
                    let y: i32 = (height as i32 - (iy as i32 + b)) * MAGNIFICATION as i32;
                    let color = if byte & 1 == 0 { BLACK } else {
                        if iy > 200 && iy < 220 { RED }
                        else if iy < 80 { GREEN }
                        else { WHITE }
                    };
                    byte >>= 1;
                    buffer[(y - 1) as usize * width + x as usize] = color;
                }
            }
        }

        // We unwrap here as we want this code to exit if it fails.
        // Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, width, height)
            .unwrap();
    }

}