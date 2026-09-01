#[cfg(not(feature = "headless"))]
use std::sync::Arc;

#[cfg(not(feature = "headless"))]
use crate::os_interface::window::{SenderFrameHandle, TripleBuffer};
use crate::{
    bus::MemoryTarget,
    game_boy::{EventQueue, GameBoyEvent, TCycles},
    graphics::{
        lcd::Lcd,
        oam::{ObjectAttributeMemory, ObjectAttributes},
        pixel_fetchers::PixelFetchers,
        video_ram::{ColoredPixel, VideoRam},
    },
    io_devices::interrupts::Interrupt,
};

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
pub const SCREEN_SIZE: usize = SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize;
const DOTS_PER_LINE: Dots = 456;

pub struct Ppu {
    mode_tracking: PpuModeTracker,
    pixel_fetchers: PixelFetchers,
    screen: Screen,
    oam_scanner: OamScanner,
}

impl Ppu {
    #[cfg(not(feature = "headless"))]
    pub fn new(frame_handle: SenderFrameHandle) -> Self {
        Self {
            screen: Screen::new(frame_handle),
            mode_tracking: Default::default(),
            pixel_fetchers: Default::default(),
            oam_scanner: Default::default(),
        }
    }

    #[cfg(feature = "headless")]
    pub fn new() -> Self {
        Self {
            screen: Screen::new(),
            mode_tracking: Default::default(),
            pixel_fetchers: Default::default(),
            oam_scanner: Default::default(),
        }
    }

    pub fn tick(
        &mut self,
        v_ram: &mut VideoRam,
        oam: &mut ObjectAttributeMemory,
        lcd: &mut Lcd,
        events: &mut EventQueue,
    ) {
        if !lcd.is_ppu_enabled() {
            return;
        }
        let mut context = PpuOperationContext::new(self, v_ram, oam, lcd, events);
        for _ in 0..4 {
            context.tick()
        }
    }

    /// Used for OAM corruption bug.
    /// There are 80 dots for Mode 2 to go across 20 rows, so row number is scanned dots / 4.
    /// Counted from the scan's own progress rather than the position in the line, because the
    /// first line after an LCD enable starts the scan 4 dots in and would otherwise read high.
    pub fn get_oam_scan_row(&self) -> Option<usize> {
        if self.mode_tracking.mode == PpuMode::OamScan {
            Some((self.mode_tracking.completed_cycles / 4) as usize)
        } else {
            None
        }
    }

    fn get_mode(&self) -> &PpuMode {
        &self.mode_tracking.mode
    }

    fn reset_for_new_scanline(&mut self) {
        self.oam_scanner.reset_for_new_scanline();
        self.pixel_fetchers.reset_for_new_scanline();
    }

    fn reset_for_new_frame(&mut self) {
        self.reset_for_new_scanline();
        self.pixel_fetchers.reset_window_y();
        self.screen.submit_frame();
    }

    pub fn handle_objects_disabled(&mut self) {
        self.pixel_fetchers.handle_objects_disabled();
    }
}

pub struct PpuOperationContext<'a, 'b, 'c, 'd, 'e> {
    ppu: &'a mut Ppu,
    v_ram: &'b mut VideoRam,
    oam: &'c mut ObjectAttributeMemory,
    lcd: &'d mut Lcd,
    events: &'e mut EventQueue,
}

impl<'a, 'b, 'c, 'd, 'e> PpuOperationContext<'a, 'b, 'c, 'd, 'e> {
    pub fn new(
        ppu: &'a mut Ppu,
        v_ram: &'b mut VideoRam,
        oam: &'c mut ObjectAttributeMemory,
        lcd: &'d mut Lcd,
        events: &'e mut EventQueue,
    ) -> Self {
        Self { ppu, v_ram, oam, lcd, events }
    }

    pub fn enable(&mut self) {
        self.ppu.reset_for_new_frame();
        self.ppu.mode_tracking = PpuModeTracker::new_from_lcd_enable();
        self.lcd.set_ppu_mode(Default::default(), self.events);
    }

