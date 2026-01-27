use crate::{
    bus::{Bus, MemoryMapEvent, MemoryTarget},
    cartridge::cartridge::Cartridge,
    dma::OamDma,
    graphics::{
        lcd,
        ppu::{Dots, Ppu, PpuOperationContext, PpuTickMode},
    },
    helper_functions::log,
    os_interface::profiling::TrackedData,
    processor::cpu::Cpu,
};

pub const EXPECTED_CLOCK_SPEED: f64 = 4.194304; // In Megahertz
#[derive(Default)]
pub struct GameBoy {
    state: GameBoyState,
    cpu: Cpu,
    ppu: Ppu,
    bus: Bus,
    oam_dma: OamDma,
}

impl GameBoy {
    pub fn new() -> Self {
        Default::default()
    }

    fn log_last_n_instructions(&self, n: usize) {
        let len = self.cpu.executed_instructions.len();
        let start = len.saturating_sub(n);
        for instruction in &self.cpu.executed_instructions[start..] {
            log(format_args!(
                "PC: 0x{:04x}, Instruction: {}",
                instruction.1, instruction.0
            ));
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge)
    }

    pub fn test_looping(&mut self, cycles: u64) {
        let tracked_data = TrackedData::new();
        while self.state.elapsed_cpu_cycles.0 < cycles {
            self.tick();
        }

        tracked_data.log_from_gameboy(self);

        self.bus.print_graphics_data();
        self.log_last_n_instructions(0x100);
    }

    fn tick_cpu(&mut self) -> Vec<GameBoyStateChange> {
        let (m_cycles, changes) = self.cpu.tick(&mut self.bus);

        self.state.elapsed_cpu_cycles += m_cycles.into();
        self.state.elapsed_cpu_cycles = m_cycles.into();

        changes
    }

    fn tick_oam_dma(&mut self) -> Vec<GameBoyStateChange> {
        for _ in 0..self.state.last_instruction_t_cycles.0 {
            let complete = self.oam_dma.tick_transfer(&mut self.bus);
            if complete {
                self.state.oam_dma_active = false;
                return vec![GameBoyStateChange::EndOamDmaTransfer];
            }
        }

        Vec::new()
    }

    fn tick_ppu_enabled(&mut self) -> Vec<GameBoyStateChange> {
        let mut changes = Vec::new();
        let (v_ram, oam, lcd_regs) = self.bus.get_ppu_context_mem();
        for _ in 0..self.state.last_instruction_t_cycles.0 {
            changes.append(&mut self.ppu.tick_ppu_enabled(v_ram, oam, lcd_regs))
        }
        changes
    }

    fn tick_ppu_disabled(&mut self) -> Vec<GameBoyStateChange> {
        let mut changes = Vec::new();
        let (v_ram, oam, lcd_regs) = self.bus.get_ppu_context_mem();
        for _ in 0..self.state.last_instruction_t_cycles.0 {
            changes.append(&mut self.ppu.tick_ppu_disabled(v_ram, oam, lcd_regs))
        }
        changes
    }

    fn tick(&mut self) {
        let mut changes = Vec::new();
        if self.state.is_cpu_active() {
            changes.append(&mut self.tick_cpu());
        } else {
            self.state.last_instruction_t_cycles = TCycles(1);
        }
        if self.state.is_ppu_active() {
            changes.append(&mut self.tick_ppu_enabled());
        } else {
            changes.append(&mut self.tick_ppu_disabled());
        }
        if self.state.is_oam_dma_active() {
            changes.append(&mut self.tick_oam_dma());
        }
        self.handle_changes(changes);
    }

    fn handle_changes(&mut self, changes: Vec<GameBoyStateChange>) {
        for change in changes {
            self.handle_change(change)
        }
    }

    fn handle_change(&mut self, change: GameBoyStateChange) {
        match change {
            GameBoyStateChange::UnmapBootRom => self.bus.handle_memory_map_event(MemoryMapEvent::UnmapBootRom),
            // GameBoyStateChange::UnmapBootRom => self.state.elapsed_cpu_cycles = TCycles(u32::MAX as u64),
            GameBoyStateChange::ChangeGameBoyMode(mode) => self.mode_transition(mode),
            GameBoyStateChange::ChangeSelectedWorkRam(bank_num) => {
                self.bus.set_active_bank_number(MemoryTarget::BankableWorkRam, bank_num)
            },
            GameBoyStateChange::ChangeObjectPriorityMode(mode) => self.bus.set_object_priority_mode(mode),
            GameBoyStateChange::StartOamDmaTransfer(input) => self.initiate_dma_transfer(input),
            GameBoyStateChange::EndOamDmaTransfer => self.end_dma_transfer(),
            GameBoyStateChange::UpdatePpuMode(mode) => {
                self.bus.handle_memory_map_event(MemoryMapEvent::UpdatePpuMode(mode))
            },
            GameBoyStateChange::ChangeLCdPpuState(enabled) => self.state.ppu_active = enabled,
            GameBoyStateChange::Interrupt(interrupt) => (),
        }
    }

    fn initiate_dma_transfer(&mut self, input: u8) {
        self.oam_dma.initiate_transfer(input);
        self.state.oam_dma_active = true;
        self.bus.handle_memory_map_event(MemoryMapEvent::StartOamDataTransfer);
    }

    fn end_dma_transfer(&mut self) {
        self.state.oam_dma_active = false;
        self.bus.handle_memory_map_event(MemoryMapEvent::EndOamDataTransfer);
    }

    fn mode_transition(&mut self, new_mode: GameBoyMode) {
        self.state.mode_transition(new_mode);
    }

    pub fn get_elapsed_cycles(&self) -> TCycles {
        self.state.elapsed_cpu_cycles
    }
}

struct GameBoyState {
    mode: GameBoyMode,
    elapsed_cpu_cycles: TCycles,
    last_instruction_t_cycles: TCycles,
    oam_dma_active: bool,
    ppu_active: bool,
    cpu_active: bool,
}

impl GameBoyState {
    fn mode_transition(&mut self, new_mode: GameBoyMode) {
        self.mode = new_mode;
        todo!()
    }
    fn is_cpu_active(&self) -> bool {
        self.cpu_active
    }
    fn is_ppu_active(&self) -> bool {
        self.ppu_active
    }
    fn is_oam_dma_active(&self) -> bool {
        self.oam_dma_active
    }
}

impl Default for GameBoyState {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            elapsed_cpu_cycles: Default::default(),
            last_instruction_t_cycles: TCycles(1),
            oam_dma_active: false,
            ppu_active: false,
            cpu_active: true,
        }
    }
}

#[derive(Default, Debug)]
pub enum GameBoyMode {
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
pub enum GameBoyStateChange {
    UnmapBootRom,
    ChangeGameBoyMode(GameBoyMode),
    ChangeSelectedWorkRam(u8),
    ChangeLCdPpuState(Enabled),
    ChangeObjectPriorityMode(crate::graphics::oam::PriorityMode),
    StartOamDmaTransfer(u8),
    UpdatePpuMode(PpuTickMode),
    EndOamDmaTransfer,
    Interrupt(Interrupt),
}

#[derive(Debug)]
pub enum Interrupt {
    LycEqualsLy,
}

pub type Enabled = bool;

pub enum HardwareType {
    Dmg,
    Cgb,
    Sgb,
}
