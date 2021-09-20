
const MEMORY_SIZE: usize = 256;

use lazy_static::lazy_static;
use std::sync::RwLock;
use std::sync::Mutex;
use std::time::Duration;
use std::thread;
use emulator::opcodes::STA;
use crate::log_time;
use once_cell::sync::Lazy;

static STATIC_MEMORY: Lazy<RwLock<Vec<u8>>> = Lazy::new(|| {
    let m: Vec<u8> = vec![0; MEMORY_SIZE];
    RwLock::new(m)
});

struct EmulatorState<'a> {
    memory: &'a Lazy<RwLock<Vec<u8>>>,
}

impl Clone for EmulatorState<'_> {
    fn clone(&self) -> Self {
        EmulatorState { memory: &STATIC_MEMORY }
    }
}

trait Emulator2 {
    fn graphic_memory(&self) -> Vec<u8>;
    fn write_memory(&mut self, address: usize, value: u8);
    fn run_one_frame(&self);
}

impl EmulatorState<'_> {
    fn new() -> Self {
        EmulatorState { memory: &STATIC_MEMORY }
    }
}

impl Emulator2 for EmulatorState<'_> {
    fn graphic_memory(&self) -> Vec<u8> {
        self.memory.read().unwrap().to_vec()
        // let mut result: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        // result.clone_from_slice(&_STATIC_MEMORY[0..MEMORY_SIZE]);
        // result
    }

    fn write_memory(&mut self, address: usize, value: u8) {
        println!("  WRITING {:04x} = {:02x}", address, value);
        STATIC_MEMORY.write().unwrap()[address] = value;
    }

    fn run_one_frame(&self) {
        println!("run_one_frame()");
        std::thread::sleep(Duration::from_millis(500));
    }
}

pub(crate) fn main() {
    let mut e2 = EmulatorState::new();
    let mut e3 = EmulatorState::new();
    let t = thread::spawn(move || {
        let mut i = 0;
        loop {
            e2.run_one_frame();
            e2.write_memory(0, i);
            i += 1;
        }
    });
    loop {
        let s = format!("============= Memory: {:02x}", e3.memory.read().unwrap()[0]);
        log_time(&s);
        thread::sleep(Duration::from_millis(500));
    }
    t.join();
}