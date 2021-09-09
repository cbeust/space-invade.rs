use crate::memory;
use crate::memory::GRAPHIC_MEMORY_SIZE;
use crate::listener::Listener;

pub struct EmulatorState {
    buffer: [u8; memory::GRAPHIC_MEMORY_SIZE],
    display: bool,
    megahertz: f64,
    in_1: u8,
    in_2: u8,
    is_paused: bool,

}

impl EmulatorState {
    pub(crate) fn new() -> EmulatorState {
        EmulatorState {
            buffer: [0; memory::GRAPHIC_MEMORY_SIZE],
            display: false,
            megahertz: 2.0,
            in_1: 8,   // bit 3 is always 1
            in_2: 0,
            is_paused: false,
        }
    }
}

impl Listener for EmulatorState {
    fn on_draw(&mut self, address: usize, value: u8) {
        if ! (0..GRAPHIC_MEMORY_SIZE).contains(&address) {
            panic!("ILLEGAL GRAPHIC MEMORY ADDRESS: {:04x}", address);
        }
        if value != 0 && address == 0xc1e{
            println!("graphic {:04x}={:02x}", address, value);
        }
        self.buffer[address] = value;
    }

    fn set_vbl(&mut self, value: bool) {
        self.display = value;
    }

    fn is_vbl(&self) -> bool { self.display }

    fn byte_color(&self, address: usize) -> u8 {
        if address >= GRAPHIC_MEMORY_SIZE {
            panic!("BEYOND GRAPHIC MEMORY");
        }
        self.buffer[address]
    }

    fn set_megahertz(&mut self, mhz: f64) {
        self.megahertz = mhz;
    }

    fn get_megahertz(&self) -> f64 {
        self.megahertz
    }

    fn set_bit_in_1(&mut self, bit: u8, value: bool) {
        let mask = 1 << bit;
        if value {
            self.in_1 |= mask;
        } else {
            self.in_1 &= ! mask;
        }
    }

    fn get_in_1(&self) -> u8 {
        self.in_1
    }

    fn set_bit_in_2(&mut self, bit: u8, value: bool) {
        let mask = 1 << bit;
        if value {
            self.in_2 |= mask;
        } else {
            self.in_2 &= ! mask;
        }
    }

    fn get_in_2(&self) -> u8 {
        self.in_2
    }

    fn is_paused(&self) -> bool { self.is_paused }
    fn pause(&mut self) { self.is_paused = true; }
    fn unpause(&mut self) { self.is_paused = false; }
}
