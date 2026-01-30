use std::sync::mpsc::Sender;

use crate::{
    bus::MemoryTarget,
    game_boy::{GameBoyEvent, notate_event},
    graphics::{
        lcd::LcdRegisters,
        oam::ObjectAttributeMemory,
        video_ram::{AccessMethod, Pixel, TargetTileMap, VideoRam},
    },
    helpers::{StackAllocQueue, WrappingIncrement},
    os_interface::graphics,
};

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
pub const SCREEN_SIZE: usize = SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize;
pub struct Ppu {
    mode_tracking: PpuModeTracker,
    sprite_fetcher: BackGroundFifo,
    background_fetcher: BackGroundFifo,
    pixel_buffer_sender: Sender<u32>,
    pushed_pixels_this_line: u8,
}

impl Ppu {
    pub const MAX_LY: u8 = 153;

    fn get_mode(&self) -> PpuTickMode {
        self.mode_tracking.mode
    }

    pub fn tick_ppu_enabled(
        &mut self,
        v_ram: &mut VideoRam,
        oam: &mut ObjectAttributeMemory,
        lcd_regs: &mut LcdRegisters,
    ) {
        let mut context = PpuOperationContext::new(self, v_ram, oam, lcd_regs);
        context.tick_dot_ppu_enabled()
    }
    pub fn tick_ppu_disabled(
        &mut self,
        v_ram: &mut VideoRam,
        oam: &mut ObjectAttributeMemory,
        lcd_regs: &mut LcdRegisters,
    ) {
        let mut context = PpuOperationContext::new(self, v_ram, oam, lcd_regs);
        context.tick_dot_ppu_disabled()
    }

    fn clear_queues(&mut self) {
        self.background_fetcher.clear_queue();
        self.sprite_fetcher.clear_queue();
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            mode_tracking: PpuModeTracker::default(),
            sprite_fetcher: BackGroundFifo::default(),
            background_fetcher: BackGroundFifo::default(),
            pixel_buffer_sender: graphics::start_window_thread(),
            pushed_pixels_this_line: 0,
        }
    }
}

pub struct PpuOperationContext<'a, 'b, 'c, 'd> {
    ppu: &'a mut Ppu,
    v_ram: &'b mut VideoRam,
    oam: &'c mut ObjectAttributeMemory,
    lcd_regs: &'d mut LcdRegisters,
}

impl<'a, 'b, 'c, 'd> PpuOperationContext<'a, 'b, 'c, 'd> {
    pub fn new(
        ppu: &'a mut Ppu,
        v_ram: &'b mut VideoRam,
        oam: &'c mut ObjectAttributeMemory,
        lcd_regs: &'d mut LcdRegisters,
    ) -> Self {
        Self { ppu, v_ram, oam, lcd_regs }
    }

