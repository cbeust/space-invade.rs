use crate::{opcodes, VERBOSE_DISASSEMBLE, VERBOSE_DISASSEMBLE_SECTION, DISASSEMBLE_SECTION_START, DISASSEMBLE_SECTION_END};
use crate::opcodes::*;
use crate::memory::Memory;
use crate::state::*;
use crate::emulator_state::SharedState;
use std::thread;
use std::time::{SystemTime, Duration};
use lazy_static::lazy_static;
use std::sync::{Mutex, RwLock, MutexGuard};
use wasm_bindgen::prelude::*;
use once_cell::sync::OnceCell;

static SHARED_STATE: OnceCell<Mutex<SharedState>> = OnceCell::new();

// lazy_static! {
//     pub(crate) static ref SHARED_STATE: Mutex<SharedState> = Mutex::new(SharedState::new());
// }

#[cfg(target_arch = "wasm32")]
pub fn graphic_memory() -> Vec<u8> {
    let memory = crate::memory::STATIC_MEMORY.read().unwrap();
    memory[0x2400..0x2400 + crate::memory::GRAPHIC_MEMORY_SIZE].to_vec()
}

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

#[wasm_bindgen]
pub struct Emulator {
    memory: Box<Memory>,
    state: Option<State>,
    shift_register: u16,
    shift_register_offset: u8,
    output_buffer: Vec<char>,
}

pub const WIDTH: u16 = 224;
pub const HEIGHT: u16 = 256;

#[wasm_bindgen(js_namespace = console)]
pub fn start_emulator_wasm() {
    println!("Starting emulator wasm");
}

#[wasm_bindgen]
pub fn spawn_emulator() {
    //
    // Spawn the game logic in a separate thread. This logic will communicate with the
    // main thread (and therefore, the actual graphics on your screen) via the `listener`
    // object that this function receives in parameter.
    //
    thread::spawn(move || {
        let mut emulator = Emulator::new_space_invaders();
        let time_per_frame_ms = 16;
        loop {
            let start = SystemTime::now();
            // Run one frame
            let cycles = emulator.run_one_frame(false);
            let elapsed = start.elapsed().unwrap().as_millis();

            // Wait until we reach 16ms before running the next frame.
            // TODO: I'm not 100% sure the event pump is being invoked on a 16ms cadence,
            // which might explain why my game is going a bit too fast. I should actually
            // rewrite this logic to guarantee that it runs every 16ms
            if elapsed < time_per_frame_ms {
                std::thread::sleep(Duration::from_millis((time_per_frame_ms - elapsed) as u64));
            }
            let after_sleep = start.elapsed().unwrap().as_micros();
            if false {
                println!("Actual time frame: {}ms, after sleep: {} ms, cycles: {}",
                         elapsed,
                         after_sleep,
                         cycles);
            }

            SHARED_STATE.get().unwrap().lock().unwrap()
                .set_megahertz(cycles as f64 / after_sleep as f64);
        }
    });
}

impl Emulator {

    pub fn new_space_invaders() -> Emulator {
        let mut memory = Memory::new();
        memory.read_file("space-invaders.rom", 0);
        Emulator::new(Box::new(memory), 0)
    }

    pub fn new(memory: Box<Memory>, pc: usize) -> Emulator {
        Emulator { memory,
            shift_register: 0,
            shift_register_offset: 0,
            state: Some(State::new(pc)),
            output_buffer: Vec::new(),
        }
    }

