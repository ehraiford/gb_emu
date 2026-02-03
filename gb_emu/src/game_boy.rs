use std::time::{Duration, Instant};

use crate::{
    bus::{Bus, MemoryMapEvent},
    cartridge::cartridge::Cartridge,
    graphics::ppu::{Ppu, PpuTickMode},
    io_devices::{dma::OamDma, interrupts::Interrupt},
    os_interface::profiling::TrackedData,
    processor::cpu::Cpu,
};

pub const EXPECTED_CLOCK_SPEED: f64 = 4.194304; // In Megahertz

use std::cell::RefCell;

thread_local! {
    static GAMEBOY_EVENTS: RefCell<Vec<GameBoyEvent>> = RefCell::new(Vec::new());
}

pub fn notate_event(event: GameBoyEvent) {
    GAMEBOY_EVENTS.with(|events| {
        events.borrow_mut().push(event);
    });
}

fn drain_events() -> Vec<GameBoyEvent> {
    GAMEBOY_EVENTS.with(|events| events.take())
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
        let desired_duration = Duration::from_secs_f64(1.0 / 4194304.0);
        let mut next_checkin = 16_667;
        while self.state.elapsed_cpu_cycles.0 < cycles {
            let start = Instant::now();

            while self.state.elapsed_cpu_cycles.0 < next_checkin {
                self.tick();
            }

            let expected_duration = desired_duration.mul_f64(16_667.0);

            let elapsed_time = start.elapsed();
            if expected_duration > elapsed_time {
                std::thread::sleep(expected_duration - elapsed_time);
            }

            next_checkin = self.state.elapsed_cpu_cycles.0 + 16_667;
        }

        tracked_data.log_from_gameboy(self);
    }

    fn tick_cpu(&mut self) {
        let t_cycles = self.cpu.tick(&mut self.bus);
        self.state.elapsed_cpu_cycles += t_cycles;

        self.state.cpu_lockstep_catchup = t_cycles;
    }

    fn tick_oam_dma(&mut self) {
        self.oam_dma.tick(&mut self.bus);
    }

    fn tick_ppu(&mut self) {
        let (v_ram, oam, lcd_regs) = self.bus.get_ppu_context_mem();

        self.ppu.tick(v_ram, oam, lcd_regs)
    }

    fn tick_timer_divider(&mut self) {
        self.bus.tick_timer_divider();
    }

    fn tick_peripherals_lockstep(&mut self) {
        for _ in 0..(self.state.cpu_lockstep_catchup.0 / 4) {
            self.tick_timer_divider();
            self.tick_oam_dma();
            self.tick_ppu();
        }
        self.state.cpu_lockstep_catchup.0 = 0;
    }

    pub fn tick(&mut self) {
        match self.state.mode {
            GameBoyMode::Executing => {
                self.tick_cpu();
                self.handle_changes();
                self.tick_peripherals_lockstep();
                self.handle_changes();
            },
            GameBoyMode::Stopped => {
                self.state.cpu_lockstep_catchup = TCycles(4);
                self.tick_peripherals_lockstep();
                self.handle_changes();
            },
            GameBoyMode::Halted => todo!(),
        }
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
            GameBoyEvent::ChangeBusAccessForPpuMode(mode) => {
                self.bus.handle_memory_map_event(MemoryMapEvent::UpdatePpuMode(mode))
            },
            GameBoyEvent::EnableLcdPpu => self.handle_enabled_lcd_ppu(),
            GameBoyEvent::Interrupt(interrupt) => self.bus.raise_interrupt_flag(&interrupt),
            GameBoyEvent::IeTriggered => notate_event(GameBoyEvent::EnableInterrupts),
            GameBoyEvent::EnableInterrupts => self.cpu.enable_interrupts(),
            GameBoyEvent::ObjectsDisabled => self.handle_objects_disabled(),
        }
    }

    fn handle_objects_disabled(&mut self) {
        // DisabledObjects events should only ever be generated when the CPU writes to 0xFF40
        // which means `cpu_lockstep_catchup` should still be the full length in TCycles of the affecting instruction
        // So we can use that to delay mode 3 of the PPU.
        // I don't love this shortcut because it's not resistant to reorganization
        // but it's better than adding another assignment in the hot loop.
        self.ppu.handle_objects_disabled(self.state.cpu_lockstep_catchup)
    }

    fn handle_enabled_lcd_ppu(&mut self) {
        self.bus.reset_ly();
        self.ppu.enable();
    }

    fn initiate_dma_transfer(&mut self, input: u8) {
        self.oam_dma.initiate_transfer(input);
        self.state.oam_dma_active = true;
        self.bus.handle_memory_map_event(MemoryMapEvent::StartOamDataTransfer);
    }

    fn end_dma_transfer(&mut self) {
        self.bus.handle_memory_map_event(MemoryMapEvent::EndOamDataTransfer);
    }

    fn mode_transition(&mut self, new_mode: GameBoyMode) {
        self.state.mode_transition(new_mode, &mut self.bus);
    }

    pub fn get_elapsed_cycles(&self) -> TCycles {
        self.state.elapsed_cpu_cycles
    }
}

struct GameBoyState {
    mode: GameBoyMode,
    elapsed_cpu_cycles: TCycles,
    cpu_lockstep_catchup: TCycles,
    oam_dma_active: bool,
}

impl GameBoyState {
    fn mode_transition(&mut self, new_mode: GameBoyMode, bus: &mut Bus) {
        match new_mode {
            GameBoyMode::Executing => todo!(),
            GameBoyMode::Stopped => bus.reset_divider_register(),
            GameBoyMode::Halted => todo!(),
        }
        self.mode = new_mode;
    }
}

impl Default for GameBoyState {
    fn default() -> Self {
        Self {
            elapsed_cpu_cycles: Default::default(),
            cpu_lockstep_catchup: TCycles(0),
            oam_dma_active: false,
            mode: Default::default(),
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
    EnableLcdPpu,
    ObjectsDisabled,
    ChangeObjectPriorityMode(crate::graphics::oam::PriorityMode),
    StartOamDmaTransfer(u8),
    ChangeBusAccessForPpuMode(PpuTickMode),
    EndOamDmaTransfer,
    Interrupt(Interrupt),
    IeTriggered, // Facilitates the delay between executing IE and actually enabling interrupts
    EnableInterrupts,
}
