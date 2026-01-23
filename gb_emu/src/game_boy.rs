use crate::{
    bus::{Bus, BusAccessOutcome},
    cartridge::cartridge::Cartridge,
    os_interface::profiling::TrackedData,
    processor::{
        cpu::{Cpu, CpuOperationContext},
        instructions::*,
    },
};

pub const EXPECTED_CLOCK_SPEED: f64 = 4.194304; // In Megahertz
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
            let just_ticked: TCycles = self.tick();
            self.elapsed_cycles += just_ticked;
        }

        tracked_data.log_from_gameboy(self);

        // self.bus.print_graphics_data();
    }

    fn tick(&mut self) -> TCycles {
        match self.mode {
            Mode::Executing => {
                let (m_cycles, changes) = self.cpu.tick_execution(&mut self.bus);
                self.handle_changes(changes);
                m_cycles.into()
            },
            Mode::Stopped => todo!(),
            Mode::Halted => todo!(),
        }
    }

    fn handle_changes(&mut self, changes: Vec<Change>) {
        for change in changes {
            self.handle_change(change)
        }
    }

    fn handle_change(&mut self, change: Change) {
        match change {
            Change::UnmapBootRom => self.bus.unmap_bootrom(),
            Change::ChangeGameBoyMode(mode) => self.mode = mode,
        }
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
pub enum Change {
    UnmapBootRom,
    ChangeGameBoyMode(Mode),
    // Add others here
}
