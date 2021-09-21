use crate::memory::Memory;

#[derive(Default)]
pub struct Psw {
    pub a: u8,
    pub sign: bool,
    pub zero: bool,
    pub auxiliary_carry: bool,
    pub parity: bool,
    pub carry: bool,
}

/*
 * PSW is: A for the high byte and the flags for the low byte, in order,
 * sign, zero, auxiliary carry, parity, carry:
 *
 * Bit:  7 6 5 4  3 2 1 0
 * Flag: S Z 0 AC 0 P 1 C
 */
impl Psw {
    pub fn to_u8(f: bool) -> u16 { if f { 1 } else { 0 } }
    pub fn to_bool(v: u8) -> bool { if v == 0 { false } else { true }}

    pub fn set_flags(&mut self, value: u8) {
        self.sign = Psw::to_bool(value & (1 << 7));
        self.zero = Psw::to_bool(value & (1 << 6));
        self.auxiliary_carry = Psw::to_bool(value & (1 << 4));
        self.parity = Psw::to_bool(value & (1 << 2));
        self.carry = Psw::to_bool(value & 1);
    }

    pub fn value(&self) -> u16 {
        (self.a as u16) << 8 |
            Psw::to_u8(self.sign) << 7 |
            Psw::to_u8(self.zero) << 6 |
            Psw::to_u8(self.auxiliary_carry) << 4 |
            Psw::to_u8(self.parity) << 2 |
            1 << 1 |
            Psw::to_u8(self.carry)
    }

    pub(crate) fn disassemble(&self) -> String {
        format!("[C={} P={} S={} Z={}]",
                Psw::to_u8(self.carry),
                Psw::to_u8(self.parity),
                Psw::to_u8(self.sign),
                Psw::to_u8(self.zero))
    }
}

#[derive(Default)]
pub struct Cpu {
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub psw: Psw,
    pub pc: usize,
    pub sp: usize,
    pub enable_interrupts: bool,
}

impl Cpu {
    pub(crate) fn new(pc: usize) -> Cpu {
        Cpu {
            pc,
            .. Default::default()
        }
    }

    pub fn m(&self) -> usize {
        Memory::to_word(self.l, self.h)
    }

    pub fn disassemble(&self) -> String {
        format!("a:{:02x} b:{:02x} c:{:02x} d:{:02x} e:{:02x} lh:{:04x} pc:{:04x} sp:{:04x} {}",
                self.psw.a, self.b, self.c, self.d, self.e, ((self.h as u16) << 8) | self.l as u16,
                self.pc, self.sp,
                self.psw.disassemble())
    }

    pub fn set_logic_flags(&mut self, value: i16) {
        self.psw.zero = value == 0;
        self.psw.sign = 0x80 == (value & 0x80);
        self.psw.parity = (value & 0xff).count_ones() % 2 == 0;
        self.psw.carry = false;
        // state.psw.auxiliary_carry = (state.psw.a < byte1);
    }

    pub fn set_arithmetic_flags(&mut self, value: i16) {
        self.psw.zero = (value & 0xff) == 0;
        self.psw.sign = 0x80 == (value & 0x80);
        self.psw.parity = (value & 0xff).count_ones() % 2 == 0;
        self.psw.carry = value < 0 || value > 0xff;
        self.psw.auxiliary_carry = self.psw.carry;
    }

    pub fn jump_if_flag(&mut self, word: usize, flag: bool) -> bool {
        if flag {
            self.pc = word;
            true
        } else {
            false
        }
    }

    pub fn call(&mut self, target_pc: usize) {
        let ret = self.pc + 3;
        Memory::write(self.sp - 1,((ret >> 8) as u8) & 0xff);
        Memory::write(self.sp - 2, (ret & 0xff) as u8);
        self.sp -= 2;
        self.pc = target_pc;
    }

    pub fn ret(&mut self, flag: bool) -> bool {
        if flag {
            self.pc = Memory::to_word(Memory::read(self.sp), Memory::read(self.sp + 1));
            self.sp += 2;
        }
        flag
    }

    pub fn dec(&mut self, n: u8) -> u8 {
        let value = if n == 0 {
            self.psw.carry = true;
            self.psw.auxiliary_carry = true;
            0xff
        } else {
            n - 1
        };
        self.set_arithmetic_flags(value as i16);
        value
    }

    pub fn xra(&mut self, value: u8) -> u8 {
        let value = self.psw.a ^ value;
        self.set_arithmetic_flags(value as i16);
        value
    }

    pub fn and(&mut self, value: u8) -> u8 {
        let value = self.psw.a & value;
        self.set_arithmetic_flags(value as i16);
        value
    }

    pub fn or(&mut self, value: u8) -> u8 {
        let value = self.psw.a | value;
        self.set_arithmetic_flags(value as i16);
        value
    }

    pub fn add(&mut self, value: u8, carry: u16) {
        let value = self.psw.a as u16 + value as u16 + carry;
        self.set_arithmetic_flags(value as i16);
        self.psw.a = value as u8 & 0xff;
    }

    pub fn sub(&mut self, value: u8, carry: u16) {
        let value = self.psw.a as i16 - value as i16 - carry as i16;
        self.set_arithmetic_flags(value as i16);
        self.psw.a = value as u8 & 0xff;
    }

    pub fn cmp(&mut self, n: u8) {
        let value: i16 = self.psw.a as i16 - n as i16;
        self.set_arithmetic_flags(value);
    }

    pub fn inr(&mut self, n: u8) -> u8 {
        let value = if n == 0xff { 0 } else { n + 1 };
        self.set_arithmetic_flags(value as i16);
        value
    }

    pub fn add_hl(&mut self, b0: u8, b1: u8) {
        let hl = Memory::to_word(self.l, self.h) as u32;
        let v = Memory::to_word(b0, b1) as u32;
        let result: u32 = hl + v;
        self.psw.carry = result > 0xffff;
        self.h = ((result & 0xff00) >> 8) as u8;
        self.l = (result & 0xff) as u8;
    }
}
