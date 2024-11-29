use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

use minifb::{Key, KeyRepeat, Window, WindowOptions};

use emulator::emulator::{HEIGHT, WIDTH};
use emulator::memory::GRAPHIC_MEMORY_SIZE;

use crate::sounds::{Message, Sound, SoundType};

const MAGNIFICATION: usize = 1;

const RED: u32 = 0x00ff0000;
const GREEN: u32 = 0x0000ff00;
const WHITE: u32 = 0xffffff;
const BLACK: u32 = 0;

struct Mapping {
    input_channel: u8,
    bit: u8,
}

impl Mapping {
    fn new(input_channel: u8, bit: u8) -> Self { Self { input_channel, bit }}
}

pub fn run_minifb() {
    let key_mappings = {
        let mut m: HashMap <Key, Mapping > = HashMap::new();
        m.insert(Key::C, Mapping::new(1, 0)); // Insert coin
        m.insert(Key::Key2, Mapping::new(1, 1)); // 2 players
        m.insert(Key::Key1, Mapping::new(1, 2)); // 1 player
        m.insert(Key::Space, Mapping::new(1, 4)); // Player 1 shoots
        m.insert(Key::Left, Mapping::new(1, 5)); // Player 1 moves left
        m.insert(Key::Right, Mapping::new(1, 6)); // Player 1 moves right
        m.insert(Key::S, Mapping::new(2, 4)); // Player 2 shoots
        m.insert(Key::A, Mapping::new(2, 5)); // Player 2 moves left
        m.insert(Key::D, Mapping::new(2, 6)); // Player 2 moves right
        m
    };

    println!("Press 'c', '1' and then play with left and right arrows, and 'space' to shoot. Enjoy!");

    let (sender, receiver): (Sender<Message>, Receiver<Message>)  = mpsc::channel();
    let sound = Sound::new(receiver);
    let mut sounds: HashSet<SoundType> = HashSet::new();
    thread::spawn(move || {
        sound.run();
    });

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
        let update_state = |key: Key, bit: bool| {
            if let Some(mapping) = key_mappings.get(&key) {
                let state = &mut shared_state.lock().unwrap();
                if mapping.input_channel == 1 {
                    state.set_bit_in_1(mapping.bit, bit);
                } else {
                    state.set_bit_in_2(mapping.bit, bit);
                }
                true
            } else {
                false
            }
        };

        //
        // Check for key events
        //

        // Released keys
        let keys_released: Vec<Key> = window.get_keys_released();
        for key in keys_released {
            update_state(key, false);
        }

        // Pressed keys
        let keys_pressed: Vec<Key> = window.get_keys_pressed(KeyRepeat::No);
        for key in keys_pressed {
            if ! update_state(key, true) {
                match key {
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

        window.update_with_buffer(&buffer, width, height)
            .unwrap();

        //
        // Process sounds
        //
        let mut maybe_send = |value: u8, bit: u8, sound_type: SoundType| {
            let sound2 = sound_type.clone();
            let is_playing = sounds.contains(&sound2);
            let on = (value & (1 << bit)) != 0;
            if on {
                sounds.insert(sound2);
            } else {
                sounds.remove(&sound2);
            }
            if (on && ! is_playing) || (!on && is_playing) {
                match sender.send(Message { sound_type, on }) {
                    Ok(_) => { }
                    Err(e) => { println!("Err: {e}") }
                }
            }
        };

        maybe_send(shared_state.lock().unwrap().get_out_3(), 0, SoundType::Ufo);
        maybe_send(shared_state.lock().unwrap().get_out_3(), 1, SoundType::Fire);
        maybe_send(shared_state.lock().unwrap().get_out_3(), 2, SoundType::PlayerDies);
        maybe_send(shared_state.lock().unwrap().get_out_3(), 3, SoundType::InvaderDies);

        maybe_send(shared_state.lock().unwrap().get_out_5(), 0, SoundType::Invader1);
        maybe_send(shared_state.lock().unwrap().get_out_5(), 1, SoundType::Invader2);
        maybe_send(shared_state.lock().unwrap().get_out_5(), 2, SoundType::Invader3);
        maybe_send(shared_state.lock().unwrap().get_out_5(), 3, SoundType::Invader4);
        maybe_send(shared_state.lock().unwrap().get_out_5(), 4, SoundType::UfoHit);


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