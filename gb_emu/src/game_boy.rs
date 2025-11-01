use std::path::PathBuf;

use crate::{bus::{Bus, BusAccessible, MMDevice}, cpu::Cpu};



#[derive(Default)]
pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    pub fn new() -> Self {
        Default::default()
    }
}

