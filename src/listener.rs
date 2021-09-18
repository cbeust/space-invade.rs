/*
 * The main interface between the 8080 emulator and the computer.
 */
pub trait Listener: Send + Sync {
    fn set_vbl(&mut self, value: bool);
    fn is_vbl(&self) -> bool;
    fn byte_color(&self, address: usize) -> u8;
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

