use crate::{
    bus::{Address, BusAccessOutcome, BusAccessible, MemoryTarget},
    onboard_memory::rom_and_ram::RamBank,
};

#[derive(Default)]
pub struct HighRam {
    data: RamBank<0x7F>,
}

impl BusAccessible for HighRam {
    const MM_DEVICE: MemoryTarget = MemoryTarget::HighRam;

    fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        let address = Self::local(address);
        self.data.read(address)
    }

    fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        let address = Self::local(address);
        self.data.write(address, value)
    }

    fn peek(&self, address: Address) -> u8 {
        let address = Self::local(address);
        self.data.peek(address)
    }
}
