use crate::memory;
use crate::memory::GRAPHIC_MEMORY_SIZE;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SharedState {
    megahertz: f64,
    in_1: u8,
    in_2: u8,
    out_3: u8,  // sound
    out_5: u8,  // sound
    is_paused: bool,

}

#[wasm_bindgen]
impl SharedState {
    pub fn new() -> SharedState {
        SharedState {
            megahertz: 2.0,
            in_1: 8,   // bit 3 is always 1
            in_2: 0,
            out_3: 0,
            out_5: 0,
            is_paused: false,
        }
    }
}

impl SharedState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn graphic_memory(&self) -> [u8; GRAPHIC_MEMORY_SIZE] {
        let mut result: [u8; GRAPHIC_MEMORY_SIZE] = [0; GRAPHIC_MEMORY_SIZE];
        let memory = memory::STATIC_MEMORY.read().unwrap();
        result.clone_from_slice(&memory[0x2400..0x2400 + GRAPHIC_MEMORY_SIZE]);
        result
    }

    pub fn set_megahertz(&mut self, mhz: f64) {
        self.megahertz = mhz;
    }

    pub fn get_megahertz(&self) -> f64 {
        self.megahertz
    }

    pub fn set_out_3(&mut self, number: u8) {
        self.out_3 = number;
    }

    pub fn set_out_5(&mut self, number: u8) {
        self.out_5 = number;
    }

    /// Port 3: (discrete sounds)
    ///  bit 0=UFO (repeats)        SX0 0.raw
    ///  bit 1=Shot                 SX1 1.raw
    ///  bit 2=Flash (player die)   SX2 2.raw
    ///  bit 3=Invader die          SX3 3.raw
    ///  bit 4=Extended play        SX4
    ///  bit 5= AMP enable          SX5
    pub fn get_out_3(&self) -> u8 { self.out_3 }

    /// Port 5:
    ///  bit 0=Fleet movement 1     SX6 4.raw
    ///  bit 1=Fleet movement 2     SX7 5.raw
    ///  bit 2=Fleet movement 3     SX8 6.raw
    ///  bit 3=Fleet movement 4     SX9 7.raw
    ///  bit 4=UFO Hit              SX10 8.raw
    pub fn get_out_5(&self) -> u8 { self.out_5 }

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
