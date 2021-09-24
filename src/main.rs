use emulator::emulator_state::SharedState;

use std::time::{UNIX_EPOCH, SystemTime};

mod sdl2;

fn main() {
    sdl2::sdl2().unwrap();
}

pub fn log_time(s: &str) {
    println!("{} {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000, s);
}