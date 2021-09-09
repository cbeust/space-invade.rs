use crate::memory::Memory;
use crate::emulator::Emulator;
use crate::emulator_state::EmulatorState;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{UNIX_EPOCH, SystemTime};

mod opcodes;
mod emulator;
mod memory;
mod state;
mod emulator_state;
mod sdl2;
mod listener;
mod test;

const VERBOSE: bool = false;
static mut VERBOSE_DISASSEMBLE: bool = false;
const VERBOSE_GRAPHIC: bool = true;
const VERBOSE_DISASSEMBLE_SECTION: bool = false;
const DISASSEMBLE_SECTION_START: usize = 0x1439;
const DISASSEMBLE_SECTION_END: usize = 0x1447;
// const DISASSEMBLE_SECTION_START: usize = 0x1439;
// const DISASSEMBLE_SECTION_END: usize = 0x1447;
const VERBOSE_MEMORY: bool = false;

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