    pub fn print_graphics_data(&self) {
        println!("Graphics Data:");
        let pixels = self.v_ram.get_logo();
        for pixel in pixels {
            self.ppu
                .pixel_buffer_sender
                .send((pixel.color_number * 64) as u32)
                .unwrap();
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    fn tick_oam_scan(&mut self) {}

    fn tick_drawing_pixels(&mut self) {
        // self.ppu.sprite_fetcher.tick(&self.lcd_regs, self.v_ram);
        self.ppu.background_fetcher.tick(&self.lcd_regs, self.v_ram);
        if self.ppu.background_fetcher.queue.length() >= 8 {
            if self.ppu.pushed_pixels_this_line < 160 {
                let pixel = self.ppu.background_fetcher.pop_pixel();
                self.ppu.pushed_pixels_this_line += 1;
                self.ppu.pixel_buffer_sender.send(pixel.into()).unwrap();
            }
        } else {
            self.ppu.mode_tracking.extra_dots += 1;
            self.ppu.mode_tracking.remaining_dots += 1;
        }
    }

    fn tick_horizontal_blank(&mut self) {}

    fn tick_vertical_blank(&mut self) {}

    pub fn tick_dot_ppu_enabled(&mut self) {
        match self.ppu.get_mode() {
            PpuTickMode::HorizontalBlank => self.tick_horizontal_blank(),
            PpuTickMode::VerticalBlank => self.tick_vertical_blank(),
            PpuTickMode::OamScan => self.tick_oam_scan(),
            PpuTickMode::DrawingPixels => self.tick_drawing_pixels(),
        };

        let result = self.ppu.mode_tracking.process_tick(self.lcd_regs.get_ly());

        if result.increment_ly {
            self.lcd_regs.increment_ly();
        }

        if let Some(mode) = result.new_mode {
            notate_event(GameBoyEvent::UpdatePpuMode(mode));
            if mode == PpuTickMode::DrawingPixels {
                self.ppu.clear_queues();
                self.ppu.background_fetcher.reset_for_new_scanline();
                // println!("Pushed {} pixels this line", self.ppu.pushed_pixels_this_line);
                self.ppu.pushed_pixels_this_line = 0;
            }
        }
    }

    pub fn tick_dot_ppu_disabled(&mut self) {
        let result = self.ppu.mode_tracking.process_tick(self.lcd_regs.get_ly());

        if result.increment_ly {
            self.lcd_regs.increment_ly();
        }
    }
}

struct Scanline {}

impl Scanline {
    const DOTS_PER_LINE: Dots = 456;
}

struct PpuModeTracker {
    mode: PpuTickMode,
    remaining_dots: Dots,
    extra_dots: Dots,
    dots_to_new_line: Dots,
}

struct PpuTickResult {
    increment_ly: bool,
    new_mode: Option<PpuTickMode>,
}

impl PpuModeTracker {
    fn process_tick(&mut self, ly: u8) -> PpuTickResult {
        let increment_ly = self.decrement_line_countdown();

        let effective_ly = if increment_ly { ly.wrapping_add(1) } else { ly };

        let new_mode = self.decrement_mode_dots(effective_ly);

        PpuTickResult { increment_ly, new_mode }
    }

    fn decrement_line_countdown(&mut self) -> bool {
        self.dots_to_new_line -= 1;
        if self.dots_to_new_line == 0 {
            self.dots_to_new_line = Scanline::DOTS_PER_LINE;
            true
        } else {
            false
        }
    }

    fn decrement_mode_dots(&mut self, ly: u8) -> Option<PpuTickMode> {
        self.remaining_dots = self.remaining_dots.saturating_sub(1);

        if self.remaining_dots == 0 {
            self.mode = self.mode.get_next_mode(ly);
            self.remaining_dots = self.mode.get_default_length();

            if self.mode == PpuTickMode::HorizontalBlank {
                self.remaining_dots -= self.extra_dots;
            }
            self.extra_dots = 0;

            Some(self.mode)
        } else {
            None
        }
    }
}

impl Default for PpuModeTracker {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            remaining_dots: Default::default(),
            extra_dots: Default::default(),
            dots_to_new_line: Scanline::DOTS_PER_LINE,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum PpuTickMode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    #[default]
    OamScan = 2,
    DrawingPixels = 3,
}

impl PpuTickMode {
    fn get_default_length(&self) -> Dots {
        match self {
            PpuTickMode::HorizontalBlank => 204,
            PpuTickMode::VerticalBlank => 4560,
            PpuTickMode::OamScan => 80,
            PpuTickMode::DrawingPixels => 172,
        }
    }
    fn get_next_mode(&self, ly: u8) -> Self {
        match self {
            PpuTickMode::HorizontalBlank => {
                if (ly + 1) < SCREEN_HEIGHT {
                    PpuTickMode::OamScan
                } else {
                    PpuTickMode::VerticalBlank
                }
            },
            PpuTickMode::VerticalBlank => PpuTickMode::OamScan,
            PpuTickMode::OamScan => PpuTickMode::DrawingPixels,
            PpuTickMode::DrawingPixels => PpuTickMode::HorizontalBlank,
        }
    }
    pub fn get_cpu_accessible_video_targets(&self) -> Vec<MemoryTarget> {
        match self {
            PpuTickMode::HorizontalBlank | PpuTickMode::VerticalBlank => {
                vec![MemoryTarget::VideoRam, MemoryTarget::ObjectAttributeMemory]
            },
            PpuTickMode::OamScan => vec![MemoryTarget::VideoRam],
            PpuTickMode::DrawingPixels => vec![],
        }
    }
    pub fn get_cpu_inaccessible_video_targets(&self) -> Vec<MemoryTarget> {
        match self {
            PpuTickMode::HorizontalBlank | PpuTickMode::VerticalBlank => vec![],
            PpuTickMode::OamScan => vec![MemoryTarget::ObjectAttributeMemory],
            PpuTickMode::DrawingPixels => vec![MemoryTarget::VideoRam, MemoryTarget::ObjectAttributeMemory],
        }
    }
}

impl From<u8> for PpuTickMode {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Self::HorizontalBlank,
            1 => Self::VerticalBlank,
            2 => Self::OamScan,
            3 => Self::DrawingPixels,
            _ => unreachable!(),
        }
    }
}
impl From<PpuTickMode> for u8 {
    fn from(mode: PpuTickMode) -> Self {
        match mode {
            PpuTickMode::HorizontalBlank => 0,
            PpuTickMode::VerticalBlank => 1,
            PpuTickMode::OamScan => 2,
            PpuTickMode::DrawingPixels => 3,
        }
    }
}

