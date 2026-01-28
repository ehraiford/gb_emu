use std::sync::{Mutex, OnceLock};

use crate::{
    bus::{Bus, MemoryMapEvent, MemoryTarget},
    cartridge::cartridge::Cartridge,
    dma::OamDma,
    graphics::ppu::{Ppu, PpuTickMode},
    helper_functions::log,
    os_interface::profiling::TrackedData,
    processor::cpu::Cpu,
};

pub const EXPECTED_CLOCK_SPEED: f64 = 4.194304; // In Megahertz

static GAMEBOY_EVENTS: OnceLock<Mutex<Vec<GameBoyEvent>>> = OnceLock::new();

fn get_events_mut() -> &'static Mutex<Vec<GameBoyEvent>> {
    GAMEBOY_EVENTS.get_or_init(|| Mutex::new(Vec::new()))
}

fn drain_events() -> Vec<GameBoyEvent> {
    let mut events = get_events_mut().lock().unwrap();
    std::mem::take(&mut *events)
}

pub fn notate_event(event: GameBoyEvent) {
    get_events_mut().lock().unwrap().push(event);
}

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
    }

    fn tick_cpu(&mut self) {
        let t_cycles = self.cpu.tick(&mut self.bus);
        self.state.elapsed_cpu_cycles += t_cycles;

        self.state.cpu_lockstep_catchup = t_cycles;

        if self.cpu.get_pc() == 0x3e {
            self.bus.print_graphics_data();
            panic!();
        }
    }

    fn tick_oam_dma(&mut self) {
        let complete = self.oam_dma.tick_transfer(&mut self.bus);
        if complete {
            self.state.oam_dma_active = false;
            notate_event(GameBoyEvent::EndOamDmaTransfer);
        }
    }

    fn tick_ppu_enabled(&mut self) {
        let (v_ram, oam, lcd_regs) = self.bus.get_ppu_context_mem();

        self.ppu.tick_ppu_enabled(v_ram, oam, lcd_regs)
    }

    fn tick_ppu_disabled(&mut self) {
        let (v_ram, oam, lcd_regs) = self.bus.get_ppu_context_mem();
        self.ppu.tick_ppu_disabled(v_ram, oam, lcd_regs)
    }

    fn tick(&mut self) {
        if self.state.is_cpu_active() {
            self.tick_cpu();
        } else {
            self.state.cpu_lockstep_catchup = TCycles(1);
        }
        self.handle_changes();
        while self.state.cpu_lockstep_catchup.0 != 0 {
            if self.state.is_ppu_active() {
                self.tick_ppu_enabled();
            } else {
                self.tick_ppu_disabled();
            }
            if self.state.is_oam_dma_active() {
                self.tick_oam_dma();
            }
            self.state.cpu_lockstep_catchup.0 -= 1;
        }
        self.handle_changes();
    }

    fn handle_changes(&mut self) {
        for change in drain_events() {
            self.handle_change(change)
        }
    }

    fn handle_change(&mut self, change: GameBoyEvent) {
        match change {
            GameBoyEvent::UnmapBootRom => self.bus.handle_memory_map_event(MemoryMapEvent::UnmapBootRom),
            GameBoyEvent::ChangeGameBoyMode(mode) => self.mode_transition(mode),

            GameBoyEvent::ChangeObjectPriorityMode(mode) => self.bus.set_object_priority_mode(mode),
            GameBoyEvent::StartOamDmaTransfer(input) => self.initiate_dma_transfer(input),
            GameBoyEvent::EndOamDmaTransfer => self.end_dma_transfer(),
            GameBoyEvent::UpdatePpuMode(mode) => self.bus.handle_memory_map_event(MemoryMapEvent::UpdatePpuMode(mode)),
            GameBoyEvent::ChangeLCdPpuState(enabled) => {
                self.bus.reset_ly();
                self.state.ppu_active = enabled;
            },
            GameBoyEvent::Interrupt(interrupt) => (),
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
    elapsed_cpu_cycles: TCycles,
    cpu_lockstep_catchup: TCycles,
    oam_dma_active: bool,
    ppu_active: bool,
    cpu_active: bool,
}

impl GameBoyState {
    fn mode_transition(&mut self, new_mode: GameBoyMode) {
        match new_mode {
            GameBoyMode::Executing => todo!(),
            GameBoyMode::Stopped => {
                self.cpu_active = false;
                self.ppu_active = false;
            },
            GameBoyMode::Halted => todo!(),
        }
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
            elapsed_cpu_cycles: Default::default(),
            cpu_lockstep_catchup: TCycles(1),
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
pub enum GameBoyEvent {
    UnmapBootRom,
    ChangeGameBoyMode(GameBoyMode),
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
