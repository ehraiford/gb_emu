use crate::{
    bus::{Bus, MemoryMapEvent},
    cartridge::cartridge::Cartridge,
    graphics::{ppu::{Ppu, PpuTickMode}, video_ram::TileMapImage},
    io_devices::{dma::OamDma, interrupts::Interrupt, joypad_input::ButtonInput},
    os_interface::window::SenderFrameHandle,
    processor::cpu::Cpu,
};

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

pub struct GameBoy {
    state: GameBoyState,
    cpu: Cpu,
    ppu: Ppu,
    bus: Bus,
    oam_dma: OamDma,
}

impl GameBoy {
    pub fn new(frame_handle: SenderFrameHandle, button_input: ButtonInput) -> Self {
        Self {
            ppu: Ppu::new(frame_handle),
            state: Default::default(),
            cpu: Default::default(),
            bus: Bus::new(button_input),
            oam_dma: Default::default(),
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge)
    }

    fn tick_cpu(&mut self) {
        let t_cycles = self.cpu.tick(&mut self.bus);

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

    fn tick_joypad(&mut self) {
        self.bus.tick_joypad();
    }

    fn tick_peripherals_lockstep(&mut self) {
        for _ in 0..(self.state.cpu_lockstep_catchup.0) {
            self.tick_timer_divider();
            self.tick_oam_dma();
            self.tick_ppu();
        }
        self.state.cpu_lockstep_catchup.0 = 0;
    }

    pub fn tick(&mut self) -> MCycles {
        self.tick_joypad();
        match self.state.mode {
            GameBoyMode::Executing => {
                self.tick_cpu();
                let ticked_m_cycles = self.state.cpu_lockstep_catchup;
                self.handle_changes();
                self.tick_peripherals_lockstep();
                self.handle_changes();
                ticked_m_cycles
            },
            GameBoyMode::Stopped => todo!(),
            GameBoyMode::Halted => {
                if self.has_unhandled_interrupts() {
                    self.mode_transition(GameBoyMode::Executing);
                    self.tick()
                } else {
                    self.state.cpu_lockstep_catchup = MCycles(1);
                    self.tick_peripherals_lockstep();
                    self.handle_changes();
                    MCycles(1)
                }
            },
        }
    }

    fn has_unhandled_interrupts(&self) -> bool {
        self.cpu.interrupts_are_enabled() && self.bus.try_get_interrupt().is_some()
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
            GameBoyEvent::ChangeLcdPpuEnabled(enabled) => self.handle_change_lcd_ppu_enabled(enabled),
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
        self.ppu.handle_objects_disabled(self.state.cpu_lockstep_catchup.into())
    }

    fn handle_change_lcd_ppu_enabled(&mut self, enabled: bool) {
        self.bus.reset_ly();
        if enabled {
            self.ppu.enable();
        }
    }

    fn initiate_dma_transfer(&mut self, input: u8) {
        self.oam_dma.initiate_transfer(input);
        self.bus.handle_memory_map_event(MemoryMapEvent::StartOamDataTransfer);
    }

    fn end_dma_transfer(&mut self) {
        self.bus.handle_memory_map_event(MemoryMapEvent::EndOamDataTransfer);
    }

    fn mode_transition(&mut self, new_mode: GameBoyMode) {
        self.state.mode_transition(new_mode, &mut self.bus);
    }

    pub fn get_tile_map_images(&self) -> [TileMapImage; 2] {
        self.bus.get_tile_map_images()
    }
}

struct GameBoyState {
    mode: GameBoyMode,
    cpu_lockstep_catchup: MCycles,
}

impl GameBoyState {
    fn mode_transition(&mut self, new_mode: GameBoyMode, bus: &mut Bus) {
        match new_mode {
            GameBoyMode::Executing => (),
            GameBoyMode::Stopped => bus.reset_divider_register(),
            GameBoyMode::Halted => (),
        }
        self.mode = new_mode;
    }
}

impl Default for GameBoyState {
    fn default() -> Self {
        Self { cpu_lockstep_catchup: MCycles(0), mode: Default::default() }
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

impl std::ops::AddAssign for MCycles {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Add for MCycles {
    type Output = MCycles;

    fn add(self, rhs: Self) -> Self::Output {
        MCycles(self.0 + rhs.0)
    }
}

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
    ChangeLcdPpuEnabled(bool),
    ObjectsDisabled,
    ChangeObjectPriorityMode(crate::graphics::oam::PriorityMode),
    StartOamDmaTransfer(u8),
    ChangeBusAccessForPpuMode(PpuTickMode),
    EndOamDmaTransfer,
    Interrupt(Interrupt),
    IeTriggered, // Facilitates the delay between executing IE and actually enabling interrupts
    EnableInterrupts,
}
