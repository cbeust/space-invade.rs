use std::time::{Duration, SystemTime};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use emulator::emulator::{Emulator, WIDTH, HEIGHT};

const RECTANGLE_SIZE: u32 = 1;
const WHITE: Color = Color::RGB(255, 255, 255);
const BLACK: Color = Color::RGB(0, 0, 0);
const RED: Color = Color::RGB(255, 0, 0);
const GREEN: Color = Color::RGB(0, 255, 0);

pub fn sdl2() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("", WIDTH as u32 * RECTANGLE_SIZE, HEIGHT as u32 * RECTANGLE_SIZE)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.clear();
    canvas.present();

    //
    // Spawn the game logic in a separate thread. This logic will communicate with the
    // main thread (and therefore, the actual graphics on your screen) via the `SHARED_STATE`
    // object returned by the start of the emilator.
    //
    let shared_state = Emulator::start_emulator();

    canvas.clear();
    canvas.present();

    // Only update the title every one second or so
    let mut last_title_update = SystemTime::now();

    // Main game loop
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {

            //
            // Read the keyboard
            //
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                    => break 'running,
                // Pause / unpause ('p')
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    let mut l = shared_state.lock().unwrap();
                    if l.is_paused() {
                        l.unpause();
                    } else {
                        l.pause();
                    }
                },

                // Tilt
                Event::KeyDown { keycode: Some(Keycode::T), .. } => {
                    shared_state.lock().unwrap().set_bit_in_2(2, true);
                },

                // Insert coin
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(0, true);
                },
                Event::KeyUp { keycode: Some(Keycode::C), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(0, false);
                },
                // Start 2 players
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(1, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Num2), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(1, false);
                },
                // Start 1 player
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(2, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Num1), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(2, false);
                },
                // Player 1 shot
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    if shared_state.lock().unwrap().is_paused() {
                        shared_state.lock().unwrap().unpause();
                    } else {
                        shared_state.lock().unwrap().set_bit_in_1(4, true);
                    }
                },
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(4, false);
                },
                // Player 1 move left
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(5, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(5, false);
                },
                // Player 1 move right
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(6, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
                    shared_state.lock().unwrap().set_bit_in_1(6, false);
                },

                // Player 2 shot ('s')
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    shared_state.lock().unwrap().set_bit_in_2(4, true);
                },
                Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                    shared_state.lock().unwrap().set_bit_in_2(4, false);
                },
                // Player 2 move left ('a')
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    shared_state.lock().unwrap().set_bit_in_2(5, true);
                },
                Event::KeyUp { keycode: Some(Keycode::A), .. } => {
                    shared_state.lock().unwrap().set_bit_in_2(5, false);
                },
                // Player 2 move right ('d')
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    shared_state.lock().unwrap().set_bit_in_2(6, true);
                },
                Event::KeyUp { keycode: Some(Keycode::D), .. } => {
                    shared_state.lock().unwrap().set_bit_in_2(6, false);
                },
                // If the emulator is paused, any key will unpause it
                Event::KeyDown { .. } => {
                    if shared_state.lock().unwrap().is_paused() {
                        shared_state.lock().unwrap().unpause();
                    }
                }
                _ => {
                }
            }
        }

        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        canvas.clear();

        //
        // Draw the graphic
        // Simply map the listener's frame buffer (updated by the main logic in a separate thread)
        // to the SDL canvas
        //
        let graphic_memory = shared_state.lock().unwrap().graphic_memory();
        let mut i: usize = 0;
        for ix in 0..WIDTH {
            for iy in (0..HEIGHT).step_by(8) {
                let mut byte = graphic_memory[i];
                i += 1;
                for b in 0..8 {
                    let x: i32 = ix as i32 * RECTANGLE_SIZE as i32;
                    let y: i32 = (HEIGHT as i32 - (iy as i32+ b)) * RECTANGLE_SIZE as i32;
                    let color = if byte & 1 == 0 { BLACK } else {
                        if iy > 200 && iy < 220 { RED }
                        else if iy < 80 { GREEN }
                        else { WHITE }
                    };
                    byte >>= 1;

                    println!("Index {}: {x},{y}", i - 1);
                    canvas.set_draw_color(color);
                    canvas.fill_rect(Rect::new(x, y, RECTANGLE_SIZE, RECTANGLE_SIZE))
                        .unwrap();
                }
            }
        }

        if last_title_update.elapsed().unwrap().gt(&Duration::from_millis(1000)) {
            let paused = if shared_state.lock().unwrap().is_paused() { " - Paused" } else { "" };
            canvas.window_mut().set_title(
                format!("space-invade.rs - Cédric Beust - {:.2} Mhz{}",
                        shared_state.lock().unwrap().get_megahertz(),
                        paused)
                    .as_str()).unwrap();
            last_title_update = SystemTime::now();
        }

        canvas.present();
    }

    Ok(())
}
