use crate::bus::{Address, BusAccessFailure};

#[derive(Default)]
pub struct Audio {}

impl Audio {
    pub fn read(&self, _address: Address) -> u8 {
        BusAccessFailure::Unimplemented.into()
    }
    pub fn write(&mut self, _address: Address, _value: u8) {
        BusAccessFailure::Unimplemented.into()
    }
}
