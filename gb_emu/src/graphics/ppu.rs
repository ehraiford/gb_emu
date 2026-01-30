use std::sync::mpsc::Sender;

use crate::{
    bus::MemoryTarget,
    game_boy::{GameBoyEvent, notate_event},
    graphics::{
        lcd::LcdRegisters,
        oam::ObjectAttributeMemory,
        video_ram::{AccessMethod, Pixel, VideoRam},
    },
    helpers::StackAllocQueue,
    io_devices::interrupts::Interrupt,
    os_interface::graphics,
};

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
pub const SCREEN_SIZE: usize = SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize;
const DOTS_PER_LINE: Dots = 456;

pub struct Ppu {
    mode_tracking: PpuModeTracker,
    sprite_fetcher: BackGroundFifo,
    background_fetcher: BackGroundFifo,
    pixel_buffer_sender: Sender<u32>,
}

impl Ppu {
    pub fn enable(&mut self) {
        self.reset_for_new_scanline();
        self.mode_tracking = Default::default();
    }

    fn get_mode(&self) -> &PpuTickMode {
        &self.mode_tracking.mode
    }

    pub fn tick(&mut self, v_ram: &mut VideoRam, oam: &mut ObjectAttributeMemory, lcd_regs: &mut LcdRegisters) {
        let mut context = PpuOperationContext::new(self, v_ram, oam, lcd_regs);
        context.tick_dot_ppu_enabled()
    }

    fn reset_for_new_scanline(&mut self) {
        self.background_fetcher.clear_queue();
        self.sprite_fetcher.clear_queue();
        self.background_fetcher.reset_for_new_scanline();
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            mode_tracking: PpuModeTracker::default(),
            sprite_fetcher: BackGroundFifo::default(),
            background_fetcher: BackGroundFifo::default(),
            pixel_buffer_sender: graphics::start_window_thread(),
        }
    }
}

pub struct PpuOperationContext<'a, 'b, 'c, 'd> {
    ppu: &'a mut Ppu,
    v_ram: &'b mut VideoRam,
    _oam: &'c mut ObjectAttributeMemory,
    lcd_regs: &'d mut LcdRegisters,
}

impl<'a, 'b, 'c, 'd> PpuOperationContext<'a, 'b, 'c, 'd> {
    pub fn new(
        ppu: &'a mut Ppu,
        v_ram: &'b mut VideoRam,
        oam: &'c mut ObjectAttributeMemory,
        lcd_regs: &'d mut LcdRegisters,
    ) -> Self {
        Self { ppu, v_ram, _oam: oam, lcd_regs }
    }

    fn tick_oam_scan(&mut self) {}

    fn tick_drawing_pixels(&mut self, mut pixels_to_ignore: u8, mut pixels_to_push: u8) -> (u8, u8) {
        self.ppu.background_fetcher.tick(&self.lcd_regs, self.v_ram);

        if self.ppu.background_fetcher.queue.length() > 0 {
            if pixels_to_push > 0 {
                if pixels_to_ignore > 0 {
                    self.ppu.background_fetcher.pop_ignored_pixel();
                    pixels_to_ignore -= 1;
                } else {
                    let pixel = self.ppu.background_fetcher.pop_pixel();
                    self.ppu.pixel_buffer_sender.send(pixel.into()).unwrap();
                    pixels_to_push -= 1;
                }
            }
        }
        (pixels_to_ignore, pixels_to_push)
    }

    fn tick_horizontal_blank(&mut self) {}

    fn tick_vertical_blank(&mut self) {}

    pub fn tick_dot_ppu_enabled(&mut self) {
        match self.ppu.get_mode() {
            PpuTickMode::HorizontalBlank => self.tick_horizontal_blank(),
            PpuTickMode::VerticalBlank => self.tick_vertical_blank(),
            PpuTickMode::OamScan { remaining_cycles: _ } => self.tick_oam_scan(),
            PpuTickMode::DrawingPixels { pixels_left_to_ignore, pixels_left_to_push } => {
                let pixels_to_ignore = *pixels_left_to_ignore;
                let pixels_to_push = *pixels_left_to_push;
                let (pixels_to_ignore, pixels_to_push) = self.tick_drawing_pixels(pixels_to_ignore, pixels_to_push);
                self.ppu.mode_tracking.mode = PpuTickMode::DrawingPixels {
                    pixels_left_to_push: pixels_to_push,
                    pixels_left_to_ignore: pixels_to_ignore,
                }
            },
        };

        let result = self
            .ppu
            .mode_tracking
            .process_tick(self.lcd_regs.get_ly(), self.lcd_regs.get_scx());

        if result.increment_ly {
            self.lcd_regs.increment_ly();
            self.ppu.reset_for_new_scanline();
        }

        if let Some(mode) = result.new_mode {
            notate_event(GameBoyEvent::UpdatePpuMode(mode));
            if let PpuTickMode::OamScan { remaining_cycles: _ } = mode {
                // println!("Pushed {} pixels this line", self.ppu.pushed_pixels_this_line);
            }
        }
    }
}

