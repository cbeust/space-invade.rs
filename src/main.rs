use std::time::{UNIX_EPOCH, SystemTime};
use crate::minifb::run_minifb;

// mod sdl2;
mod minifb;
mod sounds;

fn main() {
    run_minifb();
    // sdl2::sdl2()
}

pub fn log_time(s: &str) {
    println!("{} {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000, s);
}