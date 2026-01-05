use std::path::PathBuf;

use crate::{bus::{Bus, BusAccessible, MMDevice}, cpu::Cpu, instructions::*};



#[derive(Default)]
pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn test_looping(&mut self, cycles: usize) {

        for _ in 0..cycles {
            self.tick_cpu_execution();
        }
    }

    fn tick_cpu_execution(&mut self) {
        let instruction = self.read_next_instruction();
        OperationContext::new(&mut self.cpu, &mut self.bus).perform_instruction(&instruction);
    }

    fn read_next_instruction(&mut self) -> Instruction {
        let pc = self.cpu.get_pc();
        self.bus.read_next_instruction(pc)
    }
}

