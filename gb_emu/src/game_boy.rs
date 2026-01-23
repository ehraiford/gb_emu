use std::time::Instant;

use crate::{
    bus::{Bus, BusAccessOutcome},
    cartridge::cartridge::Cartridge,
    processor::{
        cpu::{Cpu, CpuOperationContext},
        instructions::*,
    },
};

const EXPECTED_CLOCK_SPEED: f64 = 4.194304; // In Megahertz
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
        let start = Instant::now();
        let mut total_t_cycles = 0;
        while total_t_cycles < cycles {
            total_t_cycles += self.tick_cpu_execution().0 as usize;
        }

        self.bus.print_graphics_data();
        let duration = start.elapsed();
        print!(
            "It took us {} seconds to run through {total_t_cycles} cycles.",
            duration.as_secs()
        );
        println!(
            "That's {:.2} MHz. Hardware is {EXPECTED_CLOCK_SPEED:.2} MHz.",
            ((total_t_cycles * 4) as f64) / duration.as_secs_f64() / 1_000_000_f64
        )
    }

    fn tick_cpu_execution(&mut self) -> TCycles {
        let BusAccessOutcome(instruction, side_effects) = self.read_next_instruction();
        let BusAccessOutcome(instruction_outcome, instruction_bus_outcome) =
            CpuOperationContext::new(&mut self.cpu, &mut self.bus).perform_instruction(instruction);
        let pc = self.cpu.get_pc() + instruction.bytes;
        self.cpu.set_pc(pc);
        let mut taken_cycles = instruction.cycles as u64;

        match instruction_outcome {
            InstructionOutcome::ExtraCycles(extra_cycles) => taken_cycles += extra_cycles as u64,
            InstructionOutcome::Ok => (),
            InstructionOutcome::ChangeGameBoyMode(mode) => self.mode = mode,
        };

        TCycles(taken_cycles)
    }

    fn read_next_instruction(&mut self) -> BusAccessOutcome<&'static Instruction> {
        let pc = self.cpu.get_pc();
        self.bus.read_next_instruction(pc)
    }
}

#[derive(Default)]
pub enum Mode {
    #[default]
    Executing,
    Stopped,
    Halted,
}
