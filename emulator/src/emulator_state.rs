use crate::memory;
use crate::memory::GRAPHIC_MEMORY_SIZE;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SharedState {
    display: bool,
    megahertz: f64,
    in_1: u8,
    in_2: u8,
    is_paused: bool,

}

#[wasm_bindgen]
impl SharedState {
    pub fn new() -> SharedState {
        SharedState {
            display: false,
            megahertz: 2.0,
            in_1: 8,   // bit 3 is always 1
            in_2: 0,
            is_paused: false,
        }
    }
}

impl SharedState {
    pub fn graphic_memory(&self) -> [u8; GRAPHIC_MEMORY_SIZE] {
        let mut result: [u8; GRAPHIC_MEMORY_SIZE] = [0; GRAPHIC_MEMORY_SIZE];
        let memory = memory::STATIC_MEMORY.read().unwrap();
        result.clone_from_slice(&memory[0x2400..0x2400 + GRAPHIC_MEMORY_SIZE]);
        result
    }

    fn set_vbl(&mut self, value: bool) {
        self.display = value;
    }

    fn is_vbl(&self) -> bool { self.display }

    pub fn set_megahertz(&mut self, mhz: f64) {
        self.megahertz = mhz;
    }

    pub fn get_megahertz(&self) -> f64 {
        self.megahertz
    }

    pub fn set_bit_in_1(&mut self, bit: u8, value: bool) {
        let mask = 1 << bit;
        if value {
            self.in_1 |= mask;
        } else {
            self.in_1 &= ! mask;
        }
    }

    pub fn get_in_1(&self) -> u8 {
        self.in_1
    }

    pub fn set_bit_in_2(&mut self, bit: u8, value: bool) {
        let mask = 1 << bit;
        if value {
            self.in_2 |= mask;
        } else {
            self.in_2 &= ! mask;
        }
    }

    pub fn get_in_2(&self) -> u8 {
        self.in_2
    }

    pub fn is_paused(&self) -> bool { self.is_paused }
    pub fn pause(&mut self) { self.is_paused = true; }
    pub fn unpause(&mut self) { self.is_paused = false; }
}
