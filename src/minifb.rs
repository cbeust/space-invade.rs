use minifb::{Key, Window, WindowOptions};
use emulator::emulator::{HEIGHT, WIDTH};
use emulator::memory::{GRAPHIC_MEMORY_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};

const MAGNIFICATION: usize = 1;

pub fn run_minifb() {
    let shared_state = emulator::emulator::Emulator::start_emulator();

    let width = WIDTH as usize * MAGNIFICATION;
    let height = HEIGHT as usize * MAGNIFICATION;
    let mut buffer: Vec<u32> = vec![0; width * height ];

    let mut window = Window::new(
        "Test - ESC to exit",
        width,
        height,
        WindowOptions::default(),
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    println!("Graphic memory is {SCREEN_WIDTH} x {SCREEN_HEIGHT} = {}", SCREEN_WIDTH * SCREEN_HEIGHT);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut gm_index = 0;
        let mut index = 0;
        let graphic_memory: [u8; GRAPHIC_MEMORY_SIZE] = shared_state.lock().unwrap().graphic_memory();

        let mut i: usize = 0;
        for ix in 0..width {
            for iy in (0..height).step_by(8) {
                let mut byte = graphic_memory[i];
                i += 1;
                for b in 0..8 {
                    let x: i32 = ix as i32 * MAGNIFICATION as i32;
                    let y: i32 = (height as i32 - (iy as i32 + b)) * MAGNIFICATION as i32;
                    let color = if byte & 1 == 0 { 0 } else {
                        if iy > 200 && iy < 220 { 0xff000000 }
                        else if iy < 80 { 0x00ff0000 }
                        else { 0xffffff }
                    };
                    buffer[(y - 1) as usize * width + x as usize] = color;
                    byte >>= 1;
                }
            }
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, width, height)
            .unwrap();
    }

}