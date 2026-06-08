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

/// Events that happen in a tick and need to be processed by the top level system
pub type EventQueue = Vec<GameBoyEvent>;

pub struct GameBoy {
    state: GameBoyState,
    ppu: Ppu,
    bus: Bus,
    oam_dma: OamDma,
    cpu: Cpu,
    events: EventQueue,
}

#[cfg(feature = "headless")]
impl Default for GameBoy {
    fn default() -> Self {
        Self::new()
    }
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
            events: EventQueue::new(),
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
            events: EventQueue::new(),
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge)
    }

    fn tick_oam_dma(&mut self) {
        self.oam_dma.tick(&mut self.bus, &mut self.events);
    }

    fn tick_ppu(&mut self) {
        let (v_ram, oam, lcd_regs) = self.bus.get_ppu_context_mem();

        self.ppu.tick(v_ram, oam, lcd_regs, &mut self.events)
    }

    fn tick_timer_divider(&mut self) {
        self.bus.tick_timer_divider(&mut self.events);
    }

    fn tick_joypad(&mut self) {
        self.bus.tick_joypad(&mut self.events);
    }

    fn tick_serial(&mut self) {
        self.bus.tick_serial(&mut self.events);
    }

    fn tick_cpu(&mut self) {
        self.cpu.tick(&mut self.bus, &mut self.events)
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
        let pending = std::mem::take(&mut self.events);
        for change in pending {
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
        self.ppu.handle_objects_disabled()
    }

    fn handle_change_lcd_ppu_enabled(&mut self, enabled: bool) {
        self.bus.reset_ly(&mut self.events);
        let (v_ram, oam, lcd) = self.bus.get_ppu_context_mem();
        let mut ppu_context = PpuOperationContext::new(&mut self.ppu, v_ram, oam, lcd, &mut self.events);
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
    pub fn serial_output_bit_count(&self) -> u64 {
        self.bus.serial_output_bit_count()
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
}

impl GameBoyState {}

impl Default for GameBoyState {
    fn default() -> Self {
        Self { mode: Default::default() }
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
    ChangeObjectPriorityMode(crate::graphics::oam::PriorityMode), // CGB: OBJ priority mode (0xFF6C)
    StartOamDmaTransfer(u8),
    ChangeBusAccessForPpuMode(PpuMode),
    EndOamDmaTransfer,
    Interrupt(Interrupt),
    TriedRunningIllegalInstruction,
}
