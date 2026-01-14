use crate::{
    bus::Bus,
    cartridge::cartridge::Cartridge,
    cpu::{Cpu, OperationContext},
    instructions::*,
};

#[derive(Default)]
pub struct GameBoy {
    mode: Mode,
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge)
    }

    pub fn test_looping(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.tick_cpu_execution();
        }
    }

    fn tick_cpu_execution(&mut self) {
        let instruction = self.read_next_instruction();
        let outcome = OperationContext::new(&mut self.cpu, &mut self.bus)
            .perform_instruction(instruction)
            .unwrap();

        let mut pc = self.cpu.get_pc();
        pc += instruction.bytes;

        match outcome {
            InstructionOutcome::ExtraCycles(extra_cycles) => pc += extra_cycles,
            InstructionOutcome::Ok => (),
            InstructionOutcome::ChangeGameBoyMode(mode) => self.mode = mode,
        }

        self.cpu.set_pc(pc);
    }

    fn read_next_instruction(&mut self) -> &'static Instruction {
        let pc = self.cpu.get_pc();
        self.bus
            .read_next_instruction(pc)
            .expect("We'll want a top level error that other errors can convert to later.")
    }
}

#[derive(Default)]
pub enum Mode {
    #[default]
    Executing,
    Stopped,
    Halted,
}
