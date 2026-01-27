use crate::{
    bus::MemoryTarget,
    game_boy::GameBoyStateChange,
    graphics::{lcd::LcdRegisters, oam::ObjectAttributeMemory, video_ram::VideoRam},
};
pub struct Ppu {
    mode_tracking: PpuModeTracker,
}

impl Ppu {
    const START_LINE_FOR_MODE_1: u8 = 144;
    const SCREEN_WIDTH: u8 = 160;
    pub const SCREEN_HEIGHT: u8 = 153;

    fn get_mode(&self) -> PpuTickMode {
        self.mode_tracking.mode
    }

    pub fn tick_ppu_enabled(
        &mut self,
        v_ram: &mut VideoRam,
        oam: &mut ObjectAttributeMemory,
        lcd_regs: &mut LcdRegisters,
    ) -> Vec<GameBoyStateChange> {
        let mut context = PpuOperationContext::new(self, v_ram, oam, lcd_regs);
        context.tick_dot_ppu_enabled()
    }
    pub fn tick_ppu_disabled(
        &mut self,
        v_ram: &mut VideoRam,
        oam: &mut ObjectAttributeMemory,
        lcd_regs: &mut LcdRegisters,
    ) -> Vec<GameBoyStateChange> {
        let mut context = PpuOperationContext::new(self, v_ram, oam, lcd_regs);
        context.tick_dot_ppu_disabled()
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self { mode_tracking: PpuModeTracker::default() }
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

    fn tick_oam_scan(&mut self) {}

    fn tick_drawing_pixels(&mut self) {}

    fn tick_horizontal_blank(&mut self) {}

    fn tick_vertical_blank(&mut self) {}

    pub fn tick_dot_ppu_enabled(&mut self) -> Vec<GameBoyStateChange> {
        match self.ppu.get_mode() {
            PpuTickMode::HorizontalBlank => self.tick_horizontal_blank(),
            PpuTickMode::VerticalBlank => self.tick_vertical_blank(),
            PpuTickMode::OamScan => self.tick_oam_scan(),
            PpuTickMode::DrawingPixels => self.tick_drawing_pixels(),
        };
        let mut return_vec = vec![];
        if let Some((new_mode, increment_ly)) = self.ppu.mode_tracking.process_tick(self.lcd_regs.get_ly()) {
            if increment_ly {
                return_vec.append(&mut self.lcd_regs.increment_ly());
            }
            return_vec.push(GameBoyStateChange::UpdatePpuMode(new_mode));
        }
        return_vec
    }

    pub fn tick_dot_ppu_disabled(&mut self) -> Vec<GameBoyStateChange> {
        if let Some((_, increment_ly)) = self.ppu.mode_tracking.process_tick(self.lcd_regs.get_ly()) {
            if increment_ly {
                return self.lcd_regs.increment_ly();
            }
        }
        vec![]
    }

    fn get_scanline(&mut self) -> Scanline {
        todo!()
    }

    fn get_window(&self) {
        todo!()
    }
}

struct Scanline {}

struct PpuModeTracker {
    mode: PpuTickMode,
    remaining_dots: Dots,
    extra_dots: Dots,
}

impl PpuModeTracker {
    fn process_tick(&mut self, ly: u8) -> Option<(PpuTickMode, bool)> {
        self.remaining_dots = self.remaining_dots.saturating_sub(1);
        if self.remaining_dots == 0 {
            Some((self.transition_to_next_mode(ly), self.mode.starts_new_line()))
        } else {
            None
        }
    }
    fn transition_to_next_mode(&mut self, ly: u8) -> PpuTickMode {
        self.mode = self.mode.get_next_mode(ly);
        self.remaining_dots = self.mode.get_default_length();

        // Extra time spent on Drawing Pixels is made up by less time in Horizontal Blank
        if self.mode == PpuTickMode::HorizontalBlank {
            self.remaining_dots -= self.extra_dots;
        }
        self.extra_dots = 0;

        self.mode
    }
}

impl Default for PpuModeTracker {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            remaining_dots: Default::default(),
            extra_dots: Default::default(),
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
    /// Tells whether or not transitioning into this state corresponds to a new line number (ie ly += 1)
    fn starts_new_line(&self) -> bool {
        match self {
            PpuTickMode::VerticalBlank | PpuTickMode::OamScan => true,
            PpuTickMode::HorizontalBlank | PpuTickMode::DrawingPixels => false,
        }
    }
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
                if ly < Ppu::START_LINE_FOR_MODE_1 {
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

struct FifoPixelFetcher {}

/// Unit of time for the PPU. 1 Dot basically is 1 TCycle
pub type Dots = u16;
