#[cfg(not(feature = "headless"))]
use crate::io_devices::joypad_input::ButtonInput;
#[cfg(not(feature = "headless"))]
use crate::os_interface::window::SenderFrameHandle;

use crate::{
    bus::{Bus, MemoryMapEvent},
    cartridge::cartridge::Cartridge,
    graphics::{
        ppu::{Ppu, PpuMode, PpuOperationContext},
        video_ram::TileMapImage,
    },
    helpers::Bit,
    io_devices::{dma::OamDma, interrupts::Interrupt},
    processor::cpu::Cpu,
};

use std::cell::RefCell;

thread_local! {
    static GAMEBOY_EVENTS: RefCell<Vec<GameBoyEvent>> = const { RefCell::new(Vec::new()) };
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
    ppu: Ppu,
    bus: Bus,
    oam_dma: OamDma,
    cpu: Cpu,
}

impl GameBoy {
    #[cfg(not(feature = "headless"))]
    pub fn new(frame_handle: SenderFrameHandle, button_input: ButtonInput) -> Self {
        Self {
            ppu: Ppu::new(frame_handle),
            state: Default::default(),
            cpu: Default::default(),
            bus: Bus::new(button_input),
            oam_dma: Default::default(),
        }
    }

    #[cfg(feature = "headless")]
    pub fn new() -> Self {
        Self {
            ppu: Ppu::new(),
            state: Default::default(),
            bus: Bus::new(),
            oam_dma: Default::default(),
            cpu: Cpu::default(),
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge)
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

    fn tick_serial(&mut self) {
        self.bus.tick_serial();
    }

    fn tick_cpu(&mut self) {
        self.cpu.tick(&mut self.bus)
    }

    pub fn tick(&mut self) {
        self.tick_ppu();
        self.tick_cpu();
        self.tick_joypad();
        self.tick_timer_divider();
        self.tick_oam_dma();
        self.tick_serial();
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
            GameBoyEvent::ChangeBusAccessForPpuMode(mode) => {
                self.bus.handle_memory_map_event(MemoryMapEvent::UpdatePpuMode(mode))
            },
            GameBoyEvent::ChangeLcdPpuEnabled(enabled) => self.handle_change_lcd_ppu_enabled(enabled),
            GameBoyEvent::Interrupt(interrupt) => self.bus.raise_interrupt_flag(&interrupt),
            GameBoyEvent::ObjectsDisabled => self.handle_objects_disabled(),
            GameBoyEvent::TriedRunningIllegalInstruction => {
                todo!()
            },
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
        let (v_ram, oam, lcd) = self.bus.get_ppu_context_mem();
        let mut ppu_context = PpuOperationContext::new(&mut self.ppu, v_ram, oam, lcd);
        if enabled {
            ppu_context.enable();
        } else {
            ppu_context.disable();
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
        match new_mode {
            GameBoyMode::Executing => (),
            GameBoyMode::Stopped => self.bus.reset_divider_register(),
            GameBoyMode::Halted => (),
        }

        self.state.mode = new_mode;
    }

    pub fn get_tile_map_images(&self) -> [TileMapImage; 2] {
        self.bus.get_tile_map_images()
    }
    pub fn get_serial_output(&self) -> Vec<&Bit> {
        self.bus.get_serial_output()
    }
    pub fn peek_mem(&self, address: u16) -> u8 {
        self.bus.peek(address)
    }
    pub fn get_pc(&self) -> u16 {
        self.cpu.get_pc()
    }
}

struct GameBoyState {
    mode: GameBoyMode,
    cpu_lockstep_catchup: MCycles,
}

impl GameBoyState {}

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
    ChangeBusAccessForPpuMode(PpuMode),
    EndOamDmaTransfer,
    Interrupt(Interrupt),
    TriedRunningIllegalInstruction,
}
