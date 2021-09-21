
const MEMORY_SIZE: usize = 0x4000;

use lazy_static::lazy_static;
use std::sync::RwLock;
use std::sync::Mutex;
use std::time::Duration;
use std::thread;
use emulator::opcodes::STA;
use crate::log_time;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Read;
use emulator::opcodes::*;
use emulator::opcodes;
use emulator::memory::Memory;
use emulator::state::{Psw, Cpu};

#[derive(PartialEq)]
pub enum StepStatus {
    Continue,
    Paused,
    Success(String),
    Failure(String)
}

pub struct StepResult {
    pub status: StepStatus,
    pub cycles: u8,
}

// pub struct Emulator<'a> {
//     memory: Box<Memory<'a>>,
//     state: Option<State>,
//     shift_register: u16,
//     shift_register_offset: u8,
//     output_buffer: Vec<char>,
// }

struct State {
    memory: Vec<u8>,
    cpu: Cpu,
    input_1: u8,
    input_2: u8,
}

static SHARED_STATE: Lazy<RwLock<State>> = Lazy::new(|| {
    RwLock::new(State {
        memory: vec![0; MEMORY_SIZE],
        cpu: Cpu::default(),
        input_1: 0,
        input_2: 0,
    })
});

const VERBOSE: bool = false;
static mut VERBOSE_DISASSEMBLE: bool = false;
const VERBOSE_GRAPHIC: bool = true;
const VERBOSE_DISASSEMBLE_SECTION: bool = false;
const DISASSEMBLE_SECTION_START: usize = 0x1439;
const DISASSEMBLE_SECTION_END: usize = 0x1447;
// const DISASSEMBLE_SECTION_START: usize = 0x1439;
// const DISASSEMBLE_SECTION_END: usize = 0x1447;
const VERBOSE_MEMORY: bool = false;

pub trait Emulator {
    fn run_one_frame(&mut self, verbose: bool) -> u64;
    fn is_paused(&self) -> bool;
    fn pause(&mut self);
    fn unpause(&mut self);

    fn write_memory(&mut self, address: usize, value: u8);
    fn memory(&self) -> Vec<u8>;
    fn read_memory(&self, address: usize) -> u8;
    fn set_input_1(&mut self, bit: u8, value: bool);
    fn input_1(&self) -> u8;
    fn set_input_2(&mut self, bit: u8, value: bool);
    fn input_2(&self) -> u8;
    fn megahertz(&self) -> f32;
}

pub struct Runner{
    paused: bool,
    shift_register: u16,
    shift_register_offset: u8,
}

impl Runner {
    pub fn new() -> Runner {
        Memory::read_file("space-invaders.rom", 0);
        let cpu = &mut SHARED_STATE.write().unwrap().cpu;
        cpu.pc = 0;
        Runner {
            paused: false,
            shift_register: 0,
            shift_register_offset: 0,
        }
    }

    pub const WIDTH: u16 = 224;
    pub const HEIGHT: u16 = 256;

    // pub fn new_space_invaders() -> Emulator<'static> {
    //     // let mut memory = Memory::new2();
    //     let mut memory = Memory::new(None);
    //     memory.read_file("space-invaders.rom", 0);
    //     Emulator::new(Box::new(memory), 0)
    // }
    //
    // pub fn new(memory: Box<Memory>, pc: usize) -> Emulator {
    //     Emulator { memory,
    //         shift_register: 0,
    //         shift_register_offset: 0,
    //         state: Some(State::new(pc)),
    //         output_buffer: Vec::new(),
    //     }
    // }

