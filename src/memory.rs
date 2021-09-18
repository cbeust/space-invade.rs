use std::fs::File;
use std::io::Read;
use crate::state::State;
use crate::opcodes::Opcode;
use std::sync::{Mutex, RwLock};
use crate::listener::Listener;
use lazy_static::lazy_static;

const MEMORY_SIZE: usize = 0x4000;
pub const SCREEN_WIDTH: usize = 0x20;  // 0x20 bytes (256 pixels)
pub const SCREEN_HEIGHT: usize = 0xe0;
pub const GRAPHIC_MEMORY_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub trait GraphicRenderer: Send {
    fn draw(&mut self, x: u8, y: u8, value: u8);
    fn color(&self, x: u8, y: u8) -> u8;
    fn display(&self);
}

static _STATIC_MEMORY: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];

lazy_static! {
    pub(crate) static ref STATIC_MEMORY: RwLock<[u8; MEMORY_SIZE]> = RwLock::new(_STATIC_MEMORY);
}

pub struct Memory<'a> {
    pub verbose: bool,
    pub listener: Option<&'a Mutex<dyn Listener>>,
}

impl<'a> Memory<'a> {
    pub(crate) fn new(listener: Option<&'a Mutex<dyn Listener>>) -> Self {
        Memory {
            verbose: false,
            listener,
        }
    }

    pub(crate) fn set_verbose(&mut self, v: bool) {
        self.verbose = v;
    }

    pub(crate) fn read_file(&mut self, file_name: &str, start: usize) {
        let mut file = File::open(file_name).expect("Couldn't open file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Couldn't read file");

        let mut state = State::default();
        state.pc = start;

        let mut i: usize = 0;
        for byte in buffer {
            self.write(i + state.pc, byte);
            i += 1;
        }
    }

    /*
        $2400: 0,0 -> 0,7
        $2401: 0,8 -> 0,f
        ...
        $241f: 0,f8 -> 0,ff
        $2420: 1,0 -> 1,7
        ...
        $243f: 1,f8 ->1,ff
        $2440: 2,0 -> 2,7
        ...
     */

    /*
        int i = 0x2400;  // Start of Video RAM
    for (int col = 0; col < WIDTH; col ++) {
        for (int row = HEIGHT; row > 0; row -= 8) {
            for (int j = 0; j < 8; j++) {
                int idx = (row - j) * WIDTH + col;

                if (ram->mem[i] & 1 << j) {
                    pix[idx] = 0xFFFFFF;
                } else {
                    pix[idx] = 0x000000;
                }
            }

            i++;
        }
    }
     */
    pub(crate) fn write(&mut self, address: usize, value: u8) {
        STATIC_MEMORY.write().unwrap()[address] = value;
        if self.verbose {
            println!("    mem[{:04x}={:02X}]", address, value    );
        }
    }

    pub(crate) fn read(&self, i: usize) -> u8 {
        STATIC_MEMORY.read().unwrap()[i]
    }

    pub(crate) fn read_word(&self, b0: u8, b1: u8) -> u8 {
        STATIC_MEMORY.read().unwrap()[Memory::to_word(b0, b1)]
    }

    pub(crate) fn write_word(&mut self, b0: u8, b1: u8, value: u8) {
        let address = Memory::to_word(b0, b1);
        STATIC_MEMORY.write().unwrap()[address] = value;
    }

    // pub(crate) fn disassemble_instructions(&self, start: usize, instruction_count: u16) {
    //     let mut pc = start;
    //     let mut n = instruction_count;
    //     while n > 0 {
    //         let op = OPCODES.get(&self.read(pc)).expect("Couldn't find opcode");
    //         let s =
    //             if op.size == 1 { op.display1() }
    //             else if op.size == 2 { op.display2(self.read(pc + 1))}
    //             else { op.display3(self.read(pc + 1), self.read(pc + 2))};
    //         println!("{:04x} {}", pc, s);
    //         pc += op.size;
    //         n -= 1;
    //     }
    // }

    pub(crate) fn disassemble(&self, opcode: &Opcode, pc: usize) -> (String, usize) {
        let formatted_opcode = match opcode.size {
            1 => opcode.display1(),
            2 => opcode.display2(self.read(pc + 1)),
            _ => opcode.display3(self.read(pc + 1), self.read(pc + 2)),
        };
        let result = format!("{:04x}: {}", pc, formatted_opcode);
        (result, opcode.size)
    }

    pub(crate) fn to_word(b1: u8, b2: u8) -> usize {
        return ((b2 as u16) << 8 | b1 as u16) as usize;
    }
}
