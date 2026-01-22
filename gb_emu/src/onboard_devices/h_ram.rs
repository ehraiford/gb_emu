use crate::{
    bus::{Address, BusAccessible, MMDevice, MemoryAccessResult},
    onboard_devices::rom_and_ram::RamBank,
};

#[derive(Default)]
pub struct HighRam {
    data: RamBank<0x7F>,
}

impl BusAccessible for HighRam {
    const MM_DEVICE: MMDevice = MMDevice::HighRam;

    fn read(&mut self, address: Address) -> MemoryAccessResult<u8> {
        let address = Self::local(address);
        self.data.read(address)
    }

    fn write(&mut self, address: Address, value: u8) -> MemoryAccessResult<()> {
        let address = Self::local(address);
        self.data.write(address, value)
    }

    fn peek(&self, address: Address) -> MemoryAccessResult<u8> {
        let address = Self::local(address);
        self.data.peek(address)
    }
}
