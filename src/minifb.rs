use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

use minifb::{Key, KeyRepeat, Window, WindowOptions};

use emulator::emulator::{HEIGHT, WIDTH};
use emulator::memory::GRAPHIC_MEMORY_SIZE;

use crate::sounds::{ALL_SOUNDS, Message, Sound, SoundType};

pub fn run_minifb() {
    let key_mappings = {
        let mut m: HashMap <Key, ChannelBit> = HashMap::new();
        m.insert(Key::C, ChannelBit::new(1, 0)); // Insert coin
        m.insert(Key::Key2, ChannelBit::new(1, 1)); // 2 players
        m.insert(Key::Key1, ChannelBit::new(1, 2)); // 1 player
        m.insert(Key::Space, ChannelBit::new(1, 4)); // Player 1 shoots
        m.insert(Key::Left, ChannelBit::new(1, 5)); // Player 1 moves left
        m.insert(Key::Right, ChannelBit::new(1, 6)); // Player 1 moves right
        m.insert(Key::S, ChannelBit::new(2, 4)); // Player 2 shoots
        m.insert(Key::A, ChannelBit::new(2, 5)); // Player 2 moves left
        m.insert(Key::D, ChannelBit::new(2, 6)); // Player 2 moves right
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
    let mut window = Window::new("space-invade.rs", width * 3, height * 3, options)
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
                if mapping.channel == 1 {
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
        let mut update_sound = |value: u8, bit: u8, sound_type: SoundType| {
            let sound2 = sound_type.clone();
            let is_playing = sounds.contains(&sound2);
            let on = (value & (1 << bit)) != 0;
            // Update our internal table to keep track of which sounds are currently playing
            if on {
                sounds.insert(sound2);
            } else {
                sounds.remove(&sound2);
            }
            // If the status of that sound changed, let the sound thread know
            if on ^ is_playing {
                match sender.send(Message { sound_type, on }) {
                    Ok(_) => { }
                    Err(e) => { println!("Err: {e}") }
                }
            }
        };

        {
            let state = shared_state.lock().unwrap();
            for sd in ALL_SOUNDS.iter() {
                update_sound(state.get_out(sd.channel_bit.channel), sd.channel_bit.bit, sd.sound_type)
            }
        }

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

const MAGNIFICATION: usize = 1;

const RED: u32 = 0x00ff0000;
const GREEN: u32 = 0x0000ff00;
const WHITE: u32 = 0xffffff;
const BLACK: u32 = 0;

pub struct ChannelBit {
    pub channel: u8,
    pub bit: u8,
}

impl ChannelBit {
    fn new(channel: u8, bit: u8) -> Self { Self { channel, bit }}
}
