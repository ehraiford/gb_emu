use std::sync::mpsc::Sender;

use crate::{
    bus::MemoryTarget,
    game_boy::{GameBoyEvent, TCycles, notate_event},
    graphics::{
        lcd::Lcd,
        oam::{ObjectAttributeMemory, ObjectAttributes},
        pixel_fetchers::PixelFetchers,
        video_ram::{RawPixel, VideoRam},
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
    pixel_fetchers: PixelFetchers,
    pixel_buffer_sender: Sender<u32>,
    oam_scanner: OamScanner,
}

impl Ppu {
    pub fn enable(&mut self) {
        self.reset_for_new_frame();
        self.mode_tracking = Default::default();
    }

    fn get_mode(&self) -> &PpuTickMode {
        &self.mode_tracking.mode
    }

    pub fn tick(&mut self, v_ram: &mut VideoRam, oam: &mut ObjectAttributeMemory, lcd: &mut Lcd) {
        let mut context = PpuOperationContext::new(self, v_ram, oam, lcd);
        context.tick()
    }

    fn reset_for_new_scanline(&mut self) {
        self.oam_scanner.reset_for_new_scanline();
        self.pixel_fetchers.reset_for_new_scanline();
    }

    pub fn handle_objects_disabled(&mut self, instruction_length: TCycles) {
        self.pixel_fetchers.handle_objects_disabled(instruction_length);
    }

    fn reset_for_new_frame(&mut self) {
        self.reset_for_new_scanline();
        self.pixel_fetchers.reset_window_y();
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            mode_tracking: PpuModeTracker::default(),
            pixel_buffer_sender: graphics::start_window_thread(),
            oam_scanner: OamScanner::default(),
            pixel_fetchers: PixelFetchers::default(),
        }
    }
}

pub struct PpuOperationContext<'a, 'b, 'c, 'd> {
    ppu: &'a mut Ppu,
    v_ram: &'b mut VideoRam,
    oam: &'c mut ObjectAttributeMemory,
    lcd: &'d mut Lcd,
}

impl<'a, 'b, 'c, 'd> PpuOperationContext<'a, 'b, 'c, 'd> {
    pub fn new(
        ppu: &'a mut Ppu,
        v_ram: &'b mut VideoRam,
        oam: &'c mut ObjectAttributeMemory,
        lcd: &'d mut Lcd,
    ) -> Self {
        Self { ppu, v_ram, oam, lcd }
    }

    fn tick_oam_scan(&mut self) {
        self.ppu.oam_scanner.tick(self.oam, self.lcd);
    }

    fn tick_drawing_pixels(&mut self, mut pixels_left_to_ignore: u8, mut pixels_left_to_push: u8) {
        if let Some(pixel) = self.ppu.pixel_fetchers.tick(self.lcd, self.v_ram) {
            if pixels_left_to_ignore > 0 {
                pixels_left_to_ignore -= 1;
            } else {
                self.ppu
                    .pixel_buffer_sender
                    .send(RawPixel { color_number: pixel.color }.into())
                    .unwrap();
                pixels_left_to_push -= 1;
            }
        }

        self.ppu.mode_tracking.mode = PpuTickMode::DrawingPixels { pixels_left_to_push, pixels_left_to_ignore };
    }

    fn tick_horizontal_blank(&mut self) {}

    fn tick_vertical_blank(&mut self) {}

    pub fn tick(&mut self) {
        match self.ppu.get_mode() {
            PpuTickMode::HorizontalBlank => self.tick_horizontal_blank(),
            PpuTickMode::VerticalBlank => self.tick_vertical_blank(),
            PpuTickMode::OamScan { completed_cycles: _ } => self.tick_oam_scan(),
            PpuTickMode::DrawingPixels { pixels_left_to_ignore, pixels_left_to_push } => {
                self.tick_drawing_pixels(*pixels_left_to_ignore, *pixels_left_to_push)
            },
        };

        let result = self
            .ppu
            .mode_tracking
            .process_tick(self.lcd.get_ly(), self.lcd.get_scx());

        if result.increment_ly {
            self.go_to_next_line();
        }

        if let Some(mode) = result.new_mode {
            notate_event(GameBoyEvent::ChangeBusAccessForPpuMode(mode));
            if let PpuTickMode::OamScan { completed_cycles: _ } = mode {
                // println!("Pushed {} pixels this line", self.ppu.pushed_pixels_this_line);
            }
        }
    }

