use crate::{bus::{MMDevice, BusAccessible}, cpu::Cpu};



#[derive(Default)]
pub struct GameBoy {
    cpu: Cpu,
}

impl GameBoy {
    pub fn new() -> Self {
        Default::default()
    }
}

