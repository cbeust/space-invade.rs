use crate::{opcodes, VERBOSE_DISASSEMBLE, VERBOSE_DISASSEMBLE_SECTION, DISASSEMBLE_SECTION_START, DISASSEMBLE_SECTION_END};
use crate::memory::Memory;
use crate::state::*;
use crate::emulator_state::SharedState;
use std::thread;
use wasm_timer::SystemTime;
use std::time::{Duration};
use std::sync::Mutex;
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
            thread::sleep(Duration::from_millis((time_per_frame_ms - elapsed) as u64));
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
}

pub fn spawn_emulator_thread() {
    //
    // Spawn the game logic in a separate thread. This logic will communicate with the
    // main thread (and therefore, the actual graphics on your screen) via the `listener`
    // object that this function receives in parameter.
    //
    thread::spawn(move || {
        spawn_emulator();
    });
}

#[wasm_bindgen]
extern {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


// #[cfg(target_arch = "wasm32")]
// pub fn log(s: &str) {
//     javascript_log(s);
// }

#[cfg(not(target_arch = "wasm32"))]
pub fn log(s: &str) {
    println!("{}", s);
}

impl Emulator {

    pub fn new_space_invaders() -> Emulator {
        let mut memory = Memory::new();
        #[cfg(not(target_arch = "wasm32"))]
        memory.read_file("space-invaders.rom", 0);

        #[cfg(target_arch = "wasm32")]
        log("Warning: need to read the rom file in WASM mode");

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
        let _ = SHARED_STATE.set(Mutex::new(SharedState::new()));
        spawn_emulator_thread();
        &SHARED_STATE.get().unwrap()
    }

    pub fn run_one_frame(&mut self, verbose: bool) -> u64 {
        let mut total_cycles: u64 = 0;
        let cycle_max = 2_000_000 / 60;
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

    pub fn step(&mut self, _verbose: bool) -> StepResult {
        let shared = SHARED_STATE.get().unwrap();
        if shared.lock().unwrap().is_paused() {
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

        use opcodes::*;
        match op {
            NOP => {
                cycles = 4;
            },
            INX_B => {
                state.c = ((state.c as u16) + 1) as u8;
                if state.c == 0 {
                    state.b = ((state.b as u16) + 1) as u8;
                }
                cycles = 5;
            },
            INX_D => {
                state.e = ((state.e as u16) + 1) as u8;
                if state.e == 0 {
                    state.d = ((state.d as u16) + 1) as u8;
                }
                cycles = 5;
            },
            INX_SP => {
                if state.sp == 0xffff {
                    state.sp = 0;
                } else {
                    state.sp += 1;
                }
                cycles = 5;
            },
            INX_H => {
                state.l = ((state.l as u16) + 1) as u8;
                if state.l == 0 {
                    state.h = ((state.h as u16) + 1) as u8;
                }
                cycles = 5;
            },
            DCX_B => {
                state.c = ((state.c as i8) - 1) as u8;
                if state.c == 0xff {
                    state.b = ((state.b as i8) - 1) as u8;
                }
                cycles = 5;
            },
            DCX_D => {
                state.e = ((state.e as i8) - 1) as u8;
                if state.e == 0xff {
                    state.d = ((state.d as i8) - 1) as u8;
                }
                cycles = 5;
            },
            DCX_H => {
                let m = state.m() as i32 - 1;
                state.h = (m >> 8) as u8;
                state.l = m as u8;
                cycles = 5;
            },
            DCX_SP => {
                if state.sp == 0 {
                    state.sp = 0xffff as usize;
                } else {
                    state.sp -= 1;
                }
                cycles = 5;
            },
            DAD_B => {
                state.add_hl(state.c, state.b);
                cycles = 10;
            },
            DAD_D => {
                state.add_hl(state.e, state.d);
                cycles = 10;
            },
            DAD_H => {
                // not tested by cpudiag
                state.add_hl(state.l, state.h);
                cycles = 10;
            },
            DAD_SP => {
                state.add_hl(state.sp as u8, state.sp as u8 - 1);
                cycles = 10;
            },
            RAL => {
                // high order bit
                let hob = (state.psw.a & 0x80) >> 7;
                let carry_value = Psw::to_u8(state.psw.carry) as u8;
                state.psw.a = (state.psw.a << 1) | carry_value;
                state.psw.carry = if hob == 1 { true } else { false };
                cycles = 4;
            },
            RAR => {
                // low order bit
                let lob = state.psw.a & 1;
                let carry_value = Psw::to_u8(state.psw.carry) as u8;
                state.psw.a = state.psw.a >> 1 | (carry_value << 7);
                state.psw.carry = if lob == 1 { true } else { false };
                cycles = 4;
            },
            RRC => {
                // low order bit
                let lob = state.psw.a & 1;
                state.psw.carry = lob != 0;
                state.psw.a = (lob << 7) | state.psw.a >> 1;
                cycles = 4;
            },
            RLC => {
                // high order bit
                let hob = (state.psw.a & 0x80) >> 7;
                state.psw.carry = hob != 0;
                state.psw.a = (state.psw.a << 1) | hob;
                cycles = 4;
            },
            LDAX_B => {
                state.psw.a = self.memory.read_word(state.c, state.b);
                cycles = 7;
            },
            LDAX_D => {
                state.psw.a = self.memory.read_word(state.e, state.d);
                cycles = 7;
            },
            STAX_B => {
                self.memory.write_word(state.c, state.b, state.psw.a);
                cycles = 7;
            },
            STAX_D => {
                self.memory.write_word(state.e, state.d, state.psw.a);
                cycles = 7;
            },
            LHLD => {
                state.l = self.memory.read(word);
                state.h = self.memory.read(word + 1);
                cycles = 16;
            },
            SHLD => {
                self.memory.write(word, state.l);
                self.memory.write(word + 1, state.h);
                cycles = 16;
            },
            LDA => {
                state.psw.a = self.memory.read(word);
                cycles = 13;
            },
            STA => {
                self.memory.write(word, state.psw.a);
                cycles = 13;
            },
            MVI_A => {
                state.psw.a = byte1;
                cycles = 7;
            },
            MVI_B => {
                state.b = byte1;
                cycles = 7;
            },
            MVI_C => {
                state.c = byte1;
                cycles = 7;
            },
            MVI_D => {
                state.d = byte1;
                cycles = 7;
            },
            MVI_E => {
                state.e = byte1;
                cycles = 7;
            },
            MVI_H => {
                state.h = byte1;
                cycles = 7;
            },
            MVI_L => {
                state.l = byte1;
                cycles = 7;
            },
            MVI_M => {
                self.memory.write(state.m(), byte1);
                cycles = 10;
            },
            INR_A => {
                state.psw.a = state.inr(state.psw.a);
                cycles = 5;
            },
            INR_B => {
                state.b = state.inr(state.b);
                cycles = 5;
            },
            INR_C => {
                state.c = state.inr(state.c);
                cycles = 5;
            },
            INR_D => {
                state.d = state.inr(state.d);
                cycles = 5;
            },
            INR_E => {
                state.e = state.inr(state.e);
                cycles = 5;
            },
            INR_H => {
                state.h = state.inr(state.h);
                cycles = 5;
            },
            INR_L => {
                state.l = state.inr(state.l);
                cycles = 5;
            },
            INR_M => {
                self.memory.write(state.m(), state.inr(self.memory.read(state.m())));
                cycles = 10;
            },
            DCR_A => {
                state.psw.a = state.dec(state.psw.a);
                cycles = 5;
            },
            DCR_B => {
                state.b = state.dec(state.b);
                cycles = 5;
            },
            DCR_C => {
                state.c = state.dec(state.c);
                cycles = 5;
            },
            DCR_D => {
                state.d = state.dec(state.d);
                cycles = 5;
            },
            DCR_E => {
                state.e = state.dec(state.e);
                cycles = 5;
            },
            DCR_H => {
                state.h = state.dec(state.h);
                cycles = 5;
            },
            DCR_L => {
                state.l = state.dec(state.l);
                cycles = 5;
            },
            DCR_M => {
                self.memory.write(state.m(), state.dec(self.memory.read(state.m())));
                cycles = 10;
            },
            MOV_B_A => {
                state.b = state.psw.a;
                cycles = 5;
            },
            MOV_B_B => {
                cycles = 5;
            },
            MOV_B_C => {
                state.b = state.c;
                cycles = 5;
            },
            MOV_B_D => {
                state.b = state.d;
                cycles = 5;
            },
            MOV_B_E => {
                state.b = state.e;
                cycles = 5;
            },
            MOV_B_H => {
                state.b = state.h;
                cycles = 5;
            },
            MOV_B_L => {
                state.b = state.l;
                cycles = 5;
            },
            MOV_A_M => {
                state.psw.a = self.memory.read(state.m());
                cycles = 7;
            },
            MOV_B_M => {
                state.b = self.memory.read(state.m());
                cycles = 7;
            },
            MOV_C_M => {
                state.c = self.memory.read(state.m());
                cycles = 7;
            },
            MOV_D_M => {
                state.d = self.memory.read(state.m());
                cycles = 7;
            },
            MOV_E_M => {
                state.e = self.memory.read(state.m());
                cycles = 7;
            },
            MOV_H_M => {
                state.h = self.memory.read(state.m());
                cycles = 7;
            },
            MOV_L_M => {
                state.l = self.memory.read(state.m());
                cycles = 7;
            },
            MOV_C_A => {
                state.c = state.psw.a;
                cycles = 5;
            },
            MOV_C_B => {
                state.c = state.b;
                cycles = 5;
            },
            MOV_C_C => {
                cycles = 5;
            },
            MOV_C_D => {
                state.c = state.d;
                cycles = 5;
            },
            MOV_C_E => {
                state.c = state.e;
                cycles = 5;
            },
            MOV_C_H => {
                state.c = state.h;
                cycles = 5;
            },
            MOV_C_L => {
                state.c = state.l;
                cycles = 5;
            },
            MOV_D_A => {
                state.d = state.psw.a;
                cycles = 5;
            },
            MOV_D_B => {
                state.d = state.b;
                cycles = 5;
            },
            MOV_D_C => {
                state.d = state.c;
                cycles = 5;
            },
            MOV_D_D => {
                cycles = 5;
            },
            MOV_D_E => {
                state.d = state.e;
                cycles = 5;
            },
            MOV_D_H => {
                state.d = state.h;
                cycles = 5;
            },
            MOV_D_L => {
                state.d = state.l;
                cycles = 5;
            },
            MOV_E_A => {
                // not tested by cpudiag, was bogus
                state.e = state.psw.a;
                cycles = 5;
            },
            MOV_E_B => {
                state.e = state.b;
                cycles = 5;
            },
            MOV_E_C => {
                state.e = state.c;
                cycles = 5;
            },
            MOV_E_D => {
                state.e = state.d;
                cycles = 5;
            },
            MOV_E_E => {
                cycles = 5;
            },
            MOV_E_H => {
                state.e = state.h;
                cycles = 5;
            },
            MOV_E_L => {
                state.e = state.l;
                cycles = 5;
            },
            MOV_H_A => {
                state.h = state.psw.a;
                cycles = 5;
            },
            MOV_H_B => {
                state.h = state.b;
                cycles = 5;
            },
            MOV_H_C => {
                state.h = state.c;
                cycles = 5;
            },
            MOV_H_D => {
                state.h = state.d;
                cycles = 5;
            },
            MOV_H_E => {
                state.h = state.e;
                cycles = 5;
            },
            MOV_H_H => {
                cycles = 5;
            },
            MOV_H_L => {
                state.h = state.l;
                cycles = 5;
            },
            MOV_L_A => {
                state.l = state.psw.a;
                cycles = 5;
            },
            MOV_M_B => {
                self.memory.write(state.m(), state.b);
                cycles = 7;
            },
            MOV_M_C => {
                self.memory.write(state.m(), state.c);
                cycles = 7;
            },
            MOV_M_D => {
                self.memory.write(state.m(), state.d);
                cycles = 7;
            },
            MOV_M_E => {
                self.memory.write(state.m(), state.e);
                cycles = 7;
            },
            MOV_M_H => {
                self.memory.write(state.m(), state.h);
                cycles = 7;
            },
            MOV_M_L => {
                self.memory.write(state.m(), state.l);
                cycles = 7;
            },
            MOV_M_A => {
                // self.memory.disassemble_instructions(state.pc, 10);
                self.memory.write(state.m(), state.psw.a);
                cycles = 7;
            },
            MOV_L_B => {
                state.l = state.b;
                cycles = 5;
            },
            MOV_L_C => {
                state.l = state.c;
                cycles = 5;
            },
            MOV_L_D => {
                state.l = state.d;
                cycles = 5;
            },
            MOV_L_E => {
                state.l = state.e;
                cycles = 5;
            },
            MOV_L_H => {
                state.l = state.h;
                cycles = 5;
            },
            MOV_L_L => {
                cycles = 5;
            },
            LXI_B => {
                state.c = byte1;
                state.b = byte2;
                cycles = 10;
            },
            LXI_D => {
                state.e = byte1;
                state.d = byte2;
                cycles = 10;
            },
            LXI_H => {
                state.l = byte1;
                state.h = byte2;
                cycles = 10;
            },
            MOV_A_B => {
                state.psw.a = state.b;
                cycles = 5;
            },
            MOV_A_C => {
                state.psw.a = state.c;
                cycles = 10;
            },
            MOV_A_D => {
                state.psw.a = state.d;
                cycles = 10;
            },
            MOV_A_E => {
                state.psw.a = state.e;
                cycles = 10;
            },
            MOV_A_H => {
                state.psw.a = state.h;
                cycles = 10;
            },
            MOV_A_L => {
                state.psw.a = state.l;
                cycles = 10;
            },
            LXI_SP => {
                state.sp = word;
                cycles = 10;
            },
            SUI => {
                let value = state.psw.a as i16 - byte1 as i16;
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            SBI => {
                let value = state.psw.a as i16 - (byte1 as i16 + state.psw.carry as i16);
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            ADD_A => {
                state.add(state.psw.a, 0);
                cycles = 4;
            }
            ADD_B => {
                state.add(state.b, 0);
                cycles = 4;
            }
            ADD_C => {
                state.add(state.c, 0);
                cycles = 4;
            }
            ADD_D => {
                state.add(state.d, 0);
                cycles = 4;
            }
            ADD_E => {
                state.add(state.e, 0);
                cycles = 4;
            }
            ADD_H => {
                state.add(state.h, 0);
                cycles = 4;
            }
            ADD_L => {
                state.add(state.l, 0);
                cycles = 4;
            }
            ADD_M => {
                state.add(self.memory.read(state.m()), 0);
                cycles = 7;
            }
            ADC_A => {
                state.add(state.psw.a, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            ADC_B => {
                state.add(state.b, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            ADC_C => {
                state.add(state.c, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            ADC_D => {
                state.add(state.d, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            ADC_E => {
                state.add(state.e, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            ADC_H => {
                state.add(state.h, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            ADC_L => {
                state.add(state.l, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            ADC_M => {
                state.add(self.memory.read(state.m()), Psw::to_u8(state.psw.carry));
                cycles = 7;
            }
            SUB_A => {
                state.sub(state.psw.a, 0);
                cycles = 4;
            }
            SUB_B => {
                state.sub(state.b, 0);
                cycles = 4;
            }
            SUB_C => {
                state.sub(state.c, 0);
                cycles = 4;
            }
            SUB_D => {
                state.sub(state.d, 0);
                cycles = 4;
            }
            SUB_E => {
                state.sub(state.e, 0);
                cycles = 4;
            }
            SUB_H => {
                state.sub(state.h, 0);
                cycles = 4;
            }
            SUB_L => {
                state.sub(state.l, 0);
                cycles = 4;
            }
            SUB_M => {
                state.sub(self.memory.read(state.m()), 0);
                cycles = 7;
            }
            SBB_A => {
                state.sub(state.psw.a, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            SBB_B => {
                state.sub(state.b, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            SBB_C => {
                state.sub(state.c, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            SBB_D => {
                state.sub(state.d, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            SBB_E => {
                state.sub(state.e, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            SBB_H => {
                state.sub(state.h, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            SBB_L => {
                state.sub(state.l, Psw::to_u8(state.psw.carry));
                cycles = 4;
            }
            SBB_M => {
                state.sub(self.memory.read(state.m()), Psw::to_u8(state.psw.carry));
                cycles = 7;
            }
            ADI => {
                let value = state.psw.a as i16 + byte1 as i16;
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            ACI => {
                let value = state.psw.a as i16 + byte1 as i16 + Psw::to_u8(state.psw.carry) as i16;
                state.psw.a = value as u8;
                state.set_arithmetic_flags(value);
                cycles = 7;
            },
            JMP => {
                if word == 0 {
                    let output: String = self.output_buffer.clone().into_iter().collect();
                    println!("{}", output);
                } else {
                    state.pc = word;
                    pc_was_assigned = true;
                }
                cycles = 10;
            },
            RPO => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.parity);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RPE => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.parity);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RNC => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.carry);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RC => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.carry);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RP => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.sign);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RM => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.sign);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RZ => {
                pc_was_assigned = state.ret(&mut self.memory, state.psw.zero);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RNZ => {
                pc_was_assigned = state.ret(&mut self.memory, ! state.psw.zero);
                cycles = if pc_was_assigned { 11 } else { 5 };
            },
            RET => {
                pc_was_assigned = state.ret(&mut self.memory, true);
                cycles = 11;
            },
            POP_B => {
                state.c = self.memory.read(state.sp);
                state.b = self.memory.read(state.sp + 1);
                state.sp += 2;
                cycles = 10;
            },
            POP_D => {
                state.e = self.memory.read(state.sp);
                state.d = self.memory.read(state.sp + 1);
                state.sp += 2;
                cycles = 10;
            },
            POP_H => {
                state.l = self.memory.read(state.sp);
                state.h = self.memory.read(state.sp + 1);
                state.sp += 2;
                cycles = 10;
            },
            PUSH_B => {
                self.memory.write(state.sp - 1, state.b);
                self.memory.write(state.sp - 2, state.c);
                state.sp -= 2;
                cycles = 11;
            },
            PUSH_D => {
                self.memory.write(state.sp - 1, state.d);
                self.memory.write(state.sp - 2, state.e);
                state.sp -= 2;
                cycles = 11;
            },
            PUSH_H => {
                self.memory.write(state.sp - 1, state.h);
                self.memory.write(state.sp - 2, state.l);
                state.sp -= 2;
                cycles = 11;
            },
            CC => {
                if state.psw.carry {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            CPO => {
                if ! state.psw.parity {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            CPE => {
                if state.psw.parity {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            CM => {
                if state.psw.sign {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            CP => {
                if ! state.psw.sign {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            CNZ => {
                if ! state.psw.zero {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            CZ => {
                if state.psw.zero {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            CNC => {
                if ! state.psw.carry {
                    state.call(&mut self.memory, word);
                    pc_was_assigned = true;
                }
                cycles = if pc_was_assigned { 11 } else { 17 };
            },
            STC => {
                state.psw.carry = true;
                cycles = 4;
            },
            CMC => {
                state.psw.carry = ! state.psw.carry;
                cycles = 4;
            },
            CMA => {
                state.psw.a ^= 0xff;
                cycles = 4;
            },
            DAA => {
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
            CALL => {
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
            ANI => {
                let value = state.psw.a & byte1;
                state.psw.a = value;
                state.set_logic_flags(value.into());
                cycles = 7;
            },
            ORI => {
                let value = state.psw.a | byte1;
                state.psw.a = value;
                state.set_logic_flags(value.into());
                cycles = 7;
            },
            XRI => {
                state.psw.a = state.xra(byte1);
                cycles = 7;
            },
            XRA_A => {
                state.psw.a = state.xra(state.psw.a);
                cycles = 4;
            },
            XRA_B => {
                state.psw.a = state.xra(state.b);
                cycles = 4;
            },
            XRA_C => {
                state.psw.a = state.xra(state.c);
                cycles = 4;
            },
            XRA_D => {
                state.psw.a = state.xra(state.d);
                cycles = 4;
            },
            XRA_E => {
                state.psw.a = state.xra(state.e);
                cycles = 4;
            },
            XRA_H => {
                state.psw.a = state.xra(state.h);
                cycles = 4;
            },
            XRA_L => {
                state.psw.a = state.xra(state.l);
                cycles = 4;
            },
            XRA_M => {
                state.psw.a = state.xra(self.memory.read(state.m()));
                cycles = 7;
            },
            ANA_A => {
                state.psw.a = state.and(state.psw.a);
                cycles = 4;
            },
            ANA_B => {
                state.psw.a = state.and(state.b);
                cycles = 4;
            },
            ANA_C => {
                state.psw.a = state.and(state.c);
                cycles = 4;
            },
            ANA_D => {
                state.psw.a = state.and(state.d);
                cycles = 4;
            },
            ANA_E => {
                state.psw.a = state.and(state.e);
                cycles = 4;
            },
            ANA_H => {
                state.psw.a = state.and(state.h);
                cycles = 4;
            },
            ANA_L => {
                state.psw.a = state.and(state.l);
                cycles = 4;
            },
            ANA_M => {
                state.psw.a = state.and(self.memory.read(state.m()));
                cycles = 7;
            },
            ORA_A => {
                state.psw.a = state.or(state.psw.a);
                cycles = 4;
            },
            ORA_B => {
                state.psw.a = state.or(state.b);
                cycles = 4;
            },
            ORA_C => {
                state.psw.a = state.or(state.c);
                cycles = 4;
            },
            ORA_D => {
                state.psw.a = state.or(state.d);
                cycles = 4;
            },
            ORA_E => {
                state.psw.a = state.or(state.e);
                cycles = 4;
            },
            ORA_H => {
                state.psw.a = state.or(state.h);
                cycles = 4;
            },
            ORA_L => {
                state.psw.a = state.or(state.l);
                cycles = 4;
            },
            ORA_M => {
                state.psw.a = state.or(self.memory.read(state.m()));
                cycles = 7;
            },
            XTHL => {
                let l = self.memory.read(state.sp);
                self.memory.write(state.sp, state.l);
                state.l = l;
                let h = self.memory.read(state.sp + 1);
                self.memory.write(state.sp + 1, state.h);
                state.h = h;
                cycles = 18;
            },
            JPO => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.parity);
                cycles = 10;
            },
            JPE => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.parity);
                cycles = 10;
            },
            JNZ => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.zero);
                cycles = 10;
            },
            JZ => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.zero);
                cycles = 10;
            },
            JP => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.sign);
                cycles = 10;
            },
            JM => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.sign);
                cycles = 10;
            },
            JC => {
                pc_was_assigned = state.jump_if_flag(word, state.psw.carry);
                cycles = 10;
            },
            JNC => {
                pc_was_assigned = state.jump_if_flag(word, ! state.psw.carry);
                cycles = 10;
            },
            XCHG => {
                let h = state.h;
                state.h = state.d;
                state.d = h;
                let l = state.l;
                state.l = state.e;
                state.e = l;
                cycles = 4;
            },
            PUSH_PSW => {
                self.memory.write(state.sp - 1, state.psw.a);
                self.memory.write(state.sp - 2, (state.psw.value() & 0xff) as u8);
                state.sp -= 2;
                cycles = 11;
            },
            POP_PSW => {
                state.psw.a = self.memory.read(state.sp + 1);
                state.psw.set_flags(self.memory.read(state.sp));
                state.sp += 2;
                cycles = 10;
            },
            CPI => {
                state.cmp(byte1);
                cycles = 7;
            },
            CMP_B => {
                state.cmp(state.b);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            CMP_C => {
                state.cmp(state.c);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            CMP_D => {
                state.cmp(state.d);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            CMP_E => {
                state.cmp(state.e);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            CMP_H => {
                state.cmp(state.h);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            CMP_L => {
                state.cmp(state.l);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            CMP_M => {
                state.cmp(self.memory.read(state.m()));
                cycles = 7;
            },
            CMP_A => {
                state.cmp(state.psw.a);
                cycles = 4;  // not sure, couldn't find it in the reference
            },
            SPHL => {
                state.sp = ((state.h as u16) << 8) as usize | state.l as usize;
                cycles = 5;
            },
            PCHL => {
                state.pc = ((state.h as u16) << 8) as usize | state.l as usize;
                pc_was_assigned = true;
                cycles = 5;
            },
            EI => {
                state.enable_interrupts = true;
                cycles = 4;
            }
            DI => {
                state.enable_interrupts = false;
                cycles = 4;
            }
            OUT => {
                match byte1 {
                    2 => {
                        self.shift_register_offset = state.psw.a & 0x7;
                    },
                    3 => {
                        // Port 3: (discrete sounds)
                        //  bit 0=UFO (repeats)        SX0 0.raw
                        //  bit 1=Shot                 SX1 1.raw
                        //  bit 2=Flash (player die)   SX2 2.raw
                        //  bit 3=Invader die          SX3 3.raw
                        //  bit 4=Extended play        SX4
                        //  bit 5= AMP enable          SX5
                        println!("OUT 5 CALLED, SOUND: {byte2}");
                    },
                    4 => {
                        self.shift_register = ((state.psw.a as u16) << 8)
                            | (self.shift_register >> 8)
                    },
                    5 => {
                        // Port 5:
                        //  bit 0=Fleet movement 1     SX6 4.raw
                        //  bit 1=Fleet movement 2     SX7 5.raw
                        //  bit 2=Fleet movement 3     SX8 6.raw
                        //  bit 3=Fleet movement 4     SX9 7.raw
                        //  bit 4=UFO Hit              SX10 8.raw
                        println!("OUT 5 CALLED, SOUND: {byte2}");
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
            IN => {
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
            RST_1 => {
                state.call(&mut self.memory, 1 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            RST_2 => {
                state.call(&mut self.memory, 2 * 8);
                pc_was_assigned = true;
                cycles = 10;
            }
            RST_7 => {
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
            self.state.as_mut().unwrap().enable_interrupts = false;
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
