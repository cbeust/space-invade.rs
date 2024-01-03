use std::time::{UNIX_EPOCH, SystemTime};

mod sdl2;

fn main() -> Result<(), String> {
    sdl2::sdl2()
}

pub fn log_time(s: &str) {
    println!("{} {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000, s);
}