use crate::{
    bus::{BusAccessible, MemoryAccessResult},
    cartridge::cartridge::PartOfCartridge,
};

pub struct RomBank00 {
    data: [u8; 0x8000],
}

impl BusAccessible for RomBank00 {
    fn get_enum_device(&self) -> crate::bus::MMDevice {
        crate::bus::MMDevice::RomBank00
    }

    fn read(&mut self, address: u16) -> MemoryAccessResult<u8> {
        Ok(self.data[address as usize])
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryAccessResult<()> {
        self.data[address as usize] = value;
        Ok(())
    }

    fn peek(&self, address: u16) -> MemoryAccessResult<u8> {
        Ok(self.data[address as usize])
    }
}

impl Default for RomBank00 {
    fn default() -> Self {
        Self { data: [Default::default(); 0x8000] }
    }
}

impl PartOfCartridge for RomBank00 {}