struct PpuModeTracker {
    mode: PpuTickMode,
    remaining_dots_in_line: Dots,
}

struct PpuTickResult {
    increment_ly: bool,
    new_mode: Option<PpuTickMode>,
}

impl PpuModeTracker {
    fn process_tick_horizontal_blank(&mut self, ly: u8) -> PpuTickResult {
        if self.remaining_dots_in_line == 0 {
            self.remaining_dots_in_line = DOTS_PER_LINE;
            if ly < SCREEN_HEIGHT {
                let new_mode = PpuTickMode::OamScan { remaining_cycles: 80 };
                self.mode = new_mode;
                PpuTickResult { increment_ly: true, new_mode: Some(new_mode) }
            } else {
                notate_event(GameBoyEvent::Interrupt(Interrupt::VBlank));
                let new_mode = PpuTickMode::VerticalBlank;
                self.mode = new_mode;
                PpuTickResult { increment_ly: true, new_mode: Some(new_mode) }
            }
        } else {
            PpuTickResult { increment_ly: false, new_mode: None }
        }
    }

    fn process_tick_vertical_blank(&mut self, ly: u8) -> PpuTickResult {
        match (self.remaining_dots_in_line == 0, ly == LcdRegisters::MAX_LY) {
            (true, true) => {
                self.remaining_dots_in_line = DOTS_PER_LINE;
                let new_mode = PpuTickMode::OamScan { remaining_cycles: 80 };
                self.mode = new_mode;
                PpuTickResult { increment_ly: true, new_mode: Some(new_mode) }
            },
            (true, false) => {
                self.remaining_dots_in_line = DOTS_PER_LINE;
                let new_mode = PpuTickMode::VerticalBlank;
                notate_event(GameBoyEvent::Interrupt(Interrupt::VBlank));
                self.mode = new_mode;
                PpuTickResult { increment_ly: true, new_mode: Some(new_mode) }
            },
            _ => PpuTickResult { increment_ly: false, new_mode: None },
        }
    }

    fn process_tick_oam_scan(&mut self, mut remaining_cycles: u8, scx: u8) -> PpuTickResult {
        remaining_cycles -= 1;
        if remaining_cycles == 0 {
            let new_mode = PpuTickMode::DrawingPixels { pixels_left_to_push: 160, pixels_left_to_ignore: scx % 8 };
            self.mode = new_mode;
            PpuTickResult { increment_ly: false, new_mode: Some(new_mode) }
        } else {
            self.mode = PpuTickMode::OamScan { remaining_cycles };
            PpuTickResult { increment_ly: false, new_mode: None }
        }
    }

    fn process_tick_drawing_pixels(&mut self, pixels_left_to_push: u8) -> PpuTickResult {
        if self.remaining_dots_in_line == 0 {
            panic!();
        }
        if pixels_left_to_push == 0 {
            self.mode = PpuTickMode::HorizontalBlank;
            PpuTickResult {
                increment_ly: false,
                new_mode: Some(PpuTickMode::HorizontalBlank),
            }
        } else {
            PpuTickResult { increment_ly: false, new_mode: None }
        }
    }

    fn process_tick(&mut self, ly: u8, scx: u8) -> PpuTickResult {
        self.remaining_dots_in_line -= 1;

        match self.mode {
            PpuTickMode::HorizontalBlank => self.process_tick_horizontal_blank(ly),
            PpuTickMode::VerticalBlank => self.process_tick_vertical_blank(ly),
            PpuTickMode::OamScan { remaining_cycles } => self.process_tick_oam_scan(remaining_cycles, scx),
            PpuTickMode::DrawingPixels { pixels_left_to_push, pixels_left_to_ignore: _ } => {
                self.process_tick_drawing_pixels(pixels_left_to_push)
            },
        }
    }
}

