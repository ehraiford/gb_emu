use crate::graphics::{oam::ObjectAttributeMemory, video_ram::VideoRam};

pub struct Ppu {
    scx: u8,
    scy: u8,
    wx: u8,
    wy: u8,
}

impl Ppu {}

pub struct PpuOperationContext<'a, 'b, 'c> {
    ppu: &'a mut Ppu,
    v_ram: &'b mut VideoRam,
    oam: &'c mut ObjectAttributeMemory,
}

impl<'a, 'b, 'c> PpuOperationContext<'a, 'b, 'c> {
    pub fn new(ppu: &'a mut Ppu, v_ram: &'b mut VideoRam, oam: &'c mut ObjectAttributeMemory) -> Self {
        Self { ppu, v_ram, oam }
    }

    fn get_window(&self) {
        todo!()
    }
}
