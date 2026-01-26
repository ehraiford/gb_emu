use crate::{
    bus::{Address, BusAccessible},
    onboard_memory::rom_and_ram::{BankableRam, RamBank},
};

const BANK_SIZE: usize = 4 * 1024;

#[derive(Default)]
pub struct WorkRam00 {
    bank: RamBank<BANK_SIZE>,
}

impl BusAccessible for WorkRam00 {
    const MM_DEVICE: crate::bus::MemoryTarget = crate::bus::MemoryTarget::WorkRam00;

    fn read(&mut self, address: Address) -> crate::bus::BusAccessOutcome<u8> {
        self.bank.read(Self::local(address))
    }

    fn write(&mut self, address: Address, value: u8) -> crate::bus::BusAccessOutcome<()> {
        self.bank.write(Self::local(address), value)
    }

    fn peek(&self, address: Address) -> u8 {
        self.bank.peek(Self::local(address))
    }
}

#[derive(Default)]
pub struct BankableWorkRam {
    bankable_ram: BankableRam<BANK_SIZE>,
}

impl BankableWorkRam {
    pub fn set_active_bank_number(&mut self, bank_num: u8) {
        self.bankable_ram.set_active_bank_number(bank_num);
    }
}

impl BusAccessible for BankableWorkRam {
    const MM_DEVICE: crate::bus::MemoryTarget = crate::bus::MemoryTarget::BankableWorkRam;

    fn read(&mut self, address: Address) -> crate::bus::BusAccessOutcome<u8> {
        self.bankable_ram.read(Self::local(address))
    }

    fn write(&mut self, address: Address, value: u8) -> crate::bus::BusAccessOutcome<()> {
        self.bankable_ram.write(Self::local(address), value)
    }

    fn peek(&self, address: Address) -> u8 {
        self.bankable_ram.peek(Self::local(address))
    }
}
