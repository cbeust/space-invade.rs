pub mod emulator;
pub mod memory;
pub mod state;
pub mod emulator_state;
pub mod opcodes;
pub mod listener;
mod test;

const VERBOSE: bool = false;
static mut VERBOSE_DISASSEMBLE: bool = false;
const VERBOSE_GRAPHIC: bool = true;
const VERBOSE_DISASSEMBLE_SECTION: bool = false;
const DISASSEMBLE_SECTION_START: usize = 0x1439;
const DISASSEMBLE_SECTION_END: usize = 0x1447;
// const DISASSEMBLE_SECTION_START: usize = 0x1439;
// const DISASSEMBLE_SECTION_END: usize = 0x1447;
const VERBOSE_MEMORY: bool = false;


