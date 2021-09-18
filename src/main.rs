use emulator::memory::Memory;
use emulator::emulator::Emulator;
use emulator::emulator_state::EmulatorState;

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{UNIX_EPOCH, SystemTime};

mod sdl2;

lazy_static! {
    pub(crate) static ref STATIC_LISTENER: Mutex<EmulatorState> = Mutex::new(EmulatorState::new());
}

fn main() {
    // let thread = thread::spawn(|| {
    //     run_game();
    // });
    // thread.join();
    // run_game();
    // window::piston(&STATIC_LISTENER);
    sdl2::sdl2(&STATIC_LISTENER).unwrap();
    // threads::threads();
}

pub fn log_time(s: &str) {
    println!("{} {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000, s);
}