impl Default for PpuModeTracker {
    fn default() -> Self {
        Self {
            mode: PpuTickMode::OamScan { remaining_cycles: 80 },
            remaining_dots_in_line: DOTS_PER_LINE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PpuTickMode {
    HorizontalBlank,
    VerticalBlank,
    OamScan {
        remaining_cycles: u8,
    },
    DrawingPixels {
        pixels_left_to_push: u8,
        pixels_left_to_ignore: u8,
    },
}

impl PpuTickMode {
    pub fn get_cpu_accessible_video_targets(&self) -> Vec<MemoryTarget> {
        match self {
            PpuTickMode::HorizontalBlank | PpuTickMode::VerticalBlank => {
                vec![MemoryTarget::VideoRam, MemoryTarget::ObjectAttributeMemory]
            },
            PpuTickMode::OamScan { remaining_cycles: _ } => vec![MemoryTarget::VideoRam],
            PpuTickMode::DrawingPixels { pixels_left_to_ignore: _, pixels_left_to_push: _ } => vec![],
        }
    }
    pub fn get_cpu_inaccessible_video_targets(&self) -> Vec<MemoryTarget> {
        match self {
            PpuTickMode::HorizontalBlank | PpuTickMode::VerticalBlank => vec![],
            PpuTickMode::OamScan { remaining_cycles: _ } => vec![MemoryTarget::ObjectAttributeMemory],
            PpuTickMode::DrawingPixels { pixels_left_to_ignore: _, pixels_left_to_push: _ } => {
                vec![MemoryTarget::VideoRam, MemoryTarget::ObjectAttributeMemory]
            },
        }
    }
}

impl Default for PpuTickMode {
    fn default() -> Self {
        Self::OamScan { remaining_cycles: 80 }
    }
}

impl From<PpuTickMode> for u8 {
    fn from(mode: PpuTickMode) -> Self {
        match mode {
            PpuTickMode::HorizontalBlank => 0,
            PpuTickMode::VerticalBlank => 1,
            PpuTickMode::OamScan { remaining_cycles: _ } => 2,
            PpuTickMode::DrawingPixels { pixels_left_to_ignore: _, pixels_left_to_push: _ } => 3,
        }
    }
}

#[repr(u8)]
#[derive(Default, Copy, Clone)]
pub enum _Color {
    #[default]
    Lightest = 0,
    Lighter,
    Darker,
    Darkest,
}

impl From<u8> for _Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Lightest,
            1 => Self::Lighter,
            2 => Self::Darker,
            3 => Self::Darkest,
            _ => unreachable!(),
        }
    }
}

#[derive(Default, Copy, Clone)]
struct _OamPixel {
    color: _Color,
    palette: u8,
    background_priority: bool,
}

struct BackGroundFifo {
    queue: StackAllocQueue<Pixel, 16>,
    mode: FifoMode,
    pixels_popped: u8,
    tiles_fetched: u8,
}

impl BackGroundFifo {
    fn reset_for_new_scanline(&mut self) {
        self.queue.clear_queue();
        self.mode = FifoMode::GetTile { sleep_cycle: true };
        self.pixels_popped = 0;
        self.tiles_fetched = 0;
    }

    fn current_tile_is_window_tile(&self, lcd: &LcdRegisters, fetcher_x: u8) -> bool {
        lcd.window_enabled() && lcd.coordinate_in_window(fetcher_x)
    }

    fn get_tile_location(&mut self, lcd: &LcdRegisters) -> (u8, u8, u8) {
        // Use the fetcher's progress (tiles_fetched * 8) rather than the pixels already on screen.
        // This ensures the fetcher switches to the Window tilemap at the correct dot.
        let fetcher_x = self.tiles_fetched * 8;

        if self.current_tile_is_window_tile(lcd, fetcher_x) {
            let window_x = fetcher_x.saturating_sub(lcd.get_wx().saturating_sub(7));
            let window_y = lcd.get_ly() - lcd.get_wy();

            (window_x / 8, window_y / 8, window_y % 8)
        } else {
            let scx = lcd.get_scx();
            let calced_y = (lcd.get_ly() + lcd.get_scy()) & 0xFF;
            let fetcher_bg_x = (scx.wrapping_add(fetcher_x)) & 0xFF;

            (fetcher_bg_x / 8, calced_y >> 3, calced_y & 7)
        }
    }

