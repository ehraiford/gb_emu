use crate::bus::{Address, BusAccessFailure};

#[derive(Default)]
pub struct Audio {}

impl Audio {
    pub fn read(&self, address: Address) -> u8 {
        BusAccessFailure::Unimplemented.into()
    }
    pub fn write(&mut self, address: Address, value: u8) {
        BusAccessFailure::Unimplemented.into()
    }
}