    pub fn step(&mut self, verbose: bool) -> StepResult {
        if self.paused {
            return StepResult { status: StepStatus::Paused, cycles: 0 };
        }

        let pc = SHARED_STATE.read().unwrap().cpu.pc;
        let op: u8 = self.read_memory(pc);
        let opcode = OPCODES.get(&op).expect(format!("Couldn't find opcode {:02x} at pc {:04x}",
                                                     op, pc).as_str());
        let mut pc_was_assigned = false;
        let byte1 = self.read_memory(pc + 1);
        let byte2 = self.read_memory(pc + 2);
        let word = Memory::to_word(byte1, byte2);
        let cycles;

        unsafe {
            if VERBOSE_DISASSEMBLE || (VERBOSE_DISASSEMBLE_SECTION &&
                (DISASSEMBLE_SECTION_START..=DISASSEMBLE_SECTION_END).contains(&pc)) {
                let memory = &SHARED_STATE.read().unwrap().memory;
                let line1 = Memory::disassemble(&opcode, pc);
                println!("{}    {:10}", line1.0, SHARED_STATE.read().unwrap().cpu.disassemble());
            }
        }

        let mut test_status = StepResult { status: StepStatus::Continue, cycles: 0};

        let mut cpu = &mut SHARED_STATE.write().unwrap().cpu;
        match op {
            opcodes::NOP => {
                cycles = 4;
            },
            opcodes::INX_B => {
                cpu.c = ((cpu.c as u16) + 1) as u8;
                if cpu.c == 0 {
                    cpu.b = ((cpu.b as u16) + 1) as u8;
                }
                cycles = 5;
            },
            opcodes::INX_D => {
                cpu.e = ((cpu.e as u16) + 1) as u8;
                if cpu.e == 0 {
                    cpu.d = ((cpu.d as u16) + 1) as u8;
                }
                cycles = 5;
            },
            opcodes::INX_SP => {
                if cpu.sp == 0xffff {
                    cpu.sp = 0;
                } else {
                    cpu.sp += 1;
                }
                cycles = 5;
            },
            opcodes::INX_H => {
                cpu.l = ((cpu.l as u16) + 1) as u8;
                if cpu.l == 0 {
                    cpu.h = ((cpu.h as u16) + 1) as u8;
                }
                cycles = 5;
            },
            opcodes::DCX_B => {
                cpu.c = ((cpu.c as i8) - 1) as u8;
                if cpu.c == 0xff {
                    cpu.b = ((cpu.b as i8) - 1) as u8;
                }
                cycles = 5;
            },
            opcodes::DCX_D => {
                cpu.e = ((cpu.e as i8) - 1) as u8;
                if cpu.e == 0xff {
                    cpu.d = ((cpu.d as i8) - 1) as u8;
                }
                cycles = 5;
            },
            opcodes::DCX_H => {
                let m = cpu.m() as i32 - 1;
                cpu.h = (m >> 8) as u8;
                cpu.l = m as u8;
                cycles = 5;
            },
            opcodes::DCX_SP => {
                if cpu.sp == 0 {
                    cpu.sp = 0xffff as usize;
                } else {
                    cpu.sp -= 1;
                }
                cycles = 5;
            },
            opcodes::DAD_B => {
                cpu.add_hl(cpu.c, cpu.b);
                cycles = 10;
            },
            opcodes::DAD_D => {
                cpu.add_hl(cpu.e, cpu.d);
                cycles = 10;
            },
            opcodes::DAD_H => {
                // not tested by cpudiag
                cpu.add_hl(cpu.l, cpu.h);
                cycles = 10;
            },
            opcodes::DAD_SP => {
                cpu.add_hl(cpu.sp as u8, cpu.sp as u8 - 1);
                cycles = 10;
            },
            opcodes::RAL => {
                // high order bit
                let hob = (cpu.psw.a & 0x80) >> 7;
                let carry_value = Psw::to_u8(cpu.psw.carry) as u8;
                cpu.psw.a = (cpu.psw.a << 1) | carry_value;
                cpu.psw.carry = if hob == 1 { true } else { false };
                cycles = 4;
            },
            opcodes::RAR => {
                // low order bit
                let lob = cpu.psw.a & 1;
                let carry_value = Psw::to_u8(cpu.psw.carry) as u8;
                cpu.psw.a = cpu.psw.a >> 1 | (carry_value << 7);
                cpu.psw.carry = if lob == 1 { true } else { false };
                cycles = 4;
            },
            opcodes::RRC => {
                // low order bit
                let lob = cpu.psw.a & 1;
                cpu.psw.carry = lob != 0;
                cpu.psw.a = (lob << 7) | cpu.psw.a >> 1;
                cycles = 4;
            },
            opcodes::RLC => {
                // high order bit
                let hob = (cpu.psw.a & 0x80) >> 7;
                cpu.psw.carry = hob != 0;
                cpu.psw.a = (cpu.psw.a << 1) | hob;
                cycles = 4;
            },
            opcodes::LDAX_B => {
                cpu.psw.a = Memory::read_from_bytes(cpu.c, cpu.b);
                cycles = 7;
            },
            opcodes::LDAX_D => {
                cpu.psw.a = Memory::read_from_bytes(cpu.e, cpu.d);
                cycles = 7;
            },
            opcodes::STAX_B => {
                Memory::write_word(cpu.c, cpu.b, cpu.psw.a);
                cycles = 7;
            },
            opcodes::STAX_D => {
                Memory::write_word(cpu.e, cpu.d, cpu.psw.a);
                cycles = 7;
            },
            opcodes::LHLD => {
                cpu.l = self.read_memory(word);
                cpu.h = self.read_memory(word + 1);
                cycles = 16;
            },
            opcodes::SHLD => {
                Memory::write(word, cpu.l);
                Memory::write(word + 1, cpu.h);
                cycles = 16;
            },
            opcodes::LDA => {
                cpu.psw.a = self.read_memory(word);
                cycles = 13;
            },
            opcodes::STA => {
                Memory::write(word, cpu.psw.a);
                cycles = 13;
            },
            opcodes::MVI_A => {
                cpu.psw.a = byte1;
                cycles = 7;
            },
            opcodes::MVI_B => {
                cpu.b = byte1;
                cycles = 7;
            },
            opcodes::MVI_C => {
                cpu.c = byte1;
                cycles = 7;
            },
            opcodes::MVI_D => {
                cpu.d = byte1;
                cycles = 7;
            },
            opcodes::MVI_E => {
                cpu.e = byte1;
                cycles = 7;
            },
            opcodes::MVI_H => {
                cpu.h = byte1;
                cycles = 7;
            },
            opcodes::MVI_L => {
                cpu.l = byte1;
                cycles = 7;
            },
            opcodes::MVI_M => {
                Memory::write(cpu.m(), byte1);
                cycles = 10;
            },
            opcodes::INR_A => {
                cpu.psw.a = cpu.inr(cpu.psw.a);
                cycles = 5;
            },
            opcodes::INR_B => {
                cpu.b = cpu.inr(cpu.b);
                cycles = 5;
            },
            opcodes::INR_C => {
                cpu.c = cpu.inr(cpu.c);
                cycles = 5;
            },
            opcodes::INR_D => {
                cpu.d = cpu.inr(cpu.d);
                cycles = 5;
            },
            opcodes::INR_E => {
                cpu.e = cpu.inr(cpu.e);
                cycles = 5;
            },
            opcodes::INR_H => {
                cpu.h = cpu.inr(cpu.h);
                cycles = 5;
            },
            opcodes::INR_L => {
                cpu.l = cpu.inr(cpu.l);
                cycles = 5;
            },
            opcodes::INR_M => {
                Memory::write(cpu.m(), cpu.inr(self.read_memory(cpu.m())));
                cycles = 10;
            },
            opcodes::DCR_A => {
                cpu.psw.a = cpu.dec(cpu.psw.a);
                cycles = 5;
            },
            opcodes::DCR_B => {
                cpu.b = cpu.dec(cpu.b);
                cycles = 5;
            },
            opcodes::DCR_C => {
                cpu.c = cpu.dec(cpu.c);
                cycles = 5;
            },
            opcodes::DCR_D => {
                cpu.d = cpu.dec(cpu.d);
                cycles = 5;
            },
            opcodes::DCR_E => {
                cpu.e = cpu.dec(cpu.e);
                cycles = 5;
            },
            opcodes::DCR_H => {
                cpu.h = cpu.dec(cpu.h);
                cycles = 5;
            },
            opcodes::DCR_L => {
                cpu.l = cpu.dec(cpu.l);
                cycles = 5;
            },
            opcodes::DCR_M => {
                Memory::write(cpu.m(), cpu.dec(self.read_memory(cpu.m())));
                cycles = 10;
            },
            opcodes::MOV_B_A => {
                cpu.b = cpu.psw.a;
                cycles = 5;
            },
            opcodes::MOV_B_B => {
                cycles = 5;
            },
            opcodes::MOV_B_C => {
                cpu.b = cpu.c;
                cycles = 5;
            },
            opcodes::MOV_B_D => {
                cpu.b = cpu.d;
                cycles = 5;
            },
            opcodes::MOV_B_E => {
                cpu.b = cpu.e;
                cycles = 5;
            },
            opcodes::MOV_B_H => {
                cpu.b = cpu.h;
                cycles = 5;
            },
            opcodes::MOV_B_L => {
                cpu.b = cpu.l;
                cycles = 5;
            },
            opcodes::MOV_A_M => {
                cpu.psw.a = self.read_memory(cpu.m());
                cycles = 7;
            },
            opcodes::MOV_B_M => {
                cpu.b = self.read_memory(cpu.m());
                cycles = 7;
            },
            opcodes::MOV_C_M => {
                cpu.c = self.read_memory(cpu.m());
                cycles = 7;
            },
            opcodes::MOV_D_M => {
                cpu.d = self.read_memory(cpu.m());
                cycles = 7;
            },
            opcodes::MOV_E_M => {
                cpu.e = self.read_memory(cpu.m());
                cycles = 7;
            },
            opcodes::MOV_H_M => {
                cpu.h = self.read_memory(cpu.m());
                cycles = 7;
            },
            opcodes::MOV_L_M => {
                cpu.l = self.read_memory(cpu.m());
                cycles = 7;
            },
            opcodes::MOV_C_A => {
                cpu.c = cpu.psw.a;
                cycles = 5;
            },
            opcodes::MOV_C_B => {
                cpu.c = cpu.b;
                cycles = 5;
            },
            opcodes::MOV_C_C => {
                cycles = 5;
            },
            opcodes::MOV_C_D => {
                cpu.c = cpu.d;
                cycles = 5;
            },
            opcodes::MOV_C_E => {
                cpu.c = cpu.e;
                cycles = 5;
            },
            opcodes::MOV_C_H => {
                cpu.c = cpu.h;
                cycles = 5;
            },
            opcodes::MOV_C_L => {
                cpu.c = cpu.l;
                cycles = 5;
            },
            opcodes::MOV_D_A => {
                cpu.d = cpu.psw.a;
                cycles = 5;
            },
            opcodes::MOV_D_B => {
                cpu.d = cpu.b;
                cycles = 5;
            },
            opcodes::MOV_D_C => {
                cpu.d = cpu.c;
                cycles = 5;
            },
            opcodes::MOV_D_D => {
                cycles = 5;
            },
            opcodes::MOV_D_E => {
                cpu.d = cpu.e;
                cycles = 5;
            },
            opcodes::MOV_D_H => {
                cpu.d = cpu.h;
                cycles = 5;
            },
            opcodes::MOV_D_L => {
                cpu.d = cpu.l;
                cycles = 5;
            },
            opcodes::MOV_E_A => {
                // not tested by cpudiag, was bogus
                cpu.e = cpu.psw.a;
                cycles = 5;
            },
            opcodes::MOV_E_B => {
                cpu.e = cpu.b;
                cycles = 5;
            },
            opcodes::MOV_E_C => {
                cpu.e = cpu.c;
                cycles = 5;
            },
            opcodes::MOV_E_D => {
                cpu.e = cpu.d;
                cycles = 5;
            },
            opcodes::MOV_E_E => {
                cycles = 5;
            },
            opcodes::MOV_E_H => {
                cpu.e = cpu.h;
                cycles = 5;
            },
            opcodes::MOV_E_L => {
                cpu.e = cpu.l;
                cycles = 5;
            },
            opcodes::MOV_H_A => {
                cpu.h = cpu.psw.a;
                cycles = 5;
            },
            opcodes::MOV_H_B => {
                cpu.h = cpu.b;
                cycles = 5;
            },
            opcodes::MOV_H_C => {
                cpu.h = cpu.c;
                cycles = 5;
            },
            opcodes::MOV_H_D => {
                cpu.h = cpu.d;
                cycles = 5;
            },
            opcodes::MOV_H_E => {
                cpu.h = cpu.e;
                cycles = 5;
            },
            opcodes::MOV_H_H => {
                cycles = 5;
            },
            opcodes::MOV_H_L => {
                cpu.h = cpu.l;
                cycles = 5;
            },
            opcodes::MOV_L_A => {
                cpu.l = cpu.psw.a;
                cycles = 5;
            },
            opcodes::MOV_M_B => {
                Memory::write(cpu.m(), cpu.b);
                cycles = 7;
            },
            opcodes::MOV_M_C => {
                Memory::write(cpu.m(), cpu.c);
                cycles = 7;
            },
            opcodes::MOV_M_D => {
                Memory::write(cpu.m(), cpu.d);
                cycles = 7;
            },
            opcodes::MOV_M_E => {
                Memory::write(cpu.m(), cpu.e);
                cycles = 7;
            },
            opcodes::MOV_M_H => {
                Memory::write(cpu.m(), cpu.h);
                cycles = 7;
            },
            opcodes::MOV_M_L => {
                Memory::write(cpu.m(), cpu.l);
                cycles = 7;
            },
            opcodes::MOV_M_A => {
                // self.memory.disassemble_instructions(state.pc, 10);
                Memory::write(cpu.m(), cpu.psw.a);
                cycles = 7;
            },
            opcodes::MOV_L_B => {
                cpu.l = cpu.b;
                cycles = 5;
            },
            opcodes::MOV_L_C => {
                cpu.l = cpu.c;
                cycles = 5;
            },
            opcodes::MOV_L_D => {
                cpu.l = cpu.d;
                cycles = 5;
            },
            opcodes::MOV_L_E => {
                cpu.l = cpu.e;
                cycles = 5;
            },
            opcodes::MOV_L_H => {
                cpu.l = cpu.h;
                cycles = 5;
            },
            opcodes::MOV_L_L => {
                cycles = 5;
            },
            opcodes::LXI_B => {
                cpu.c = byte1;
                cpu.b = byte2;
                cycles = 10;
            },
            opcodes::LXI_D => {
                cpu.e = byte1;
                cpu.d = byte2;
                cycles = 10;
            },
            opcodes::LXI_H => {
                cpu.l = byte1;
                cpu.h = byte2;
                cycles = 10;
            },
            opcodes::MOV_A_B => {
                cpu.psw.a = cpu.b;
                cycles = 5;
            },
            opcodes::MOV_A_C => {
                cpu.psw.a = cpu.c;
                cycles = 10;
            },
            opcodes::MOV_A_D => {
                cpu.psw.a = cpu.d;
                cycles = 10;
            },
            opcodes::MOV_A_E => {
                cpu.psw.a = cpu.e;
                cycles = 10;
            },
            opcodes::MOV_A_H => {
                cpu.psw.a = cpu.h;
                cycles = 10;
            },
            opcodes::MOV_A_L => {
                cpu.psw.a = cpu.l;
                cycles = 10;
            },
            opcodes::LXI_SP => {
                cpu.sp = word;
                cycles = 10;
            },
            opcodes::SUI => {
                let value = cpu.psw.a as i16 - byte1 as i16;
                cpu.psw.a = value as u8;
                cpu.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::SBI => {
                let value = cpu.psw.a as i16 - (byte1 as i16 + cpu.psw.carry as i16);
                cpu.psw.a = value as u8;
                cpu.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::ADD_A => {
                cpu.add(cpu.psw.a, 0);
                cycles = 4;
            }
            opcodes::ADD_B => {
                cpu.add(cpu.b, 0);
                cycles = 4;
            }
            opcodes::ADD_C => {
                cpu.add(cpu.c, 0);
                cycles = 4;
            }
            opcodes::ADD_D => {
                cpu.add(cpu.d, 0);
                cycles = 4;
            }
            opcodes::ADD_E => {
                cpu.add(cpu.e, 0);
                cycles = 4;
            }
            opcodes::ADD_H => {
                cpu.add(cpu.h, 0);
                cycles = 4;
            }
            opcodes::ADD_L => {
                cpu.add(cpu.l, 0);
                cycles = 4;
            }
            opcodes::ADD_M => {
                cpu.add(self.read_memory(cpu.m()), 0);
                cycles = 7;
            }
            opcodes::ADC_A => {
                cpu.add(cpu.psw.a, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_B => {
                cpu.add(cpu.b, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_C => {
                cpu.add(cpu.c, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_D => {
                cpu.add(cpu.d, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_E => {
                cpu.add(cpu.e, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_H => {
                cpu.add(cpu.h, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_L => {
                cpu.add(cpu.l, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_M => {
                cpu.add(self.read_memory(cpu.m()), Psw::to_u8(cpu.psw.carry));
                cycles = 7;
            }
            opcodes::SUB_A => {
                cpu.sub(cpu.psw.a, 0);
                cycles = 4;
            }
            opcodes::SUB_B => {
                cpu.sub(cpu.b, 0);
                cycles = 4;
            }
            opcodes::SUB_C => {
                cpu.sub(cpu.c, 0);
                cycles = 4;
            }
            opcodes::SUB_D => {
                cpu.sub(cpu.d, 0);
                cycles = 4;
            }
            opcodes::SUB_E => {
                cpu.sub(cpu.e, 0);
                cycles = 4;
            }
            opcodes::SUB_H => {
                cpu.sub(cpu.h, 0);
                cycles = 4;
            }
            opcodes::SUB_L => {
                cpu.sub(cpu.l, 0);
                cycles = 4;
            }
            opcodes::SUB_M => {
                cpu.sub(self.read_memory(cpu.m()), 0);
                cycles = 7;
            }
            opcodes::SBB_A => {
                cpu.sub(cpu.psw.a, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_B => {
                cpu.sub(cpu.b, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_C => {
                cpu.sub(cpu.c, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_D => {
                cpu.sub(cpu.d, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_E => {
                cpu.sub(cpu.e, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_H => {
                cpu.sub(cpu.h, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_L => {
                cpu.sub(cpu.l, Psw::to_u8(cpu.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_M => {
                cpu.sub(self.read_memory(cpu.m()), Psw::to_u8(cpu.psw.carry));
                cycles = 7;
            }
            opcodes::ADI => {
                let value = cpu.psw.a as i16 + byte1 as i16;
                cpu.psw.a = value as u8;
                cpu.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::ACI => {
                let value = cpu.psw.a as i16 + byte1 as i16 + Psw::to_u8(cpu.psw.carry) as i16;
                cpu.psw.a = value as u8;
                cpu.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::JMP => {
                if word == 0 {
                    todo!("Need to implement output routine for tests");
                    // let output: String = self.output_buffer.clone().into_iter().collect();
                    // println!("{}", output);
                } else {
                    cpu.pc = word;
                    pc_was_assigned = true;
                }
                cycles = 10;
            },
            opcodes::RPO => {
                pc_was_assigned = cpu.ret(! cpu.psw.parity);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RPE => {
                pc_was_assigned = cpu.ret(cpu.psw.parity);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RNC => {
                pc_was_assigned = cpu.ret(! cpu.psw.carry);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RC => {
                pc_was_assigned = cpu.ret(cpu.psw.carry);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RP => {
                pc_was_assigned = cpu.ret(! cpu.psw.sign);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RM => {
                pc_was_assigned = cpu.ret(cpu.psw.sign);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RZ => {
                pc_was_assigned = cpu.ret(cpu.psw.zero);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RNZ => {
                pc_was_assigned = cpu.ret(! cpu.psw.zero);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RET => {
                pc_was_assigned = cpu.ret(true);
                cycles = 11;
            },
            opcodes::POP_B => {
                cpu.c = self.read_memory(cpu.sp);
                cpu.b = self.read_memory(cpu.sp + 1);
                cpu.sp += 2;
                cycles = 10;
            },
            opcodes::POP_D => {
                cpu.e = self.read_memory(cpu.sp);
                cpu.d = self.read_memory(cpu.sp + 1);
                cpu.sp += 2;
                cycles = 10;
            },
            opcodes::POP_H => {
                cpu.l = self.read_memory(cpu.sp);
                cpu.h = self.read_memory(cpu.sp + 1);
                cpu.sp += 2;
                cycles = 10;
            },
            opcodes::PUSH_B => {
                Memory::write(cpu.sp - 1, cpu.b);
                Memory::write(cpu.sp - 2, cpu.c);
                cpu.sp -= 2;
                cycles = 11;
            },
            opcodes::PUSH_D => {
                Memory::write(cpu.sp - 1, cpu.d);
                Memory::write(cpu.sp - 2, cpu.e);
                cpu.sp -= 2;
                cycles = 11;
            },
            opcodes::PUSH_H => {
                Memory::write(cpu.sp - 1, cpu.h);
                Memory::write(cpu.sp - 2, cpu.l);
                cpu.sp -= 2;
                cycles = 11;
            },
            opcodes::CC => {
                if cpu.psw.carry {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CPO => {
                if ! cpu.psw.parity {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CPE => {
                if cpu.psw.parity {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CM => {
                if cpu.psw.sign {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CP => {
                if ! cpu.psw.sign {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CNZ => {
                if ! cpu.psw.zero {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CZ => {
                if cpu.psw.zero {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CNC => {
                if ! cpu.psw.carry {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::STC => {
                cpu.psw.carry = true;
                cycles = 4;
            },
            opcodes::CMC => {
                cpu.psw.carry = ! cpu.psw.carry;
                cycles = 4;
            },
            opcodes::CMA => {
                cpu.psw.a ^= 0xff;
                cycles = 4;
            },
            opcodes::DAA => {
                let mut a: u16 = cpu.psw.a as u16;
                // least significant bits
                let lsb = a & 0x0f;
                if lsb > 9 || cpu.psw.auxiliary_carry {
                    a += 6;
                    cpu.psw.auxiliary_carry = (lsb + 6) > 0xf;
                };
                // most significant bits
                let mut msb = (a & 0xf0) >> 4;
                if (msb > 9) || cpu.psw.carry { msb += 6; }
                a = (msb << 4) | (a & 0xf);
                cpu.psw.auxiliary_carry = (msb + 6) > 0xf;
                cpu.set_arithmetic_flags(a as i16);
                cpu.psw.a = a as u8;

                cycles = 4;
            },
            opcodes::CALL => {
                cycles = 17;
                if cfg!(test) {
                    // In test mode, this function returns 1 for successful test, 2 for failure
                    if word == 5 && cpu.c == 9 {
                        // print message at address word(D, E)
                        let ind = Memory::to_word(cpu.e, cpu.d);
                        if ind == 0x174 {
                            test_status.status = StepStatus::Success("Success".into());
                        } else if ind == 0x18b {
                            let sp = cpu.sp + 4;
                            let pc = Memory::to_word(self.read_memory(sp), self.read_memory(sp+1))
                                - 6;
                            for i in 0..20 {
                                println!("sp + {}: {:02x}", i, self.read_memory(sp + i));
                            }
                            let s = format!("Failure at pc {:04x}", pc);
                            test_status.status = StepStatus::Failure(s);
                        } else {
                            cpu.call(word);
                            pc_was_assigned = true;
                        }
                    } else {
                        cpu.call(word);
                        pc_was_assigned = true;
                    }
                } else {
                    cpu.call(word);
                    pc_was_assigned = true;
                }
            },
            opcodes::ANI => {
                let value = cpu.psw.a & byte1;
                cpu.psw.a = value;
                cpu.set_logic_flags(value.into());
                cycles = 7;
            },
            opcodes::ORI => {
                let value = cpu.psw.a | byte1;
                cpu.psw.a = value;
                cpu.set_logic_flags(value.into());
                cycles = 7;
            },
            opcodes::XRI => {
                cpu.psw.a = cpu.xra(byte1);
                cycles = 7;
            },
            opcodes::XRA_A => {
                cpu.psw.a = cpu.xra(cpu.psw.a);
                cycles = 4;
            },
            opcodes::XRA_B => {
                cpu.psw.a = cpu.xra(cpu.b);
                cycles = 4;
            },
            opcodes::XRA_C => {
                cpu.psw.a = cpu.xra(cpu.c);
                cycles = 4;
            },
            opcodes::XRA_D => {
                cpu.psw.a = cpu.xra(cpu.d);
                cycles = 4;
            },
            opcodes::XRA_E => {
                cpu.psw.a = cpu.xra(cpu.e);
                cycles = 4;
            },
            opcodes::XRA_H => {
                cpu.psw.a = cpu.xra(cpu.h);
                cycles = 4;
            },
            opcodes::XRA_L => {
                cpu.psw.a = cpu.xra(cpu.l);
                cycles = 4;
            },
            opcodes::XRA_M => {
                cpu.psw.a = cpu.xra(self.read_memory(cpu.m()));
                cycles = 7;
            },
            opcodes::ANA_A => {
                cpu.psw.a = cpu.and(cpu.psw.a);
                cycles = 4;
            },
            opcodes::ANA_B => {
                cpu.psw.a = cpu.and(cpu.b);
                cycles = 4;
            },
            opcodes::ANA_C => {
                cpu.psw.a = cpu.and(cpu.c);
                cycles = 4;
            },
            opcodes::ANA_D => {
                cpu.psw.a = cpu.and(cpu.d);
                cycles = 4;
            },
            opcodes::ANA_E => {
                cpu.psw.a = cpu.and(cpu.e);
                cycles = 4;
            },
            opcodes::ANA_H => {
                cpu.psw.a = cpu.and(cpu.h);
                cycles = 4;
            },
            opcodes::ANA_L => {
                cpu.psw.a = cpu.and(cpu.l);
                cycles = 4;
            },
            opcodes::ANA_M => {
                cpu.psw.a = cpu.and(self.read_memory(cpu.m()));
                cycles = 7;
            },
            opcodes::ORA_A => {
                cpu.psw.a = cpu.or(cpu.psw.a);
                cycles = 4;
            },
            opcodes::ORA_B => {
                cpu.psw.a = cpu.or(cpu.b);
                cycles = 4;
            },
            opcodes::ORA_C => {
                cpu.psw.a = cpu.or(cpu.c);
                cycles = 4;
            },
            opcodes::ORA_D => {
                cpu.psw.a = cpu.or(cpu.d);
                cycles = 4;
            },
            opcodes::ORA_E => {
                cpu.psw.a = cpu.or(cpu.e);
                cycles = 4;
            },
            opcodes::ORA_H => {
                cpu.psw.a = cpu.or(cpu.h);
                cycles = 4;
            },
            opcodes::ORA_L => {
                cpu.psw.a = cpu.or(cpu.l);
                cycles = 4;
            },
            opcodes::ORA_M => {
                cpu.psw.a = cpu.or(self.read_memory(cpu.m()));
                cycles = 7;
            },
            opcodes::XTHL => {
                let l = self.read_memory(cpu.sp);
                Memory::write(cpu.sp, cpu.l);
                cpu.l = l;
                let h = self.read_memory(cpu.sp + 1);
                Memory::write(cpu.sp + 1, cpu.h);
                cpu.h = h;
                cycles = 18;
            },
            opcodes::JPO => {
                pc_was_assigned = cpu.jump_if_flag(word, ! cpu.psw.parity);
                cycles = 10;
            },
            opcodes::JPE => {
                pc_was_assigned = cpu.jump_if_flag(word, cpu.psw.parity);
                cycles = 10;
            },
            opcodes::JNZ => {
                pc_was_assigned = cpu.jump_if_flag(word, ! cpu.psw.zero);
                cycles = 10;
            },
            opcodes::JZ => {
                pc_was_assigned = cpu.jump_if_flag(word, cpu.psw.zero);
                cycles = 10;
            },
            opcodes::JP => {
                pc_was_assigned = cpu.jump_if_flag(word, ! cpu.psw.sign);
                cycles = 10;
            },
            opcodes::JM => {
                pc_was_assigned = cpu.jump_if_flag(word, cpu.psw.sign);
                cycles = 10;
            },
            opcodes::JC => {
                pc_was_assigned = cpu.jump_if_flag(word, cpu.psw.carry);
                cycles = 10;
            },
            opcodes::JNC => {
                pc_was_assigned = cpu.jump_if_flag(word, ! cpu.psw.carry);
                cycles = 10;
            },
            opcodes::XCHG => {
                let h = cpu.h;
                cpu.h = cpu.d;
                cpu.d = h;
                let l = cpu.l;
                cpu.l = cpu.e;
                cpu.e = l;
                cycles = 4;
            },
            opcodes::PUSH_PSW => {
                Memory::write(cpu.sp - 1, cpu.psw.a);
                Memory::write(cpu.sp - 2, (cpu.psw.value() & 0xff) as u8);
                cpu.sp -= 2;
                cycles = 11;
            },
            opcodes::POP_PSW => {
                cpu.psw.a = self.read_memory(cpu.sp + 1);
                cpu.psw.set_flags(self.read_memory(cpu.sp));
                cpu.sp += 2;
                cycles = 10;
            },
            opcodes::CPI => {
                cpu.cmp(byte1);
                cycles = 7;
            },
            opcodes::CMP_B => {
                cpu.cmp(cpu.b);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_C => {
                cpu.cmp(cpu.c);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_D => {
                cpu.cmp(cpu.d);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_E => {
                cpu.cmp(cpu.e);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_H => {
                cpu.cmp(cpu.h);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_L => {
                cpu.cmp(cpu.l);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_M => {
                cpu.cmp(self.read_memory(cpu.m()));
                cycles = 7;
            },
            opcodes::CMP_A => {
                cpu.cmp(cpu.psw.a);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::SPHL => {
                cpu.sp = ((cpu.h as u16) << 8) as usize | cpu.l as usize;
                cycles = 5;
            },
            opcodes::PCHL => {
                cpu.pc = ((cpu.h as u16) << 8) as usize | cpu.l as usize;
                pc_was_assigned = true;
                cycles = 5;
            },
            opcodes::EI => {
                cpu.enable_interrupts = true;
                cycles = 4;
            }
            opcodes::DI => {
                cpu.enable_interrupts = false;
                cycles = 4;
            }
            opcodes::OUT => {
                match byte1 {
                    2 => {
                        self.shift_register_offset = cpu.psw.a & 0x7;
                    },
                    3 => {
                        // sound
                    },
                    4 => {
                        self.shift_register = ((cpu.psw.a as u16) << 8)
                            | (self.shift_register >> 8)
                    },
                    5 => {
                        // sound
                    },
                    6 => {
                        // watch dog
                    }
                    _ => {
                        println!("Unsupported OUT port: {}", byte1);
                    }
                }
                cycles = 10;
            }
            opcodes::IN => {
                match byte1 {
                    1 => {
                        // println!("IMPLEMENT IN 1");
                        // cpu.psw.a = self.memory.listener.unwrap().lock().unwrap().get_in_1();
                    },
                    2 => {
                        // println!("IMPLEMENT IN 2");
                        // cpu.psw.a = self.memory.listener.unwrap().lock().unwrap().get_in_2();
                    },
                    3 => {
                        // println!("IMPLEMENT IN 3");
                        // let shift_amount = 8 - self.shift_register_offset;
                        // cpu.psw.a = (self.shift_register >> shift_amount) as u8;
                    },
                    _ => {
                        panic!("Unsupported IN port: {}", byte1);
                    }
                }
                cycles = 10;
            }
            opcodes::RST_1 => {
                cpu.call(1 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            opcodes::RST_2 => {
                cpu.call(2 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            opcodes::RST_7 => {
                cpu.call(7 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            _ => panic!("Don't know how to run opcode: {:02x} at {:04x}", op, cpu.pc),
        }

        if ! pc_was_assigned {
            cpu.pc += opcode.size;
        }

        if cycles == 0 {
            panic!("Cycles not assigned");
        }

        if test_status.status != StepStatus::Continue {
            test_status
        } else {
            StepResult { status: StepStatus::Continue, cycles }
        }
    }

    fn interrupt(&mut self, interrupt_number: u8) {
        if SHARED_STATE.read().unwrap().cpu.enable_interrupts {
            // log_time(format!("Interrupt {}", interrupt_number).as_str());
            let cpu = &mut SHARED_STATE.write().unwrap().cpu;
            Memory::write(cpu.sp - 1, ((cpu.pc as u16 & 0xff00) >> 8) as u8);
            Memory::write(cpu.sp - 2, (cpu.pc as u16 & 0xff) as u8);
            cpu.sp -= 2;
            // Interrupt 0 goes to $0, 1 to $08, 2 to $10, etc...
            cpu.pc = (interrupt_number as usize) << 3;
        }
    }
}

impl Emulator for Runner {
    fn run_one_frame(&mut self, verbose: bool) -> u64 {
        let mut total_cycles: u64 = 0;
        let cycle_max = 33_000;
        while total_cycles < cycle_max / 2 {
            total_cycles += self.step(verbose).cycles as u64;
        }
        self.interrupt(1);

        while total_cycles < cycle_max {
            total_cycles += self.step(verbose).cycles as u64;
        }
        self.interrupt(2);

        total_cycles
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn unpause(&mut self) {
        self.paused = false;
    }

    fn write_memory(&mut self, address: usize, value: u8) {
        Memory::write(address, value);
    }

    fn memory(&self) -> Vec<u8> {
        SHARED_STATE.read().unwrap().memory.to_vec()
    }

    fn read_memory(&self, address: usize) -> u8 {
        Memory::read(address)
    }

    fn set_input_1(&mut self, bit: u8, value: bool) {
        println!("TODO: set_input_1");
    }

    fn input_1(&self) -> u8 {
        SHARED_STATE.read().unwrap().input_1
    }

    fn set_input_2(&mut self, bit: u8, value: bool) {
        println!("TODO: set_input_2");
    }

    fn input_2(&self) -> u8 {
        SHARED_STATE.read().unwrap().input_2
    }

    fn megahertz(&self) -> f32 {
        return 0.0;
    }
}

// pub(crate) fn main() {
//     let mut e2 = Runner::new();
//     let mut e3 = Runner::new();
//     let t = thread::spawn(move || {
//         let mut i = 0;
//         loop {
//             e2.run_one_frame(true);
//             println!("Writing {}, input_1: {}", i, e2.input_1());
//             e2.write_memory(0, i);
//             i += 1;
//         }
//     });
//     loop {
//         let value = e3.memory()[0];
//         let s = format!("============= Memory: {:02x}", value);
//         e3.set_input_1(value);
//         log_time(&s);
//         thread::sleep(Duration::from_millis(500));
//     }
// }