    fn tick(&mut self, lcd: &LcdRegisters, v_ram: &VideoRam) {
        // first check if we should act this cycle.
        // This is used to force the modes to take two cycles
        if self.mode.should_sleep() {
            self.mode.sleep();
            return;
        }

        match self.mode {
            FifoMode::GetTile { sleep_cycle: _ } => {
                let access_method = lcd.get_background_window_tiles_address_mode();

                // DECISION: Use fetcher X (tiles_fetched * 8) to choose the tilemap
                let fetcher_x = self.tiles_fetched * 8;
                let map = lcd.get_target_tilemap(fetcher_x);

                let (column, row, in_sprite_row) = self.get_tile_location(lcd);
                let tile_number = v_ram.get_tile_index_from_map(&map, row, column);

                // Increment tiles_fetched AFTER fetching so the first tile is 0
                self.tiles_fetched = self.tiles_fetched.wrapping_add(1);

                self.mode = FifoMode::GetTileDataLow {
                    sleep_cycle: true,
                    access_method,
                    tile_number,
                    byte_number: in_sprite_row << 1,
                }
            },
            FifoMode::GetTileDataLow { sleep_cycle: _, access_method, tile_number, byte_number } => {
                let low_byte = v_ram.get_tile_byte(access_method, tile_number, byte_number);
                self.mode = FifoMode::GetTileDataHigh {
                    sleep_cycle: true,
                    access_method,
                    tile_number: tile_number,
                    byte_number: byte_number + 1,
                    low_byte,
                }
            },
            FifoMode::GetTileDataHigh {
                sleep_cycle: _,
                access_method,
                tile_number,
                low_byte,
                byte_number,
            } => {
                let high_byte = v_ram.get_tile_byte(access_method, tile_number, byte_number);
                self.try_push_pixels(low_byte, high_byte);
            },
        }
    }

    fn try_push_pixels(&mut self, low_byte: u8, high_byte: u8) {
        if self.queue.length() > 8 {
            return; // we stall if there isn't enough space in the queue
        }
        for byte in Pixel::from_bytes(low_byte, high_byte) {
            self.queue.push(byte);
        }
        self.mode = FifoMode::GetTile { sleep_cycle: true };
    }

    fn pop_pixel(&mut self) -> Pixel {
        let pixel = self.queue.pop_unchecked();
        // pixels_popped should justbe a simple counter from 0..160
        self.pixels_popped += 1;
        pixel
    }
    fn pop_ignored_pixel(&mut self) {
        self.queue.pop_unchecked();
    }

    fn clear_queue(&mut self) {
        self.queue.clear_queue();
    }
}

impl Default for BackGroundFifo {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            mode: Default::default(),
            pixels_popped: 0,
            tiles_fetched: 0,
        }
    }
}

#[derive(Debug)]
enum FifoMode {
    GetTile {
        sleep_cycle: bool,
    },
    GetTileDataLow {
        sleep_cycle: bool,
        access_method: AccessMethod,
        tile_number: u8,
        byte_number: u8,
    },
    GetTileDataHigh {
        sleep_cycle: bool,
        access_method: AccessMethod,
        tile_number: u8,
        byte_number: u8,
        low_byte: u8,
    },
}

impl Default for FifoMode {
    fn default() -> Self {
        Self::GetTile { sleep_cycle: true }
    }
}

impl FifoMode {
    fn should_sleep(&self) -> bool {
        match self {
            FifoMode::GetTile { sleep_cycle }
            | FifoMode::GetTileDataLow {
                sleep_cycle,
                access_method: _,
                tile_number: _,
                byte_number: _,
            }
            | FifoMode::GetTileDataHigh {
                sleep_cycle,
                access_method: _,
                tile_number: _,
                low_byte: _,
                byte_number: _,
            } => *sleep_cycle,
            _ => false,
        }
    }
    fn sleep(&mut self) {
        match self {
            FifoMode::GetTile { sleep_cycle }
            | FifoMode::GetTileDataLow {
                sleep_cycle,
                access_method: _,
                tile_number: _,
                byte_number: _,
            }
            | FifoMode::GetTileDataHigh {
                sleep_cycle,
                access_method: _,
                tile_number: _,
                low_byte: _,
                byte_number: _,
            } => *sleep_cycle = false,
        }
    }
}

/// Unit of time for the PPU. 1 Dot basically is 1 TCycle
pub type Dots = u16;
