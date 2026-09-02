use crate::{
    bus::{Address, BusAccessFailure, BusDefault},
    cartridge::cartridge::ROM_BANK_SIZE,
};

#[derive(Clone, Copy)]
pub struct RomBank {
    data: [u8; ROM_BANK_SIZE],
}

impl RomBank {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn read(&self, address: Address) -> u8 {
        *self.data.get(address as usize).unwrap_or(&u8::DEFAULT_BUS_VALUE)
    }

    pub fn write(&mut self, _: Address, _: u8) {
        BusAccessFailure::TriedWritingToReadOnlyMemory.into()
    }

    pub fn peek(&self, address: Address) -> u8 {
        *self.data.get(address as usize).unwrap_or(&u8::DEFAULT_BUS_VALUE)
    }

    pub fn get_data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl Default for RomBank {
    fn default() -> Self {
        Self { data: [0; ROM_BANK_SIZE] }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RamBank<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> RamBank<SIZE> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read(&self, address: Address) -> u8 {
        *self.data.get(address as usize).unwrap_or(&u8::DEFAULT_BUS_VALUE)
    }

    pub fn write(&mut self, address: Address, value: u8) {
        if let Some(v) = self.data.get_mut(address as usize) {
            *v = value;
        }
    }

    pub fn peek(&self, address: Address) -> u8 {
        *self.data.get(address as usize).unwrap_or(&u8::DEFAULT_BUS_VALUE)
    }

    pub fn get_data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl<const SIZE: usize> Default for RamBank<SIZE> {
    fn default() -> Self {
        Self { data: [0; SIZE] }
    }
}