    pub fn start_emulator() -> &'static Mutex<SharedState> {
        SHARED_STATE.set(Mutex::new(SharedState::new()));
        spawn_emulator();
        &SHARED_STATE.get().unwrap()
    }

    pub fn run_one_frame(&mut self, verbose: bool) -> u64 {
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

    pub fn step(&mut self, verbose: bool) -> StepResult {
        if SHARED_STATE.get().unwrap().lock().unwrap().is_paused() {
            return StepResult { status: StepStatus::Paused, cycles: 0 };
        }

        let state = &mut self.state.as_mut().unwrap();
        let op: u8 = self.memory.read(state.pc);
        let opcode = OPCODES.get(&op).expect(format!("Couldn't find opcode {:02x} at pc {:04x}",
                                                     op, state.pc).as_str());
        let mut pc_was_assigned = false;
        let byte1 = self.memory.read(state.pc + 1);
        let byte2 = self.memory.read(state.pc + 2);
        let word = Memory::to_word(byte1, byte2);
        let cycles;

        unsafe {
            if VERBOSE_DISASSEMBLE || (VERBOSE_DISASSEMBLE_SECTION &&
                (DISASSEMBLE_SECTION_START..=DISASSEMBLE_SECTION_END).contains(&state.pc)) {
                let line1 = self.memory.disassemble(&opcode, state.pc);
                println!("{}    {:10}", line1.0, state.disassemble());
            }
        }

        let mut test_status = StepResult { status: StepStatus::Continue, cycles: 0};

        match op {
            opcodes::NOP => {
                cycles = 4;
            },
            opcodes::INX_B => {
                state.c = ((state.c as u16) + 1) as u8;
                if state.c == 0 {
                    state.b = ((state.b as u16) + 1) as u8;
                }
                cycles = 5;
            },
            opcodes::INX_D => {
                state.e = ((state.e as u16) + 1) as u8;
                if state.e == 0 {
                    state.d = ((state.d as u16) + 1) as u8;
                }
                cycles = 5;
            },
            opcodes::INX_SP => {
                if state.sp == 0xffff {
                    state.sp = 0;
                } else {
                    state.sp += 1;
                }
                cycles = 5;
            },
            opcodes::INX_H => {
                state.l = ((state.l as u16) + 1) as u8;
                if state.l == 0 {
                    state.h = ((state.h as u16) + 1) as u8;
                }
                cycles = 5;
            },
            opcodes::DCX_B => {
                state.c = ((state.c as i8) - 1) as u8;
                if state.c == 0xff {
                    state.b = ((state.b as i8) - 1) as u8;
                }
                cycles = 5;
            },
            opcodes::DCX_D => {
                state.e = ((state.e as i8) - 1) as u8;
                if state.e == 0xff {
                    state.d = ((state.d as i8) - 1) as u8;
                }
                cycles = 5;
            },
            opcodes::DCX_H => {
                let m = state.m() as i32 - 1;
                state.h = (m >> 8) as u8;
                state.l = m as u8;
                cycles = 5;
            },
            opcodes::DCX_SP => {
                if state.sp == 0 {
                    state.sp = 0xffff as usize;
                } else {
                    state.sp -= 1;
                }
                cycles = 5;
            },
            opcodes::DAD_B => {
                state.add_hl(state.c, state.b);
                cycles = 10;
            },
            opcodes::DAD_D => {
                state.add_hl(state.e, state.d);
                cycles = 10;
            },
            opcodes::DAD_H => {
                // not tested by cpudiag
                state.add_hl(state.l, state.h);
                cycles = 10;
            },
            opcodes::DAD_SP => {
                state.add_hl(state.sp as u8, state.sp as u8 - 1);
                cycles = 10;
            },
            opcodes::RAL => {
                // high order bit
                let hob = (state.psw.a & 0x80) >> 7;
                let carry_value = Psw::to_u8(state.psw.carry) as u8;
                state.psw.a = (state.psw.a << 1) | carry_value;
                state.psw.carry = if hob == 1 { true } else { false };
                cycles = 4;
            },
            opcodes::RAR => {
                // low order bit
                let lob = state.psw.a & 1;
                let carry_value = Psw::to_u8(state.psw.carry) as u8;
                state.psw.a = state.psw.a >> 1 | (carry_value << 7);
                state.psw.carry = if lob == 1 { true } else { false };
                cycles = 4;
            },
            opcodes::RRC => {
                // low order bit
                let lob = state.psw.a & 1;
                state.psw.carry = lob != 0;
                state.psw.a = (lob << 7) | state.psw.a >> 1;
                cycles = 4;
            },
            opcodes::RLC => {
                // high order bit
                let hob = (state.psw.a & 0x80) >> 7;
                state.psw.carry = hob != 0;
                state.psw.a = (state.psw.a << 1) | hob;
                cycles = 4;
            },
            opcodes::LDAX_B => {
                state.psw.a = self.memory.read_word(state.c, state.b);
                cycles = 7;
            },
            opcodes::LDAX_D => {
                state.psw.a = self.memory.read_word(state.e, state.d);
                cycles = 7;
            },
            opcodes::STAX_B => {
                self.memory.write_word(state.c, state.b, state.psw.a);
                cycles = 7;
            },
            opcodes::STAX_D => {
                self.memory.write_word(state.e, state.d, state.psw.a);
                cycles = 7;
            },
            opcodes::LHLD => {
                state.l = self.memory.read(word);
                state.h = self.memory.read(word + 1);
                cycles = 16;
            },
            opcodes::SHLD => {
                self.memory.write(word, state.l);
                self.memory.write(word + 1, state.h);
                cycles = 16;
            },
            opcodes::LDA => {
                state.psw.a = self.memory.read(word);
                cycles = 13;
            },
            opcodes::STA => {
                self.memory.write(word, state.psw.a);
                cycles = 13;
            },
            opcodes::MVI_A => {
                state.psw.a = byte1;
                cycles = 7;
            },
            opcodes::MVI_B => {
                state.b = byte1;
                cycles = 7;
            },
            opcodes::MVI_C => {
                state.c = byte1;
                cycles = 7;
            },
            opcodes::MVI_D => {
                state.d = byte1;
                cycles = 7;
            },
            opcodes::MVI_E => {
                state.e = byte1;
                cycles = 7;
            },
            opcodes::MVI_H => {
                state.h = byte1;
                cycles = 7;
            },
            opcodes::MVI_L => {
                state.l = byte1;
                cycles = 7;
            },
            opcodes::MVI_M => {
                self.memory.write(state.m(), byte1);
                cycles = 10;
            },
            opcodes::INR_A => {
                state.psw.a = state.inr(state.psw.a);
                cycles = 5;
            },
            opcodes::INR_B => {
                state.b = state.inr(state.b);
                cycles = 5;
            },
            opcodes::INR_C => {
                state.c = state.inr(state.c);
                cycles = 5;
            },
            opcodes::INR_D => {
                state.d = state.inr(state.d);
                cycles = 5;
            },
            opcodes::INR_E => {
                state.e = state.inr(state.e);
                cycles = 5;
            },
            opcodes::INR_H => {
                state.h = state.inr(state.h);
                cycles = 5;
            },
            opcodes::INR_L => {
                state.l = state.inr(state.l);
                cycles = 5;
            },
            opcodes::INR_M => {
                self.memory.write(state.m(), state.inr(self.memory.read(state.m())));
                cycles = 10;
            },
            opcodes::DCR_A => {
                state.psw.a = state.dec(state.psw.a);
                cycles = 5;
            },
            opcodes::DCR_B => {
                state.b = state.dec(state.b);
                cycles = 5;
            },
            opcodes::DCR_C => {
                state.c = state.dec(state.c);
                cycles = 5;
            },
            opcodes::DCR_D => {
                state.d = state.dec(state.d);
                cycles = 5;
            },
            opcodes::DCR_E => {
                state.e = state.dec(state.e);
                cycles = 5;
            },
            opcodes::DCR_H => {
                state.h = state.dec(state.h);
                cycles = 5;
            },
            opcodes::DCR_L => {
                state.l = state.dec(state.l);
                cycles = 5;
            },
            opcodes::DCR_M => {
                self.memory.write(state.m(), state.dec(self.memory.read(state.m())));
                cycles = 10;
            },
            opcodes::MOV_B_A => {
                state.b = state.psw.a;
                cycles = 5;
            },
            opcodes::MOV_B_B => {
                cycles = 5;
            },
            opcodes::MOV_B_C => {
                state.b = state.c;
                cycles = 5;
            },
            opcodes::MOV_B_D => {
                state.b = state.d;
                cycles = 5;
            },
            opcodes::MOV_B_E => {
                state.b = state.e;
                cycles = 5;
            },
            opcodes::MOV_B_H => {
                state.b = state.h;
                cycles = 5;
            },
            opcodes::MOV_B_L => {
                state.b = state.l;
                cycles = 5;
            },
            opcodes::MOV_A_M => {
                state.psw.a = self.memory.read(state.m());
                cycles = 7;
            },
            opcodes::MOV_B_M => {
                state.b = self.memory.read(state.m());
                cycles = 7;
            },
            opcodes::MOV_C_M => {
                state.c = self.memory.read(state.m());
                cycles = 7;
            },
            opcodes::MOV_D_M => {
                state.d = self.memory.read(state.m());
                cycles = 7;
            },
            opcodes::MOV_E_M => {
                state.e = self.memory.read(state.m());
                cycles = 7;
            },
            opcodes::MOV_H_M => {
                state.h = self.memory.read(state.m());
                cycles = 7;
            },
            opcodes::MOV_L_M => {
                state.l = self.memory.read(state.m());
                cycles = 7;
            },
            opcodes::MOV_C_A => {
                state.c = state.psw.a;
                cycles = 5;
            },
            opcodes::MOV_C_B => {
                state.c = state.b;
                cycles = 5;
            },
            opcodes::MOV_C_C => {
                cycles = 5;
            },
            opcodes::MOV_C_D => {
                state.c = state.d;
                cycles = 5;
            },
            opcodes::MOV_C_E => {
                state.c = state.e;
                cycles = 5;
            },
            opcodes::MOV_C_H => {
                state.c = state.h;
                cycles = 5;
            },
            opcodes::MOV_C_L => {
                state.c = state.l;
                cycles = 5;
            },
            opcodes::MOV_D_A => {
                state.d = state.psw.a;
                cycles = 5;
            },
            opcodes::MOV_D_B => {
                state.d = state.b;
                cycles = 5;
            },
            opcodes::MOV_D_C => {
                state.d = state.c;
                cycles = 5;
            },
            opcodes::MOV_D_D => {
                cycles = 5;
            },
            opcodes::MOV_D_E => {
                state.d = state.e;
                cycles = 5;
            },
            opcodes::MOV_D_H => {
                state.d = state.h;
                cycles = 5;
            },
            opcodes::MOV_D_L => {
                state.d = state.l;
                cycles = 5;
            },
            opcodes::MOV_E_A => {
                // not tested by cpudiag, was bogus
                state.e = state.psw.a;
                cycles = 5;
            },
            opcodes::MOV_E_B => {
                state.e = state.b;
                cycles = 5;
            },
            opcodes::MOV_E_C => {
                state.e = state.c;
                cycles = 5;
            },
            opcodes::MOV_E_D => {
                state.e = state.d;
                cycles = 5;
            },
            opcodes::MOV_E_E => {
                cycles = 5;
            },
            opcodes::MOV_E_H => {
                state.e = state.h;
                cycles = 5;
            },
            opcodes::MOV_E_L => {
                state.e = state.l;
                cycles = 5;
            },
            opcodes::MOV_H_A => {
                state.h = state.psw.a;
                cycles = 5;
            },
            opcodes::MOV_H_B => {
                state.h = state.b;
                cycles = 5;
            },
            opcodes::MOV_H_C => {
                state.h = state.c;
                cycles = 5;
            },
            opcodes::MOV_H_D => {
                state.h = state.d;
                cycles = 5;
            },
            opcodes::MOV_H_E => {
                state.h = state.e;
                cycles = 5;
            },
            opcodes::MOV_H_H => {
                cycles = 5;
            },
            opcodes::MOV_H_L => {
                state.h = state.l;
                cycles = 5;
            },
            opcodes::MOV_L_A => {
                state.l = state.psw.a;
                cycles = 5;
            },
            opcodes::MOV_M_B => {
                self.memory.write(state.m(), state.b);
                cycles = 7;
            },
            opcodes::MOV_M_C => {
                self.memory.write(state.m(), state.c);
                cycles = 7;
            },
            opcodes::MOV_M_D => {
                self.memory.write(state.m(), state.d);
                cycles = 7;
            },
            opcodes::MOV_M_E => {
                self.memory.write(state.m(), state.e);
                cycles = 7;
            },
            opcodes::MOV_M_H => {
                self.memory.write(state.m(), state.h);
                cycles = 7;
            },
            opcodes::MOV_M_L => {
                self.memory.write(state.m(), state.l);
                cycles = 7;
            },
            opcodes::MOV_M_A => {
                // self.memory.disassemble_instructions(state.pc, 10);
                self.memory.write(state.m(), state.psw.a);
                cycles = 7;
            },
            opcodes::MOV_L_B => {
                state.l = state.b;
                cycles = 5;
            },
            opcodes::MOV_L_C => {
                state.l = state.c;
                cycles = 5;
            },
            opcodes::MOV_L_D => {
                state.l = state.d;
                cycles = 5;
            },
            opcodes::MOV_L_E => {
                state.l = state.e;
                cycles = 5;
            },
            opcodes::MOV_L_H => {
                state.l = state.h;
                cycles = 5;
            },
            opcodes::MOV_L_L => {
                cycles = 5;
            },
            opcodes::LXI_B => {
                state.c = byte1;
                state.b = byte2;
                cycles = 10;
            },
            opcodes::LXI_D => {
                state.e = byte1;
                state.d = byte2;
                cycles = 10;
            },
            opcodes::LXI_H => {
                state.l = byte1;
                state.h = byte2;
                cycles = 10;
            },
            opcodes::MOV_A_B => {
                state.psw.a = state.b;
                cycles = 5;
            },
            opcodes::MOV_A_C => {
                state.psw.a = state.c;
                cycles = 10;
            },
            opcodes::MOV_A_D => {
                state.psw.a = state.d;
                cycles = 10;
            },
            opcodes::MOV_A_E => {
                state.psw.a = state.e;
                cycles = 10;
            },
            opcodes::MOV_A_H => {
                state.psw.a = state.h;
                cycles = 10;
            },
            opcodes::MOV_A_L => {
                state.psw.a = state.l;
                cycles = 10;
            },
            opcodes::LXI_SP => {
                state.sp = word;
                cycles = 10;
            },
            opcodes::SUI => {
                let value = state.psw.a as i16 - byte1 as i16;
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::SBI => {
                let value = state.psw.a as i16 - (byte1 as i16 + state.psw.carry as i16);
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::ADD_A => {
                state.add(state.psw.a, 0);
                cycles = 4;
            }
            opcodes::ADD_B => {
                state.add(state.b, 0);
                cycles = 4;
            }
            opcodes::ADD_C => {
                state.add(state.c, 0);
                cycles = 4;
            }
            opcodes::ADD_D => {
                state.add(state.d, 0);
                cycles = 4;
            }
            opcodes::ADD_E => {
                state.add(state.e, 0);
                cycles = 4;
            }
            opcodes::ADD_H => {
                state.add(state.h, 0);
                cycles = 4;
            }
            opcodes::ADD_L => {
                state.add(state.l, 0);
                cycles = 4;
            }
            opcodes::ADD_M => {
                state.add(self.memory.read(state.m()), 0);
                cycles = 7;
            }
            opcodes::ADC_A => {
                state.add(state.psw.a, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_B => {
                state.add(state.b, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_C => {
                state.add(state.c, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_D => {
                state.add(state.d, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_E => {
                state.add(state.e, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_H => {
                state.add(state.h, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_L => {
                state.add(state.l, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::ADC_M => {
                state.add(self.memory.read(state.m()), Psw::to_u8(state.psw.carry));
                cycles = 7;
            }
            opcodes::SUB_A => {
                state.sub(state.psw.a, 0);
                cycles = 4;
            }
            opcodes::SUB_B => {
                state.sub(state.b, 0);
                cycles = 4;
            }
            opcodes::SUB_C => {
                state.sub(state.c, 0);
                cycles = 4;
            }
            opcodes::SUB_D => {
                state.sub(state.d, 0);
                cycles = 4;
            }
            opcodes::SUB_E => {
                state.sub(state.e, 0);
                cycles = 4;
            }
            opcodes::SUB_H => {
                state.sub(state.h, 0);
                cycles = 4;
            }
            opcodes::SUB_L => {
                state.sub(state.l, 0);
                cycles = 4;
            }
            opcodes::SUB_M => {
                state.sub(self.memory.read(state.m()), 0);
                cycles = 7;
            }
            opcodes::SBB_A => {
                state.sub(state.psw.a, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_B => {
                state.sub(state.b, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_C => {
                state.sub(state.c, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_D => {
                state.sub(state.d, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_E => {
                state.sub(state.e, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_H => {
                state.sub(state.h, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_L => {
                state.sub(state.l, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            opcodes::SBB_M => {
                state.sub(self.memory.read(state.m()), Psw::to_u8(state.psw.carry));
                cycles = 7;
            }
            opcodes::ADI => {
                let value = state.psw.a as i16 + byte1 as i16;
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::ACI => {
                let value = state.psw.a as i16 + byte1 as i16 + Psw::to_u8(state.psw.carry) as i16;
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            opcodes::JMP => {
                if word == 0 {
                    let output: String = self.output_buffer.clone().into_iter().collect();
                    println!("{}", output);
                } else {
                    state.pc = word;
                    pc_was_assigned = true;
                }
                cycles = 10;
            },
            opcodes::RPO => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.parity);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RPE => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.parity);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RNC => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.carry);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RC => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.carry);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RP => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.sign);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RM => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.sign);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RZ => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.zero);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RNZ => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.zero);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            opcodes::RET => {
                pc_was_assigned = state.ret(&mut self.memory, true);
                cycles = 11;
            },
            opcodes::POP_B => {
                state.c = self.memory.read(state.sp);
                state.b = self.memory.read(state.sp + 1);
                state.sp += 2;
                cycles = 10;
            },
            opcodes::POP_D => {
                state.e = self.memory.read(state.sp);
                state.d = self.memory.read(state.sp + 1);
                state.sp += 2;
                cycles = 10;
            },
            opcodes::POP_H => {
                state.l = self.memory.read(state.sp);
                state.h = self.memory.read(state.sp + 1);
                state.sp += 2;
                cycles = 10;
            },
            opcodes::PUSH_B => {
                self.memory.write(state.sp - 1, state.b);
                self.memory.write(state.sp - 2, state.c);
                state.sp -= 2;
                cycles = 11;
            },
            opcodes::PUSH_D => {
                self.memory.write(state.sp - 1, state.d);
                self.memory.write(state.sp - 2, state.e);
                state.sp -= 2;
                cycles = 11;
            },
            opcodes::PUSH_H => {
                self.memory.write(state.sp - 1, state.h);
                self.memory.write(state.sp - 2, state.l);
                state.sp -= 2;
                cycles = 11;
            },
            opcodes::CC => {
                if state.psw.carry {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CPO => {
                if ! state.psw.parity {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CPE => {
                if state.psw.parity {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CM => {
                if state.psw.sign {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CP => {
                if ! state.psw.sign {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CNZ => {
                if ! state.psw.zero {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CZ => {
                if state.psw.zero {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::CNC => {
                if ! state.psw.carry {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            opcodes::STC => {
                state.psw.carry = true;
                cycles = 4;
            },
            opcodes::CMC => {
                state.psw.carry = ! state.psw.carry;
                cycles = 4;
            },
            opcodes::CMA => {
                state.psw.a ^= 0xff;
                cycles = 4;
            },
            opcodes::DAA => {
                let mut a: u16 = state.psw.a as u16;
                // least significant bits
                let lsb = a & 0x0f;
                if lsb > 9 || state.psw.auxiliary_carry {
                    a += 6;
                    state.psw.auxiliary_carry = (lsb + 6) > 0xf;
                };
                // most significant bits
                let mut msb = (a & 0xf0) >> 4;
                if (msb > 9) || state.psw.carry { msb += 6; }
                a = (msb << 4) | (a & 0xf);
                state.psw.auxiliary_carry = (msb + 6) > 0xf;
                state.set_arithmetic_flags(a as i16);
                state.psw.a = a as u8;

                cycles = 4;
            },
            opcodes::CALL => {
                cycles = 17;
                if cfg!(test) {
                    // In test mode, this function returns 1 for successful test, 2 for failure
                    if word == 5 && state.c == 9 {
                        // print message at address word(D, E)
                        let ind = Memory::to_word(state.e, state.d);
                        if ind == 0x174 {
                            test_status.status = StepStatus::Success("Success".into());
                        } else if ind == 0x18b {
                            let sp = state.sp + 4;
                            let pc = Memory::to_word(self.memory.read(sp), self.memory.read(sp+1))
                                - 6;
                            for i in 0..20 {
                                println!("sp + {}: {:02x}", i, self.memory.read(sp + i));
                            }
                            let s = format!("Failure at pc {:04x}", pc);
                            test_status.status = StepStatus::Failure(s);
                        } else {
                            state.call(&mut self.memory, word);
                            pc_was_assigned = true;
                        }
                    } else {
                        state.call(&mut self.memory, word);
                        pc_was_assigned = true;
                    }
                } else {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
            },
            opcodes::ANI => {
                let value = state.psw.a & byte1;
                state.psw.a = value;
                state.set_logic_flags(value.into());
                cycles = 7;
            },
            opcodes::ORI => {
                let value = state.psw.a | byte1;
                state.psw.a = value;
                state.set_logic_flags(value.into());
                cycles = 7;
            },
            opcodes::XRI => {
                state.psw.a = state.xra(byte1);
                cycles = 7;
            },
            opcodes::XRA_A => {
                state.psw.a = state.xra(state.psw.a);
                cycles = 4;
            },
            opcodes::XRA_B => {
                state.psw.a = state.xra(state.b);
                cycles = 4;
            },
            opcodes::XRA_C => {
                state.psw.a = state.xra(state.c);
                cycles = 4;
            },
            opcodes::XRA_D => {
                state.psw.a = state.xra(state.d);
                cycles = 4;
            },
            opcodes::XRA_E => {
                state.psw.a = state.xra(state.e);
                cycles = 4;
            },
            opcodes::XRA_H => {
                state.psw.a = state.xra(state.h);
                cycles = 4;
            },
            opcodes::XRA_L => {
                state.psw.a = state.xra(state.l);
                cycles = 4;
            },
            opcodes::XRA_M => {
                state.psw.a = state.xra(self.memory.read(state.m()));
                cycles = 7;
            },
            opcodes::ANA_A => {
                state.psw.a = state.and(state.psw.a);
                cycles = 4;
            },
            opcodes::ANA_B => {
                state.psw.a = state.and(state.b);
                cycles = 4;
            },
            opcodes::ANA_C => {
                state.psw.a = state.and(state.c);
                cycles = 4;
            },
            opcodes::ANA_D => {
                state.psw.a = state.and(state.d);
                cycles = 4;
            },
            opcodes::ANA_E => {
                state.psw.a = state.and(state.e);
                cycles = 4;
            },
            opcodes::ANA_H => {
                state.psw.a = state.and(state.h);
                cycles = 4;
            },
            opcodes::ANA_L => {
                state.psw.a = state.and(state.l);
                cycles = 4;
            },
            opcodes::ANA_M => {
                state.psw.a = state.and(self.memory.read(state.m()));
                cycles = 7;
            },
            opcodes::ORA_A => {
                state.psw.a = state.or(state.psw.a);
                cycles = 4;
            },
            opcodes::ORA_B => {
                state.psw.a = state.or(state.b);
                cycles = 4;
            },
            opcodes::ORA_C => {
                state.psw.a = state.or(state.c);
                cycles = 4;
            },
            opcodes::ORA_D => {
                state.psw.a = state.or(state.d);
                cycles = 4;
            },
            opcodes::ORA_E => {
                state.psw.a = state.or(state.e);
                cycles = 4;
            },
            opcodes::ORA_H => {
                state.psw.a = state.or(state.h);
                cycles = 4;
            },
            opcodes::ORA_L => {
                state.psw.a = state.or(state.l);
                cycles = 4;
            },
            opcodes::ORA_M => {
                state.psw.a = state.or(self.memory.read(state.m()));
                cycles = 7;
            },
            opcodes::XTHL => {
                let l = self.memory.read(state.sp);
                self.memory.write(state.sp, state.l);
                state.l = l;
                let h = self.memory.read(state.sp + 1);
                self.memory.write(state.sp + 1, state.h);
                state.h = h;
                cycles = 18;
            },
            opcodes::JPO => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.parity);
                cycles = 10;
            },
            opcodes::JPE => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.parity);
                cycles = 10;
            },
            opcodes::JNZ => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.zero);
                cycles = 10;
            },
            opcodes::JZ => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.zero);
                cycles = 10;
            },
            opcodes::JP => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.sign);
                cycles = 10;
            },
            opcodes::JM => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.sign);
                cycles = 10;
            },
            opcodes::JC => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.carry);
                cycles = 10;
            },
            opcodes::JNC => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.carry);
                cycles = 10;
            },
            opcodes::XCHG => {
                let h = state.h;
                state.h = state.d;
                state.d = h;
                let l = state.l;
                state.l = state.e;
                state.e = l;
                cycles = 4;
            },
            opcodes::PUSH_PSW => {
                self.memory.write(state.sp - 1, state.psw.a);
                self.memory.write(state.sp - 2, (state.psw.value() & 0xff) as u8);
                state.sp -= 2;
                cycles = 11;
            },
            opcodes::POP_PSW => {
                state.psw.a = self.memory.read(state.sp + 1);
                state.psw.set_flags(self.memory.read(state.sp));
                state.sp += 2;
                cycles = 10;
            },
            opcodes::CPI => {
                state.cmp(byte1);
                cycles = 7;
            },
            opcodes::CMP_B => {
                state.cmp(state.b);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_C => {
                state.cmp(state.c);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_D => {
                state.cmp(state.d);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_E => {
                state.cmp(state.e);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_H => {
                state.cmp(state.h);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_L => {
                state.cmp(state.l);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::CMP_M => {
                state.cmp(self.memory.read(state.m()));
                cycles = 7;
            },
            opcodes::CMP_A => {
                state.cmp(state.psw.a);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            opcodes::SPHL => {
                state.sp = ((state.h as u16) << 8) as usize | state.l as usize;
                cycles = 5;
            },
            opcodes::PCHL => {
                state.pc = ((state.h as u16) << 8) as usize | state.l as usize;
                pc_was_assigned = true;
                cycles = 5;
            },
            opcodes::EI => {
                state.enable_interrupts = true;
                cycles = 4;
            }
            opcodes::DI => {
                state.enable_interrupts = false;
                cycles = 4;
            }
            opcodes::OUT => {
                match byte1 {
                    2 => {
                        self.shift_register_offset = state.psw.a & 0x7;
                    },
                    3 => {
                        // sound
                    },
                    4 => {
                        self.shift_register = ((state.psw.a as u16) << 8)
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
                        state.psw.a = SHARED_STATE.get().unwrap().lock().unwrap().get_in_1();
                    },
                    2 => {
                        state.psw.a = SHARED_STATE.get().unwrap().lock().unwrap().get_in_2();
                    },
                    3 => {
                        let shift_amount = 8 - self.shift_register_offset;
                        state.psw.a = (self.shift_register >> shift_amount) as u8;
                    },
                    _ => {
                        panic!("Unsupported IN port: {}", byte1);
                    }
                }
                cycles = 10;
            }
            opcodes::RST_1 => {
                state.call(&mut self.memory, 1 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            opcodes::RST_2 => {
                state.call(&mut self.memory, 2 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            opcodes::RST_7 => {
                state.call(&mut self.memory, 7 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            _ => panic!("Don't know how to run opcode: {:02x} at {:04x}", op, state.pc),
        }

        if ! pc_was_assigned {
            state.pc += opcode.size;
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
        if self.state.as_ref().unwrap().enable_interrupts {
            // log_time(format!("Interrupt {}", interrupt_number).as_str());
            let state = self.state.as_mut().unwrap();
            self.memory.write(state.sp - 1, ((state.pc as u16 & 0xff00) >> 8) as u8);
            self.memory.write(state.sp - 2, (state.pc as u16 & 0xff) as u8);
            state.sp -= 2;
            // Interrupt 0 goes to $0, 1 to $08, 2 to $10, etc...
            state.pc = (interrupt_number as usize) << 3;
        }
    }
}
