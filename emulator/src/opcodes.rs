use std::collections::HashMap;
use crate::memory::Memory;
use lazy_static::lazy_static;

pub struct Opcode {
    opcode: u8,
    pub size: usize,
    name: &'static str
}

impl Opcode {
    fn new(opcode: u8, size: usize, name: &'static str) -> Opcode {
        Opcode { opcode, size, name }
    }

    pub(crate) fn display1(&self) -> String {
        let s = self.name;
        format!("{:02x}         {:14}", self.opcode, s)
    }

    pub(crate) fn display2(&self, byte1: u8) -> String {
        let s = format!("{} {:02x}", self.name, byte1);
        format!("{:02x} {:02x}      {:14}", self.opcode, byte1, s)
    }

    pub(crate) fn display3(&self, byte1: u8, byte2: u8) -> String {
        let s = format!("{} {:04x}", self.name, Memory::to_word(byte1, byte2));
        format!("{:02x} {:02x} {:02x}   {:14}", self.opcode, byte1, byte2, s)
    }
}   

pub const NOP: u8 = 0x00;
pub const LXI_B: u8 = 0x01;
pub const STAX_B: u8 = 0x02;
pub const INX_B: u8 = 0x03;
pub const INR_B: u8 = 0x04;
pub const DCR_B: u8 = 0x05;
pub const MVI_B: u8 = 0x06;
pub const RLC: u8 = 0x07;
pub const DAD_B: u8 = 0x09;
pub const LDAX_B: u8 = 0x0a;
pub const DCX_B: u8 = 0x0b;
pub const RRC: u8 = 0x0f;
pub const LXI_D: u8 = 0x11;
pub const STAX_D: u8 = 0x12;
pub const INX_D: u8 = 0x13;
pub const INR_C: u8 = 0x0c;
pub const DCR_C: u8 = 0x0d;
pub const MVI_C: u8 = 0x0e;
pub const INR_D: u8 = 0x14;
pub const DCR_D: u8 = 0x15;
pub const MVI_D: u8 = 0x16;
pub const RAL: u8 = 0x17;
pub const DAD_D: u8 = 0x19;
pub const LDAX_D: u8 = 0x1a;
pub const DCX_D: u8 = 0x1b;
pub const INR_E: u8 = 0x1c;
pub const DCR_E: u8 = 0x1d;
pub const MVI_E: u8 = 0x1e;
pub const RAR: u8 = 0x1f;
pub const LXI_H: u8 = 0x21;
pub const SHLD: u8 = 0x22;
pub const INX_H: u8 = 0x23;
pub const INR_H: u8 = 0x24;
pub const DCR_H: u8 = 0x25;
pub const MVI_H: u8 = 0x26;
pub const DAA: u8 = 0x27;
pub const DAD_H: u8 = 0x29;
pub const LHLD: u8 = 0x2a;
pub const DCX_H: u8 = 0x2b;
pub const INR_L: u8 = 0x2c;
pub const DCR_L: u8 = 0x2d;
pub const MVI_L: u8 = 0x2e;
pub const CMA: u8 = 0x2f;
pub const LXI_SP: u8 = 0x31;
pub const STA: u8 = 0x32;
pub const INX_SP: u8 = 0x33;
pub const INR_M: u8 = 0x34;
pub const DCR_M: u8 = 0x35;
pub const MVI_M: u8 = 0x36;
pub const STC: u8 = 0x37;
pub const DAD_SP: u8 = 0x39;
pub const LDA: u8 = 0x3a;
pub const DCX_SP: u8 = 0x3b;
pub const INR_A: u8 = 0x3c;
pub const DCR_A: u8 = 0x3d;
pub const MVI_A: u8 = 0x3e;
pub const CMC: u8 = 0x3f;

pub const MOV_B_B: u8 = 0x40;
pub const MOV_B_C: u8 = 0x41;
pub const MOV_B_D: u8 = 0x42;
pub const MOV_B_E: u8 = 0x43;
pub const MOV_B_H: u8 = 0x44;
pub const MOV_B_L: u8 = 0x45;
pub const MOV_B_M: u8 = 0x46;
pub const MOV_B_A: u8 = 0x47;

