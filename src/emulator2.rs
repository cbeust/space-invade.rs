
const MEMORY_SIZE: usize = 256;

use lazy_static::lazy_static;
use std::sync::RwLock;
use std::sync::Mutex;
use std::time::Duration;
use std::thread;
use emulator::opcodes::STA;
use crate::log_time;
use once_cell::sync::Lazy;

struct State {
    memory: Vec<u8>,
    input_1: u8,
}

static SHARED_STATE: Lazy<RwLock<State>> = Lazy::new(|| {
    RwLock::new(State {
        memory: vec![0; MEMORY_SIZE],
        input_1: 0,
    })
});

trait Emulator {
    fn run_one_frame(&self);
    fn write_memory(&mut self, address: usize, value: u8);
    fn memory(&self) -> Vec<u8>;
    fn set_input_1(&mut self, value: u8);
    fn input_1(&self) -> u8;
}
struct Runner{}

impl Emulator for Runner {
    fn run_one_frame(&self) {
        println!("Running one frame");
        std::thread::sleep(Duration::from_millis(500));
    }

    fn write_memory(&mut self, address: usize, value: u8) {
        SHARED_STATE.write().unwrap().memory[address] = value;
    }

    fn memory(&self) -> Vec<u8> {
        SHARED_STATE.read().unwrap().memory.to_vec()
    }

    fn set_input_1(&mut self, value: u8) {
        SHARED_STATE.write().unwrap().input_1 = value;
    }

    fn input_1(&self) -> u8 {
        SHARED_STATE.read().unwrap().input_1
    }
}

pub(crate) fn main() {
    let mut e2 = Runner{};
    let mut e3 = Runner{};
    let t = thread::spawn(move || {
        let mut i = 0;
        loop {
            e2.run_one_frame();
            println!("Writing {}, input_1: {}", i, e2.input_1());
            e2.write_memory(0, i);
            i += 1;
        }
    });
    loop {
        let value = e3.memory()[0];
        let s = format!("============= Memory: {:02x}", value);
        e3.set_input_1(value);
        log_time(&s);
        thread::sleep(Duration::from_millis(500));
    }
}
