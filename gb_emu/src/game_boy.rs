use crate::{
    bus::Bus,
    cpu::{Cpu, OperationContext},
    instructions::*,
};

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
        let (instruction, bytes) = self.read_next_instruction();
        OperationContext::new(&mut self.cpu, &mut self.bus, bytes)
            .perform_instruction(instruction)
            .unwrap();
    }

    fn read_next_instruction(&mut self) -> (&'static Instruction, [u8; 3]) {
        let pc = self.cpu.get_pc();
        self.bus
            .read_next_instruction(pc)
            .expect("We'll want a top level error that other errors can convert to later.")
    }
}