pub const MOV_C_B: u8 = 0x48;
pub const MOV_C_C: u8 = 0x49;
pub const MOV_C_D: u8 = 0x4a;
pub const MOV_C_E: u8 = 0x4b;
pub const MOV_C_H: u8 = 0x4c;
pub const MOV_C_L: u8 = 0x4d;
pub const MOV_C_M: u8 = 0x4e;
pub const MOV_C_A: u8 = 0x4f;

pub const MOV_D_B: u8 = 0x50;
pub const MOV_D_C: u8 = 0x51;
pub const MOV_D_D: u8 = 0x52;
pub const MOV_D_E: u8 = 0x53;
pub const MOV_D_H: u8 = 0x54;
pub const MOV_D_L: u8 = 0x55;
pub const MOV_D_M: u8 = 0x56;
pub const MOV_D_A: u8 = 0x57;

pub const MOV_E_B: u8 = 0x58;
pub const MOV_E_C: u8 = 0x59;
pub const MOV_E_D: u8 = 0x5a;
pub const MOV_E_E: u8 = 0x5b;
pub const MOV_E_H: u8 = 0x5c;
pub const MOV_E_L: u8 = 0x5d;
pub const MOV_E_M: u8 = 0x5e;
pub const MOV_E_A: u8 = 0x5f;

pub const MOV_H_B: u8 = 0x60;
pub const MOV_H_C: u8 = 0x61;
pub const MOV_H_D: u8 = 0x62;
pub const MOV_H_E: u8 = 0x63;
pub const MOV_H_H: u8 = 0x64;
pub const MOV_H_L: u8 = 0x65;
pub const MOV_H_M: u8 = 0x66;
pub const MOV_H_A: u8 = 0x67;

pub const MOV_L_B: u8 = 0x68;
pub const MOV_L_C: u8 = 0x69;
pub const MOV_L_D: u8 = 0x6a;
pub const MOV_L_E: u8 = 0x6b;
pub const MOV_L_H: u8 = 0x6c;
pub const MOV_L_L: u8 = 0x6d;
pub const MOV_L_M: u8 = 0x6e;
pub const MOV_L_A: u8 = 0x6f;

pub const MOV_M_B: u8 = 0x70;
pub const MOV_M_C: u8 = 0x71;
pub const MOV_M_D: u8 = 0x72;
pub const MOV_M_E: u8 = 0x73;
pub const MOV_M_H: u8 = 0x74;
pub const MOV_M_L: u8 = 0x75;
pub const MOV_M_A: u8 = 0x77;

pub const MOV_A_B: u8 = 0x78;
pub const MOV_A_C: u8 = 0x79;
pub const MOV_A_D: u8 = 0x7a;
pub const MOV_A_E: u8 = 0x7b;
pub const MOV_A_H: u8 = 0x7c;
pub const MOV_A_L: u8 = 0x7d;
pub const MOV_A_M: u8 = 0x7e;

pub const ADD_B: u8 = 0x80;
pub const ADD_C: u8 = 0x81;
pub const ADD_D: u8 = 0x82;
pub const ADD_E: u8 = 0x83;
pub const ADD_H: u8 = 0x84;
pub const ADD_L: u8 = 0x85;
pub const ADD_M: u8 = 0x86;
pub const ADD_A: u8 = 0x87;

pub const ADC_B: u8 = 0x88;
pub const ADC_C: u8 = 0x89;
pub const ADC_D: u8 = 0x8a;
pub const ADC_E: u8 = 0x8b;
pub const ADC_H: u8 = 0x8c;
pub const ADC_L: u8 = 0x8d;
pub const ADC_M: u8 = 0x8e;
pub const ADC_A: u8 = 0x8f;

pub const SUB_B: u8 = 0x90;
pub const SUB_C: u8 = 0x91;
pub const SUB_D: u8 = 0x92;
pub const SUB_E: u8 = 0x93;
pub const SUB_H: u8 = 0x94;
pub const SUB_L: u8 = 0x95;
pub const SUB_M: u8 = 0x96;
pub const SUB_A: u8 = 0x97;