    fn go_to_next_line(&mut self) {
        match self.lcd.increment_ly() == 0 {
            true => self.ppu.reset_for_new_scanline(),
            false => self.ppu.reset_for_new_scanline(),
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
            if ly < SCREEN_HEIGHT - 1 {
                let new_mode = PpuTickMode::OamScan { completed_cycles: 0 };
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
        if self.remaining_dots_in_line == 0 {
            self.remaining_dots_in_line = DOTS_PER_LINE;

            if ly >= Lcd::MAX_LY {
                let new_mode = PpuTickMode::OamScan { completed_cycles: 0 };
                self.mode = new_mode;
                PpuTickResult { increment_ly: true, new_mode: Some(new_mode) }
            } else {
                let new_mode = PpuTickMode::VerticalBlank;
                self.mode = new_mode;
                PpuTickResult { increment_ly: true, new_mode: Some(new_mode) }
            }
        } else {
            PpuTickResult { increment_ly: false, new_mode: None }
        }
    }

    fn process_tick_oam_scan(&mut self, mut completed_cycles: u8, scx: u8) -> PpuTickResult {
        completed_cycles += 1;
        if completed_cycles == 80 {
            let new_mode = PpuTickMode::DrawingPixels { pixels_left_to_push: 160, pixels_left_to_ignore: scx % 8 };
            self.mode = new_mode;
            PpuTickResult { increment_ly: false, new_mode: Some(new_mode) }
        } else {
            self.mode = PpuTickMode::OamScan { completed_cycles };
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
            PpuTickMode::OamScan { completed_cycles } => self.process_tick_oam_scan(completed_cycles, scx),
            PpuTickMode::DrawingPixels { pixels_left_to_push, pixels_left_to_ignore: _ } => {
                self.process_tick_drawing_pixels(pixels_left_to_push)
            },
        }
    }
}

impl Default for PpuModeTracker {
    fn default() -> Self {
        Self {
            mode: PpuTickMode::OamScan { completed_cycles: 0 },
            remaining_dots_in_line: DOTS_PER_LINE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PpuTickMode {
    HorizontalBlank,
    VerticalBlank,
    OamScan {
        completed_cycles: u8,
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
            PpuTickMode::OamScan { completed_cycles: _ } => vec![MemoryTarget::VideoRam],
            PpuTickMode::DrawingPixels { pixels_left_to_ignore: _, pixels_left_to_push: _ } => vec![],
        }
    }
    pub fn get_cpu_inaccessible_video_targets(&self) -> Vec<MemoryTarget> {
        match self {
            PpuTickMode::HorizontalBlank | PpuTickMode::VerticalBlank => vec![],
            PpuTickMode::OamScan { completed_cycles: _ } => vec![MemoryTarget::ObjectAttributeMemory],
            PpuTickMode::DrawingPixels { pixels_left_to_ignore: _, pixels_left_to_push: _ } => {
                vec![MemoryTarget::VideoRam, MemoryTarget::ObjectAttributeMemory]
            },
        }
    }
}

impl Default for PpuTickMode {
    fn default() -> Self {
        Self::OamScan { completed_cycles: 0 }
    }
}

impl From<PpuTickMode> for u8 {
    fn from(mode: PpuTickMode) -> Self {
        match mode {
            PpuTickMode::HorizontalBlank => 0,
            PpuTickMode::VerticalBlank => 1,
            PpuTickMode::OamScan { completed_cycles: _ } => 2,
            PpuTickMode::DrawingPixels { pixels_left_to_ignore: _, pixels_left_to_push: _ } => 3,
        }
    }
}

#[derive(Default)]
struct OamScanner {
    objects_on_this_line: ObjectsOnThisLine,
    scanner_stage: OamScannerStage,
}

impl OamScanner {
    fn tick(&mut self, oam: &ObjectAttributeMemory, lcd: &Lcd) {
        if self.objects_on_this_line.is_at_capacity() {
            return;
        }
        match self.scanner_stage {
            OamScannerStage::Scanning { object_number } => self.get_object_attributes(oam, object_number),
            OamScannerStage::Comparing { object_number, object_in_question } => {
                self.see_if_object_on_scanline(object_number, object_in_question, lcd)
            },
        }
    }

    fn get_object_attributes(&mut self, oam: &ObjectAttributeMemory, object_number: u8) {
        self.scanner_stage = OamScannerStage::Comparing {
            object_number,
            object_in_question: oam.get_object_attributes(object_number),
        }
    }

    fn see_if_object_on_scanline(&mut self, object_number: u8, object_in_question: ObjectAttributes, lcd: &Lcd) {
        if lcd.object_on_scanline(object_in_question.get_y_position()) {
            self.objects_on_this_line.insert(object_in_question);
        }

        self.scanner_stage = OamScannerStage::Scanning { object_number: object_number + 1 };
    }

    fn reset_for_new_scanline(&mut self) {
        self.objects_on_this_line.reset_for_new_scanline();
        self.scanner_stage = OamScannerStage::Scanning { object_number: 0 }
    }
}

enum OamScannerStage {
    Scanning {
        object_number: u8,
    },
    Comparing {
        object_number: u8,
        object_in_question: ObjectAttributes,
    },
}

impl Default for OamScannerStage {
    fn default() -> Self {
        Self::Scanning { object_number: 0 }
    }
}

#[derive(Default)]
pub struct ObjectsOnThisLine {
    objects: [ObjectAttributes; 10],
    number_of_objects: u8, // cannot be more than 10
}

impl ObjectsOnThisLine {
    fn insert(&mut self, object_attributes: ObjectAttributes) {
        self.objects[self.number_of_objects as usize] = object_attributes;
        self.number_of_objects += 1;
    }

    fn is_at_capacity(&self) -> bool {
        self.number_of_objects == 10
    }
    pub fn borrow_objects(&self) -> &[ObjectAttributes; 10] {
        &self.objects
    }
    fn reset_for_new_scanline(&mut self) {
        self.number_of_objects = 0;
    }
}

/// Unit of time for the PPU. 1 Dot basically is 1 TCycle
pub type Dots = u16;
impl From<TCycles> for Dots {
    fn from(value: TCycles) -> Self {
        value.0 as u16
    }
}
