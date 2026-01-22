use crate::graphics::{
    lcd::{LcdRegisters, PpuMode},
    oam::ObjectAttributeMemory,
    video_ram::VideoRam,
};

pub struct Ppu {}

impl Ppu {}

struct PpuOperationContext<'a, 'b, 'c, 'd> {
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

    pub fn tick_dot(&mut self) -> PpuTickOutcome {
        todo!()
    }

    fn get_scanline(&mut self) -> Scanline {
        todo!()
    }

    fn get_window(&self) {
        todo!()
    }

    fn get_ppu_mode(&self) -> PpuMode {
        self.lcd_regs.get_ppu_mode()
    }

    fn set_ppu_mode(&mut self, mode: PpuMode) {
        self.v_ram.update_ppu_mode(mode);
        self.oam.update_ppu_mode(mode);
        self.lcd_regs.update_ppu_mode(mode);
    }
}

struct Scanline {}

pub enum PpuTickOutcome {
    NewPpuMode(PpuMode),
}

pub trait VideoMemory {
    fn update_ppu_mode(&mut self, mode: PpuMode);
}