pub const SBB_B: u8 = 0x98;
pub const SBB_C: u8 = 0x99;
pub const SBB_D: u8 = 0x9a;
pub const SBB_E: u8 = 0x9b;
pub const SBB_H: u8 = 0x9c;
pub const SBB_L: u8 = 0x9d;
pub const SBB_M: u8 = 0x9e;
pub const SBB_A: u8 = 0x9f;

pub const ORA_B: u8 = 0xb0;
pub const ORA_C: u8 = 0xb1;
pub const ORA_D: u8 = 0xb2;
pub const ORA_E: u8 = 0xb3;
pub const ORA_H: u8 = 0xb4;
pub const ORA_L: u8 = 0xb5;
pub const ORA_M: u8 = 0xb6;
pub const ORA_A: u8 = 0xb7;

pub const CMP_B: u8 = 0xb8;
pub const CMP_C: u8 = 0xb9;
pub const CMP_D: u8 = 0xba;
pub const CMP_E: u8 = 0xbb;
pub const CMP_H: u8 = 0xbc;
pub const CMP_L: u8 = 0xbd;
pub const CMP_M: u8 = 0xbe;
pub const CMP_A: u8 = 0xbf;

pub const ANA_B: u8 = 0xa0;
pub const ANA_C: u8 = 0xa1;
pub const ANA_D: u8 = 0xa2;
pub const ANA_E: u8 = 0xa3;
pub const ANA_H: u8 = 0xa4;
pub const ANA_L: u8 = 0xa5;
pub const ANA_M: u8 = 0xa6;
pub const ANA_A: u8 = 0xa7;

pub const XRA_B: u8 = 0xa8;
pub const XRA_C: u8 = 0xa9;
pub const XRA_D: u8 = 0xaa;
pub const XRA_E: u8 = 0xab;
pub const XRA_H: u8 = 0xac;
pub const XRA_L: u8 = 0xad;
pub const XRA_M: u8 = 0xae;
pub const XRA_A: u8 = 0xaf;

pub const RNZ: u8 = 0xc0;
pub const POP_B: u8 = 0xc1;
pub const JNZ: u8 = 0xc2;
pub const JMP: u8 = 0xc3;
pub const CNZ: u8 = 0xc4;
pub const PUSH_B: u8 = 0xc5;
pub const ADI: u8 = 0xc6;
pub const RZ: u8 = 0xc8;
pub const RET: u8 = 0xc9;
pub const JZ: u8 = 0xca;
pub const CZ: u8 = 0xcc;
pub const CALL: u8 = 0xcd;
pub const ACI: u8 = 0xce;
pub const RST_1: u8 = 0xcf;
pub const RNC: u8 = 0xd0;
pub const POP_D: u8 = 0xd1;
pub const JNC: u8 = 0xd2;
pub const OUT: u8 = 0xd3;
pub const CNC: u8 = 0xd4;
pub const PUSH_D: u8 = 0xd5;
pub const SUI: u8 = 0xd6;
pub const RST_2: u8 = 0xd7;
pub const RC: u8 = 0xd8;
pub const JC: u8 = 0xda;
pub const IN: u8 = 0xdb;
pub const CC: u8 = 0xdc;
pub const SBI: u8 = 0xde;
pub const RPO: u8 = 0xe0;
pub const POP_H: u8 = 0xe1;
pub const JPO: u8 = 0xe2;
pub const XTHL: u8 = 0xe3;
pub const CPO: u8 = 0xe4;
pub const PUSH_H: u8 = 0xe5;
pub const ANI: u8 = 0xe6;
pub const RPE: u8 = 0xe8;
pub const PCHL: u8 = 0xe9;
pub const JPE: u8 = 0xea;
pub const XCHG: u8 = 0xeb;
pub const CPE: u8 = 0xec;
pub const XRI: u8 = 0xee;
pub const RP: u8 = 0xf0;
pub const POP_PSW: u8 = 0xf1;
pub const JP: u8 = 0xf2;
pub const DI: u8 = 0xf3;
pub const CP: u8 = 0xf4;
pub const PUSH_PSW: u8 = 0xf5;
pub const ORI: u8 = 0xf6;
pub const RM: u8 = 0xf8;
pub const SPHL: u8 = 0xf9;
pub const JM: u8 = 0xfa;
pub const EI: u8 = 0xfb;
pub const CM: u8 = 0xfc;
pub const CPI: u8 = 0xfe;
pub const RST_7: u8 = 0xff;

