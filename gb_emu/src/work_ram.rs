use crate::{
    bus::BusAccessible,
    rom_and_ram::{BankableRam, RamBank},
};

const BANK_SIZE: usize = 4 * 1024;

#[derive(Default)]
pub struct WorkRam00 {
    bank: RamBank<BANK_SIZE>,
}

impl BusAccessible for WorkRam00 {
    const MM_DEVICE: crate::bus::MMDevice = crate::bus::MMDevice::WorkRam00;

    fn read(&mut self, address: u16) -> crate::bus::MemoryAccessResult<u8> {
        self.bank.read(address)
    }

    fn write(&mut self, address: u16, value: u8) -> crate::bus::MemoryAccessResult<()> {
        self.bank.write(address, value)
    }

    fn peek(&self, address: u16) -> crate::bus::MemoryAccessResult<u8> {
        self.bank.peek(address)
    }
}

#[derive(Default)]
pub struct BankableWorkRam {
    bankable_ram: BankableRam<BANK_SIZE>,
}

impl BankableWorkRam {
    pub fn new(num_of_banks: usize) -> Self {
        Self { bankable_ram: BankableRam::new(num_of_banks) }
    }
}

impl BusAccessible for BankableWorkRam {
    const MM_DEVICE: crate::bus::MMDevice = crate::bus::MMDevice::BankableWorkRam;

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
