use crate::{
    bus::{Bus, BusAccessOutcome},
    cartridge::cartridge::Cartridge,
    os_interface::profiling::TrackedData,
    processor::{
        cpu::{Cpu, CpuOperationContext},
        instructions::*,
    },
};

pub const EXPECTED_CLOCK_SPEED: f32 = 4.194304; // In Megahertz
#[derive(Default)]
pub struct GameBoy {
    mode: Mode,
    cpu: Cpu,
    bus: Bus,
    elapsed_cycles: TCycles,
}

impl GameBoy {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge)
    }

    pub fn test_looping(&mut self, cycles: u64) {
        let tracked_data = TrackedData::new();
        while self.elapsed_cycles.0 < cycles {
            let just_ticked = self.tick_cpu_execution().into();
            self.elapsed_cycles += just_ticked;
        }

        tracked_data.log_from_gameboy(self);
    }

    fn tick_cpu_execution(&mut self) -> MCycles {
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

        MCycles(taken_cycles)
    }

    fn read_next_instruction(&mut self) -> BusAccessOutcome<&'static Instruction> {
        let pc = self.cpu.get_pc();
        self.bus.read_next_instruction(pc)
    }

    pub fn get_elapsed_cycles(&self) -> TCycles {
        self.elapsed_cycles
    }
}

#[derive(Default)]
pub enum Mode {
    #[default]
    Executing,
    Stopped,
    Halted,
}

/// Instruction and memory access clock cycle
#[derive(Default, PartialEq, PartialOrd, Clone, Copy)]
pub struct MCycles(pub u64);

/// Smallest unit of time for the Game Boy
#[derive(Default, PartialEq, PartialOrd, Clone, Copy)]
pub struct TCycles(pub u64);

impl From<MCycles> for TCycles {
    fn from(value: MCycles) -> Self {
        Self(value.0 * 4)
    }
}

impl std::ops::AddAssign for TCycles {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