fn init_opcodes() -> HashMap<u8, Opcode> {
    // Opcode, size, disassembly name, cycles (appendix B of the ref manual)
    let ops: Vec<(u8, usize, &str)> = vec![
        (NOP, 1, "NOP"),
        (LXI_B, 3, "LD BC,"),
        (STAX_B, 1, "STA (BC)"),
        (INX_B, 1, "INC (BC)"),
        (INR_B, 1, "INR B"),
        (DCR_B, 1, "DEC B"),
        (MVI_B, 2, "LD B,"),
        (RLC, 1, "RLC"),
        (DAD_B, 1, "ADD HL,BC"),
        (LDAX_B, 1, "LD A,(BC)"),
        (DCX_B, 1, "DEC (BC)"),
        (INR_C, 1, "INR C"),
        (DCR_C, 1, "DEC C"),
        (MVI_C, 2, "LD C,"),
        (RRC, 1, "RRC"),
        (LXI_D, 3, "LD DE,"),
        (STAX_D, 1, "STA (DE)"),
        (INX_D, 1, "INC (DE)"),
        (INR_D, 1, "INR D"),
        (DCR_D, 1, "DEC D"),
        (MVI_D, 2, "LD D,"),
        (RAL, 1, "RAL"),
        (DAD_D, 1, "ADD HL,DE"),
        (LDAX_D, 1, "LD A,(DE)"),
        (DCX_D, 1, "DC (DE)"),
        (INR_E, 1, "INR E"),
        (DCR_E, 1, "DEC E"),
        (MVI_E, 2, "LD E,"),
        (RAR, 1, "RAR"),
        (LXI_H, 3, "LD HL,"),
        (SHLD, 3, "SHLD"),
        (INX_H, 1, "INC (HL)"),
        (INR_H, 1, "INR H"),
        (DCR_H, 1, "DEC HL"),
        (MVI_H, 2, "MOV H,"),
        (DAA, 1, "DAA"),
        (DAD_H, 1, "ADD HL,HL"),
        (LHLD, 3, "LHLD"),
        (DCX_H, 1, "DEC (HL)"),
        (INR_L, 1, "INR L"),
        (DCR_L, 1, "DEC L"),
        (MVI_L, 2, "LD L,"),
        (CMA, 1, "CMA"),
        (LXI_SP, 3, "LD SP,"),
        (STA, 3, "STA"),
        (INX_SP, 1, "INC (SP)"),
        (INR_M, 1, "INC (HL)"),
        (DCR_M, 1, "DEC (HL)"),
        (MVI_M, 2, "MV (HL),"),
        (STC, 1, "STC"),
        (DAD_SP, 1, "ADD HL,SP"),
        (0x3a, 3, "LDA"),
        (DCX_SP, 1, "DEC (SP)"),
        (INR_A, 1, "INR A"),
        (DCR_A, 1, "DEC A"),
        (MVI_A, 2, "LD A,"),
        (CMC, 1, "CMC"),
        (MOV_B_C, 1, "MOV B,C"),
        (MOV_B_D, 1, "MOV B,D"),
        (MOV_B_E, 1, "MOV B,E"),
        (MOV_B_H, 1, "MOV B,H"),
        (MOV_B_L, 1, "MOV B,L"),
        (MOV_B_M, 1, "MOV B,M"),
        (MOV_B_A, 1, "MOV B,A"),
        (MOV_C_B, 1, "MOV C,B"),
        (MOV_C_C, 1, "MOV C,C"),
        (MOV_C_D, 1, "MOV C,D"),
        (MOV_C_E, 1, "MOV C,E"),
        (MOV_C_H, 1, "MOV C,H"),
        (MOV_C_L, 1, "MOV C,L"),
        (MOV_C_M, 1, "MOV C,M"),
        (MOV_C_A, 1, "MOV C,A"),
        (MOV_D_B, 1, "MOV D,B"),
        (MOV_D_C, 1, "MOV D,C"),
        (MOV_D_D, 1, "MOV D,D"),
        (MOV_D_E, 1, "MOV D,E"),
        (MOV_D_H, 1, "MOV D,H"),
        (MOV_D_L, 1, "MOV D,L"),
        (MOV_D_M, 1, "MOV D,M"),
        (MOV_D_A, 1, "MOV D,A"),
        (MOV_E_B, 1, "MOV E,B"),
        (MOV_E_C, 1, "MOV E,C"),
        (MOV_E_D, 1, "MOV E,D"),
        (MOV_E_E, 1, "MOV E,E"),
        (MOV_E_H, 1, "MOV E,H"),
        (MOV_E_L, 1, "MOV E,L"),
        (MOV_E_M, 1, "MOV E,M"),
        (MOV_E_A, 1, "MOV E,A"),
        (MOV_H_B, 1, "MOV H,B"),
        (MOV_H_C, 1, "MOV H,C"),
        (MOV_H_D, 1, "MOV H,D"),
        (MOV_H_E, 1, "MOV H,E"),
        (MOV_H_H, 1, "MOV H,H"),
        (MOV_H_L, 1, "MOV H,L"),
        (MOV_H_M, 1, "MOV H,M"),
        (MOV_H_A, 1, "MOV H,A"),
        (MOV_L_B, 1, "MOV L,B"),
        (MOV_L_C, 1, "MOV L,C"),
        (MOV_L_D, 1, "MOV L,D"),
        (MOV_L_E, 1, "MOV L,E"),
        (MOV_L_H, 1, "MOV L,H"),
        (MOV_L_L, 1, "MOV L,L"),
        (MOV_L_M, 1, "MOV L,M"),
        (MOV_L_A, 1, "MOV L,A"),
        (MOV_M_B, 1, "LD (HL),B"),
        (MOV_M_C, 1, "LD (HL),C"),
        (MOV_M_D, 1, "LD (HL),D"),
        (MOV_M_E, 1, "LD (HL),E"),
        (MOV_M_H, 1, "LD (HL),H"),
        (MOV_M_L, 1, "LD (HL),L"),
        (MOV_M_A, 1, "LD (HL),A"),
        (MOV_A_B, 1, "MOV A,B"),
        (MOV_A_C, 1, "MOV A,C"),
        (MOV_A_D, 1, "MOV A,D"),
        (MOV_A_E, 1, "MOV A,E"),
        (MOV_A_H, 1, "MOV A,H"),
        (MOV_A_L, 1, "MOV A,L"),
        (MOV_A_M, 1, "MOV A,(HL)"),
        (ADD_B, 1, "ADD B"),
        (ADD_C, 1, "ADD C"),
        (ADD_D, 1, "ADD D"),
        (ADD_E, 1, "ADD E"),
        (ADD_H, 1, "ADD H"),
        (ADD_L, 1, "ADD L"),
        (ADD_M, 1, "ADD M"),
        (ADD_A, 1, "ADD A"),
        (ADC_B, 1, "ADC B"),
        (ADC_C, 1, "ADC C"),
        (ADC_D, 1, "ADC D"),
        (ADC_E, 1, "ADC E"),
        (ADC_H, 1, "ADC H"),
        (ADC_L, 1, "ADC L"),
        (ADC_M, 1, "ADC M"),
        (ADC_A, 1, "ADC A"),
        (SUB_B, 1, "SUB B"),
        (SUB_C, 1, "SUB C"),
        (SUB_D, 1, "SUB D"),
        (SUB_E, 1, "SUB E"),
        (SUB_H, 1, "SUB H"),
        (SUB_L, 1, "SUB L"),
        (SUB_M, 1, "SUB M"),
        (SUB_A, 1, "SUB A"),
        (SBB_B, 1, "SBB B"),
        (SBB_C, 1, "SBB C"),
        (SBB_D, 1, "SBB D"),
        (SBB_E, 1, "SBB E"),
        (SBB_H, 1, "SBB H"),
        (SBB_L, 1, "SBB L"),
        (SBB_M, 1, "SBB M"),
        (SBB_A, 1, "SBB A"),
        (ANA_B, 1, "AND B"),
        (ANA_C, 1, "AND C"),
        (ANA_D, 1, "AND D"),
        (ANA_E, 1, "AND E"),
        (ANA_H, 1, "AND H"),
        (ANA_L, 1, "AND L"),
        (ANA_M, 1, "AND M"),
        (ANA_A, 1, "AND A"),
        (ORA_B, 1, "ORA B"),
        (ORA_C, 1, "ORA C"),
        (ORA_D, 1, "ORA D"),
        (ORA_E, 1, "ORA E"),
        (ORA_H, 1, "ORA H"),
        (ORA_L, 1, "ORA L"),
        (ORA_M, 1, "ORA M"),
        (ORA_A, 1, "ORA A"),
        (CMP_B, 1, "CMP B"),
        (CMP_C, 1, "CMP C"),
        (CMP_D, 1, "CMP D"),
        (CMP_E, 1, "CMP E"),
        (CMP_H, 1, "CMP H"),
        (CMP_L, 1, "CMP L"),
        (CMP_M, 1, "CMP M"),
        (CMP_A, 1, "CMP A"),
        (XRA_B, 1, "XRA B"),
        (XRA_C, 1, "XRA C"),
        (XRA_D, 1, "XRA D"),
        (XRA_E, 1, "XRA E"),
        (XRA_H, 1, "XRA H"),
        (XRA_L, 1, "XRA L"),
        (XRA_M, 1, "XRA M"),
        (XRA_A, 1, "XRA A"),
        (RNZ, 1, "RNZ"),
        (POP_B, 1, "POP BC"),
        (JNZ, 3, "JNZ"),
        (JMP, 3, "JMP"),
        (CNZ, 3, "CNZ"),
        (PUSH_B, 1, "PUSH BC"),
        (ADI, 2, "ADI"),
        (RZ, 1, "RZ"),
        (RET, 1, "RET"),
        (JZ, 3, "JZ"),
        (CZ, 3, "CZ"),
        (CALL, 3, "CALL"),
        (ACI, 2, "ACI"),
        (RNC, 1, "RNC"),
        (POP_D, 1, "POP DE"),
        (JNC, 3, "JNC"),
        (OUT, 2, "OUT"),
        (CNC, 3, "CNC"),
        (PUSH_D, 1, "PUSH DE"),
        (SUI, 2, "SUI"),
        (RC, 1, "RC"),
        (JC, 3, "JC"),
        (IN, 2, "IN"),
        (CC, 3, "CC"),
        (SBI, 2, "SBI"),
        (RPO, 1, "RPO"),
        (POP_H, 1, "POP HL"),
        (JPO, 3, "JPO"),
        (XTHL, 1, "XTHL"),
        (CPO, 3, "CPO"),
        (PUSH_H, 1, "PUSH HL"),
        (ANI, 2, "ANI"),
        (RPE, 1, "RPE"),
        (PCHL, 1, "PCHL"),
        (JPE, 3, "JPE"),
        (XCHG, 1, "EX DE,HL"),
        (CPE, 3, "CPE"),
        (XRI, 2, "XRI"),
        (RP, 1, "RP"),
        (POP_PSW, 1, "POP PSW"),
        (JP, 3, "JP"),
        (CP, 3, "CP"),
        (PUSH_PSW, 1, "PUSH PSW"),
        (ORI, 2, "ORI"),
        (RM, 1, "RM"),
        (SPHL, 1, "SPHL"),
        (JM, 3, "JM"),
        (CM, 3, "CM"),
        (EI, 1, "EI"),
        (DI, 1, "DI"),
        (CPI, 2, "CPI"),
        (RST_1, 2, "RST 1"),
        (RST_2, 2, "RST 2"),
        (RST_7, 2, "RST 7"),
    ];
    let mut result: HashMap<u8, Opcode> = HashMap::new();
    for op in ops {
        if result.get(&op.0).is_some() {
            panic!("REPEATED OPCODE {:02x}", op.0);
        }
        result.insert(op.0, Opcode::new(op.0, op.1, op.2));
    }
    result
}

lazy_static! {
    pub static ref OPCODES: HashMap<u8, Opcode> = init_opcodes();
}

// fn insert(map: &mut HashMap<u8, Opcode>, opcode: u8, size: usize, name: &'static str) {
//     map.insert(opcode, Opcode::new(opcode, size, name));
// }
