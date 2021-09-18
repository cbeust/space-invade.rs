use emulator::emulator_state::EmulatorState;

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{UNIX_EPOCH, SystemTime};

mod sdl2;

lazy_static! {
    pub(crate) static ref STATIC_LISTENER: Mutex<EmulatorState> = Mutex::new(EmulatorState::new());
}

fn main() {
    sdl2::sdl2(&STATIC_LISTENER).unwrap();
}

pub fn log_time(s: &str) {
    println!("{} {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000, s);
}