#[repr(u8)]
#[derive(Default, Copy, Clone)]
pub enum Color {
    #[default]
    Lightest = 0,
    Lighter,
    Darker,
    Darkest,
}

impl From<u8> for Color {
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
struct OamPixel {
    color: Color,
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

    fn current_tile_is_window_tile(&self, lcd: &LcdRegisters) -> bool {
        lcd.coordinate_in_window(self.pixels_popped)
    }

    fn get_tile_location(&mut self, lcd: &LcdRegisters) -> (u8, u8, u8) {
        if self.current_tile_is_window_tile(lcd) {
            let window_x = self.pixels_popped.saturating_sub(lcd.get_wx().saturating_sub(7));
            let window_y = lcd.get_ly() - lcd.get_wy();

            let column = window_x / 8;
            let row = window_y / 8;
            let pixel_row = window_y % 8;

            (column, row, pixel_row)
        } else {
            let calced_y = (lcd.get_ly() + lcd.get_scy()) & 0xFF;
            (
                ((lcd.get_scx() / 8) + self.tiles_fetched) & 0x1F,
                calced_y >> 3,
                calced_y & 7,
            )
        }
    }

    fn tick(&mut self, lcd: &LcdRegisters, v_ram: &VideoRam) {
        // first check if we should act this cycle.
        // This is used to force the first four modes to take two cycles
        if self.mode.should_sleep() {
            self.mode.sleep();
            return;
        }

        match self.mode {
            FifoMode::GetTile { sleep_cycle: _ } => {
                let access_method = lcd.get_background_window_tiles_address_mode();
                let map = lcd.get_target_tilemap(self.pixels_popped);

                let (column, row, in_sprite_row) = self.get_tile_location(lcd);
                let tile_number = v_ram.get_tile_index_from_map(&map, row, column);

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
                self.mode = FifoMode::Sleep { sleep_cycle: true, low_byte, high_byte }
            },
            FifoMode::Sleep { sleep_cycle: _, low_byte, high_byte } => {
                self.mode = FifoMode::Push { low_byte, high_byte }
            },
            FifoMode::Push { low_byte, high_byte } => {
                for byte in Pixel::from_bytes(low_byte, high_byte) {
                    self.queue.push(byte);
                }
                self.mode = FifoMode::GetTile { sleep_cycle: true };
                self.tiles_fetched = self.tiles_fetched.wrapping_increment(32);
            },
        }
    }

    fn pop_pixel(&mut self) -> Pixel {
        self.pixels_popped = self.pixels_popped.wrapping_increment(SCREEN_WIDTH);
        self.queue.pop_unchecked()
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
    Sleep {
        sleep_cycle: bool,
        low_byte: u8,
        high_byte: u8,
    },
    Push {
        low_byte: u8,
        high_byte: u8,
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
            }
            | FifoMode::Sleep { sleep_cycle, low_byte: _, high_byte: _ } => *sleep_cycle,
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
            }
            | FifoMode::Sleep { sleep_cycle, low_byte: _, high_byte: _ } => *sleep_cycle = false,
            _ => (),
        }
    }
}

#[derive(Default)]
struct FifoModeData {
    target_tile_map: TargetTileMap, // which of the two tile maps the target is in
    access_method: AccessMethod,    // The method that we access videoram with to get the tile
    tile_address_row: u8,           // The row in the tile map the tile's address is at
    tile_address_column: u8,        // the column in the tile map the tile's address is at
    tile_address: u8,               // the address of the tile found within the TileMap
    tile_pixel_row: u8,             // the row inside of the tile that holds the 8 pixels we want
}

/// Unit of time for the PPU. 1 Dot basically is 1 TCycle
pub type Dots = u16;
