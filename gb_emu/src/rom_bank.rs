use crate::bus::{BusAccessible, MemoryAccessResult};

#[derive(Default)]
pub struct RomBank00 {}

impl BusAccessible for RomBank00 {
    fn get_enum_device(&self) -> crate::bus::MMDevice {
        crate::bus::MMDevice::RomBank00
    }

    fn read(&mut self, address: u16) -> MemoryAccessResult<u8> {
        Ok(0)
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryAccessResult<()> {
        Ok(())
    }

    fn peek(&self, address: u16) -> MemoryAccessResult<u8> {
        Ok(0)
    }
}
