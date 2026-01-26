use crate::{
    bus::{Bus, MemoryTarget},
    cartridge::cartridge::Cartridge,
    dma::OamDma,
    os_interface::profiling::TrackedData,
    processor::cpu::Cpu,
};

pub const EXPECTED_CLOCK_SPEED: f64 = 4.194304; // In Megahertz
#[derive(Default)]
pub struct GameBoy {
    state: GameBoyState,
    cpu: Cpu,
    bus: Bus,
    oam_dma: OamDma,
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
        while self.state.elapsed_cycles.0 < cycles {
            self.tick();
        }

        tracked_data.log_from_gameboy(self);

        // self.bus.print_graphics_data();
    }

    fn tick_cpu(&mut self) {
        let (m_cycles, changes) = self.cpu.tick_execution(&mut self.bus);
        self.handle_changes(changes);
        self.state.elapsed_cycles += m_cycles.into();
    }

    fn tick_oam_dma(&mut self) {
        if self.state.oam_dma_active {
            let complete = self.oam_dma.tick_transfer(&mut self.bus);
            if complete {
                self.state.oam_dma_active = false;
                self.bus.end_oam_dma_transfer();
            }
        }
    }

    fn tick_ppu(&mut self) {}

    fn tick(&mut self) {
        match self.state.mode {
            Mode::Executing => {
                self.tick_cpu();
                self.tick_oam_dma();
                self.tick_ppu();
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
            Change::ChangeGameBoyMode(mode) => self.state.mode = mode,
            Change::ChangeSelectedWorkRam(bank_num) => {
                self.bus.set_active_bank_number(MemoryTarget::BankableWorkRam, bank_num)
            },
            Change::ChangeObjectPriorityMode(mode) => self.bus.set_object_priority_mode(mode),
            Change::StartOamDmaTransfer(input) => self.initiate_dma_transfer(input),
        }
    }

    fn initiate_dma_transfer(&mut self, input: u8) {
        self.oam_dma.initiate_transfer(input);
        self.state.oam_dma_active = true;
        self.bus.start_oam_dma_transfer();
    }

    pub fn get_elapsed_cycles(&self) -> TCycles {
        self.state.elapsed_cycles
    }
}

struct GameBoyState {
    mode: Mode,
    elapsed_cycles: TCycles,
    oam_dma_active: bool,
}

impl Default for GameBoyState {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            elapsed_cycles: Default::default(),
            oam_dma_active: false,
        }
    }
}

#[derive(Default, Debug)]
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

#[derive(Debug)]
pub enum Change {
    UnmapBootRom,
    ChangeGameBoyMode(Mode),
    ChangeSelectedWorkRam(u8),
    ChangeObjectPriorityMode(crate::graphics::oam::PriorityMode),
    StartOamDmaTransfer(u8),
    // Add others here
}

pub enum HardwareType {
    Dmg,
    Cgb,
    Sgb,
}