    pub fn disable(&mut self) {
        self.ppu.reset_for_new_frame();
        self.ppu.mode_tracking.start_horizontal_blank();
        self.lcd.set_ppu_mode(PpuMode::HorizontalBlank, self.events);
    }

    fn tick_oam_scan(&mut self) {
        self.ppu.oam_scanner.tick(self.oam, self.lcd);
    }

    fn tick_drawing_pixels(&mut self) {
        if let Some(pixel) = self.ppu.pixel_fetchers.tick(self.lcd, self.v_ram) {
            self.ppu.screen.draw_pixel(pixel);
            self.ppu.mode_tracking.pixels_left_to_push -= 1;
        }
    }

    fn tick_horizontal_blank(&mut self) {}

    fn tick_vertical_blank(&mut self) {}

    pub fn tick(&mut self) {
        match self.ppu.get_mode() {
            PpuMode::HorizontalBlank => self.tick_horizontal_blank(),
            PpuMode::VerticalBlank => self.tick_vertical_blank(),
            PpuMode::OamScan => self.tick_oam_scan(),
            PpuMode::DrawingPixels => self.tick_drawing_pixels(),
        };

        let (increment_ly, new_mode) = self
            .ppu
            .mode_tracking
            .process_tick(self.lcd.get_ly(), self.events)
            .destructure();

        if increment_ly {
            self.go_to_next_line();
        }

        if let Some(mode) = new_mode {
            self.events.push(GameBoyEvent::ChangeBusAccessForPpuMode(mode));
            self.lcd.set_ppu_mode(mode, self.events);
            if mode == PpuMode::DrawingPixels {
                self.ppu
                    .pixel_fetchers
                    .take_scanned_objects(self.ppu.oam_scanner.objects_on_this_line);
            }
        }

        self.apply_ly_153_quirk();
    }

    fn apply_ly_153_quirk(&mut self) {
        const QUIRK_ONSET_DOTS: Dots = 4; // LY reads 153 for one M-cycle, then 0
        if self.lcd.get_ly() == Lcd::MAX_LY && self.ppu.mode_tracking.dot_in_line() >= QUIRK_ONSET_DOTS {
            self.lcd.mark_ly_reads_as_zero(self.events);
        }
    }

    fn go_to_next_line(&mut self) {
        match self.lcd.increment_ly(self.events) == 0 {
            true => self.ppu.reset_for_new_frame(),
            false => self.ppu.reset_for_new_scanline(),
        }
    }
}

struct PpuTickOutcome {
    increment_ly: bool,
    new_mode: Option<PpuMode>,
}

impl PpuTickOutcome {
    fn destructure(self) -> (bool, Option<PpuMode>) {
        (self.increment_ly, self.new_mode)
    }
}

struct PpuModeTracker {
    mode: PpuMode,
    remaining_dots_in_line: Dots,
    completed_cycles: u16,
    pixels_left_to_push: u8,
}

impl PpuModeTracker {
    fn process_tick_horizontal_blank(&mut self, ly: u8, events: &mut EventQueue) -> PpuTickOutcome {
        if self.remaining_dots_in_line == 0 {
            self.remaining_dots_in_line = DOTS_PER_LINE;
            if ly < SCREEN_HEIGHT - 1 {
                self.start_oam_scan()
            } else {
                events.push(GameBoyEvent::Interrupt(Interrupt::VBlank));
                self.start_vertical_blank()
            }
        } else {
            PpuTickOutcome { increment_ly: false, new_mode: None }
        }
    }

    fn process_tick_vertical_blank(&mut self, ly: u8) -> PpuTickOutcome {
        if self.remaining_dots_in_line == 0 {
            self.remaining_dots_in_line = DOTS_PER_LINE;

            if ly >= Lcd::MAX_LY {
                self.start_oam_scan()
            } else {
                self.start_vertical_blank()
            }
        } else {
            PpuTickOutcome { increment_ly: false, new_mode: None }
        }
    }

