use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;

use emulator::emulator_state::EmulatorState;

mod sdl2;
mod emulator2;

lazy_static! {
    pub(crate) static ref STATIC_LISTENER: Mutex<EmulatorState> = Mutex::new(EmulatorState::new());
}

fn main() {
    sdl2::sdl2().unwrap();
    // emulator2::main();
}

pub fn log_time(s: &str) {
    println!("{} {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000, s);
}
