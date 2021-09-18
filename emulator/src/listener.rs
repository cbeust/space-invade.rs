/*
 * The main interface between the 8080 emulator and the computer.
 */
use crate::memory::GRAPHIC_MEMORY_SIZE;

pub trait Listener: Send + Sync {
    fn graphic_memory(&self) -> [u8; GRAPHIC_MEMORY_SIZE];
    fn set_vbl(&mut self, value: bool);
    fn is_vbl(&self) -> bool;
    fn set_megahertz(&mut self, mhz: f64);
    fn get_megahertz(&self) -> f64;
    fn set_bit_in_1(&mut self, bit: u8, value: bool);
    fn get_in_1(&self) -> u8;
    fn set_bit_in_2(&mut self, bit: u8, value: bool);
    fn get_in_2(&self) -> u8;

    fn is_paused(&self) -> bool;
    fn pause(&mut self);
    fn unpause(&mut self);
}