    fn process_tick_oam_scan(&mut self) -> PpuTickOutcome {
        self.completed_cycles += 1;

        if self.completed_cycles == 80 {
            self.start_drawing_pixels()
        } else {
            PpuTickOutcome { increment_ly: false, new_mode: None }
        }
    }

    fn process_tick_drawing_pixels(&mut self) -> PpuTickOutcome {
        if self.remaining_dots_in_line == 0 {
            panic!("DrawingPixels exhausted all dots in the line without transitioning to HBlank");
        }
        if self.pixels_left_to_push == 0 {
            self.start_horizontal_blank()
        } else {
            PpuTickOutcome { increment_ly: false, new_mode: None }
        }
    }

    fn dot_in_line(&self) -> Dots {
        DOTS_PER_LINE - self.remaining_dots_in_line
    }

    fn process_tick(&mut self, ly: u8, events: &mut EventQueue) -> PpuTickOutcome {
        self.remaining_dots_in_line -= 1;
        match self.mode {
            PpuMode::HorizontalBlank => self.process_tick_horizontal_blank(ly, events),
            PpuMode::VerticalBlank => self.process_tick_vertical_blank(ly),
            PpuMode::OamScan => self.process_tick_oam_scan(),
            PpuMode::DrawingPixels => self.process_tick_drawing_pixels(),
        }
    }

    fn start_oam_scan(&mut self) -> PpuTickOutcome {
        self.mode = PpuMode::OamScan;
        self.completed_cycles = 0;
        PpuTickOutcome { increment_ly: true, new_mode: Some(PpuMode::OamScan) }
    }
    fn start_drawing_pixels(&mut self) -> PpuTickOutcome {
        self.mode = PpuMode::DrawingPixels;
        self.pixels_left_to_push = 160;
        PpuTickOutcome { increment_ly: false, new_mode: Some(PpuMode::DrawingPixels) }
    }
    fn start_vertical_blank(&mut self) -> PpuTickOutcome {
        self.mode = PpuMode::VerticalBlank;
        PpuTickOutcome { increment_ly: true, new_mode: Some(PpuMode::VerticalBlank) }
    }
    pub fn start_horizontal_blank(&mut self) -> PpuTickOutcome {
        self.mode = PpuMode::HorizontalBlank;
        PpuTickOutcome {
            increment_ly: false,
            new_mode: Some(PpuMode::HorizontalBlank),
        }
    }
}

impl Default for PpuModeTracker {
    fn default() -> Self {
        Self {
            mode: PpuMode::OamScan,
            remaining_dots_in_line: DOTS_PER_LINE,
            completed_cycles: 0,
            pixels_left_to_push: 0,
        }
    }
}

