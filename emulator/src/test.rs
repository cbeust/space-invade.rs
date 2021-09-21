#[cfg(test)]
mod test {
    use crate::memory::Memory;
    use crate::emulator::{Emulator, StepResult, StepStatus};

    #[test]
    fn run_cpu_diag() {
        let mut memory = Memory::new();
        let start = 0x100;
        memory.read_file("cpudiag.bin", start);
        let mut computer = Emulator::new(Box::new(memory), start as usize);
        let mut result = StepResult { status: StepStatus::Continue, cycles: 0 };
        unsafe {
            while result.status == StepStatus::Continue {
                result = computer.step(true);
            }
        }
        match result.status {
            StepStatus::Success(s) => {
                println!("Success: {}", s)
            },
            StepStatus::Failure(s) => {
                panic!(s);
            },
            _ => {
                println!("Something went wrong");
            }
        }
    }
}
