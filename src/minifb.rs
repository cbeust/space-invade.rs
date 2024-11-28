use std::time::{Duration, SystemTime};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
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
        width * 3,
        height * 3,
        options
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Only update the title every one second or so
    let mut last_title_update = SystemTime::now();

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {

        //
        // Check for key events
        //
        let keys_released: Vec<Key> = window.get_keys_released();

        for key in keys_released {
            match key {
                Key::Right => {
                    // Player 1 moved right
                    shared_state.lock().unwrap().set_bit_in_1(6, false);
                }
                Key::Left => {
                    // Player 1 moved left
                    shared_state.lock().unwrap().set_bit_in_1(5, false);
                }
                Key::Space => {
                    // Player 1 shot
                    shared_state.lock().unwrap().set_bit_in_1(4, false);
                }
                Key::A => {
                    // Player 2 moved left
                    shared_state.lock().unwrap().set_bit_in_2(5, false);
                }
                Key::D => {
                    // Player 2 moved right
                    shared_state.lock().unwrap().set_bit_in_2(6, false);
                }
                Key::S => {
                    // Player 2 shot
                    shared_state.lock().unwrap().set_bit_in_2(4, false);
                }
                Key::NumPad1 => {
                    // Start 1 player
                    shared_state.lock().unwrap().set_bit_in_1(2, false);
                }
                Key::NumPad2 => {
                    // Start 2 players
                    shared_state.lock().unwrap().set_bit_in_1(1, false);
                }
                Key::C => {
                    // Insert coin
                    shared_state.lock().unwrap().set_bit_in_1(0, false);
                }
                _ => {}
            }
        }

        let keys_pressed: Vec<Key> = window.get_keys_pressed(KeyRepeat::No);
        for key in keys_pressed {
            match key {
                Key::Right => {
                    // Player 1 moved right
                    shared_state.lock().unwrap().set_bit_in_1(6, true);
                }
                Key::Left => {
                    // Player 1 moved left
                    shared_state.lock().unwrap().set_bit_in_1(5, true);
                }
                Key::Space => {
                    // Player 1 shot or unpause
                    if shared_state.lock().unwrap().is_paused() {
                        shared_state.lock().unwrap().unpause();
                    } else {
                        shared_state.lock().unwrap().set_bit_in_1(4, true);
                    }
                }
                Key::A => {
                    // Player 2 moved left
                    shared_state.lock().unwrap().set_bit_in_2(5, true);
                }
                Key::D => {
                    // Player 2 moved right
                    shared_state.lock().unwrap().set_bit_in_2(6, true);
                }
                Key::S => {
                    // Player 2 shot
                    shared_state.lock().unwrap().set_bit_in_2(4, true);
                }
                Key::NumPad1 => {
                    // Start 1 player
                    shared_state.lock().unwrap().set_bit_in_1(2, true);
                }
                Key::NumPad2 => {
                    // Start 2 players
                    shared_state.lock().unwrap().set_bit_in_1(1, true);
                }
                Key::C => {
                    // Insert coin
                    shared_state.lock().unwrap().set_bit_in_1(0, true);
                }
                Key::P => {
                    // Pause
                    let mut l = shared_state.lock().unwrap();
                    if l.is_paused() {
                        l.unpause();
                    } else {
                        l.pause();
                    }
                }
                Key::T => {
                    // Tilt
                    shared_state.lock().unwrap().set_bit_in_2(2, true);
                }
                _ => {
                    // If the emulator is paused, any key will unpause it
                    if shared_state.lock().unwrap().is_paused() {
                        shared_state.lock().unwrap().unpause();
                    }
                }
            }
        }

        //
        // Update the graphics
        //
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

        if last_title_update.elapsed().unwrap().gt(&Duration::from_millis(1000)) {
            let paused = if shared_state.lock().unwrap().is_paused() { " - Paused" } else { "" };
            window.set_title(
                format!("space-invade.rs - Cédric Beust - {:.2} Mhz{}",
                    shared_state.lock().unwrap().get_megahertz(),
                    paused)
                    .as_str());
            last_title_update = SystemTime::now();
        }
    }

}