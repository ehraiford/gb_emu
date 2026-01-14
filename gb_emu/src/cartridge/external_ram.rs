use crate::{
    bus::BusAccessible,
    cartridge::cartridge::RAM_BANK_SIZE,
    rom_and_ram::{BankableRam, RamBank},
};

#[derive(Default)]
pub struct ExternalRam {
    bankable_ram: BankableRam<RAM_BANK_SIZE>,
}

impl ExternalRam {
    pub fn new(num_of_banks: usize) -> Self {
        Self {
            bankable_ram: BankableRam::<RAM_BANK_SIZE>::new(num_of_banks),
        }
    }
}

impl BusAccessible for ExternalRam {
    const MM_DEVICE: crate::bus::MMDevice = crate::bus::MMDevice::ExternalRam;

    fn read(&mut self, address: u16) -> crate::bus::MemoryAccessResult<u8> {
        self.bankable_ram.read(address)
    }

    fn write(&mut self, address: u16, value: u8) -> crate::bus::MemoryAccessResult<()> {
        self.bankable_ram.write(address, value)
    }

    fn peek(&self, address: u16) -> crate::bus::MemoryAccessResult<u8> {
        self.bankable_ram.peek(address)
    }
}