impl PpuModeTracker {
    // The first scanline after an LCD enable is 4 dots (1 M-cycle) shorter than normal, so
    // remaining_dots starts at 452 instead of 456. Widening this to 16 dots regresses
    // 1-lcd_sync, 4-scanline_timing, 5-timing_bug, 6-timing_no_bug and 8-instr_effect.
    fn new_from_lcd_enable() -> Self {
        Self {
            mode: PpuMode::OamScan,
            remaining_dots_in_line: DOTS_PER_LINE - 4,
            completed_cycles: 0,
            pixels_left_to_push: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PpuMode {
    HorizontalBlank,
    VerticalBlank,
    #[default]
    OamScan,
    DrawingPixels,
}

impl PpuMode {
    pub fn get_cpu_accessible_video_targets(&self) -> &'static [MemoryTarget] {
        match self {
            PpuMode::HorizontalBlank | PpuMode::VerticalBlank => {
                &[MemoryTarget::VideoRam, MemoryTarget::ObjectAttributeMemory]
            },
            PpuMode::OamScan => &[MemoryTarget::VideoRam],
            PpuMode::DrawingPixels => &[],
        }
    }
    pub fn get_cpu_inaccessible_video_targets(&self) -> &'static [MemoryTarget] {
        match self {
            PpuMode::HorizontalBlank | PpuMode::VerticalBlank => &[],
            PpuMode::OamScan => &[MemoryTarget::ObjectAttributeMemory],
            PpuMode::DrawingPixels => &[MemoryTarget::VideoRam, MemoryTarget::ObjectAttributeMemory],
        }
    }
}

impl From<PpuMode> for u8 {
    fn from(mode: PpuMode) -> Self {
        match mode {
            PpuMode::HorizontalBlank => 0,
            PpuMode::VerticalBlank => 1,
            PpuMode::OamScan => 2,
            PpuMode::DrawingPixels => 3,
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

#[derive(Default, Clone, Copy)]
pub struct ObjectsOnThisLine {
    objects: [ObjectAttributes; 10],
    number_of_objects: u8, // cannot be more than 10
    taken_objects: [bool; 10],
}

impl ObjectsOnThisLine {
    fn insert(&mut self, object_attributes: ObjectAttributes) {
        self.objects[self.number_of_objects as usize] = object_attributes;
        self.number_of_objects += 1;
    }

    fn is_at_capacity(&self) -> bool {
        self.number_of_objects == 10
    }

    pub fn reset_for_new_scanline(&mut self) {
        self.number_of_objects = 0;
        self.taken_objects = [Default::default(); 10];
    }
    fn take_object(&mut self, index: usize) -> Option<ObjectAttributes> {
        self.objects.get(index).map(|o| {
            self.taken_objects[index] = true;
            *o
        })
    }
    pub fn take_object_at_x(&mut self, x: u8) -> Option<ObjectAttributes> {
        for i in 0..self.number_of_objects as usize {
            if !self.taken_objects[i] && self.objects[i].is_at_x_position(x) {
                return self.take_object(i);
            }
        }

        None
    }
}

/// Unit of time for the PPU. 1 Dot basically is 1 TCycle
pub type Dots = u16;
impl From<TCycles> for Dots {
    fn from(value: TCycles) -> Self {
        value.0 as u16
    }
}

struct Screen {
    current_pixel_index: u16,
    frame_being_drawn: Frame,
    #[cfg(not(feature = "headless"))]
    shared: Arc<TripleBuffer>,
}

impl Screen {}
#[cfg(feature = "headless")]
impl Screen {
    pub fn new() -> Self {
        Self { current_pixel_index: 0, frame_being_drawn: Frame::default() }
    }
    fn draw_pixel(&mut self, _: ColoredPixel) {}

    fn turn_off_screen(&mut self) {
        self.current_pixel_index = 0;
    }

    fn submit_frame(&self) {}
}

#[cfg(not(feature = "headless"))]
impl Screen {
    pub fn new(frame_handle: SenderFrameHandle) -> Self {
        Self {
            current_pixel_index: 0,
            frame_being_drawn: frame_handle.buffer,
            shared: frame_handle.shared,
        }
    }

    fn draw_pixel(&mut self, pixel: ColoredPixel) {
        self.frame_being_drawn.set_pixel(pixel, self.current_pixel_index);
        self.current_pixel_index += 1;

        if self.current_pixel_index == SCREEN_SIZE as u16 {
            self.current_pixel_index = 0;
        }
    }

    fn submit_frame(&mut self) {
        if let Ok(mut pending_frame) = self.shared.pending_frame.lock() {
            std::mem::swap(&mut self.frame_being_drawn, &mut *pending_frame);
            self.shared
                .has_new_frame
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

pub struct Frame {
    frame: Box<[ColoredPixel; SCREEN_SIZE]>,
}

impl Frame {
    fn set_pixel(&mut self, pixel: ColoredPixel, index: u16) {
        self.frame[index as usize] = pixel;
    }

    pub fn send_to_pixel_buffer(&self, pixel_buffer: &mut Box<[u32; SCREEN_SIZE]>) {
        for (source, destination) in self.frame.iter().zip(pixel_buffer.iter_mut()) {
            *destination = source.to_packed_rgb();
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            frame: vec![ColoredPixel::default(); SCREEN_SIZE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        }
    }
}
