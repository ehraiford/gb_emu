use crate::{
    bus::{Address, BusAccessible, BusDefault},
    onboard_memory::rom_and_ram::RamBank,
};

const BANK_SIZE: usize = 4 * 1024;

#[derive(Default)]
pub struct WorkRam00 {
    bank: RamBank<BANK_SIZE>,
}

impl BusAccessible for WorkRam00 {
    const MM_DEVICE: crate::bus::MemoryTarget = crate::bus::MemoryTarget::WorkRam00;

    fn read(&mut self, address: Address) -> u8 {
        self.bank.read(Self::local(address))
    }

    fn write(&mut self, address: Address, value: u8) {
        self.bank.write(Self::local(address), value)
    }

    fn peek(&self, address: Address) -> u8 {
        self.bank.peek(Self::local(address))
    }
}

pub struct BankableWorkRam {
    bankable_ram: Vec<RamBank<BANK_SIZE>>,
    active_bank_number: usize,
}

impl BankableWorkRam {
    pub fn set_active_bank_number(&mut self, bank_num: u8) {
        self.active_bank_number = bank_num as usize;
    }
}

impl Default for BankableWorkRam {
    /// A DMG has a single bank hardwired at 0xD000. Deriving `Default` here would leave the vec
    /// empty, which silently turns the whole 0xD000-0xDFFF range into open bus.
    fn default() -> Self {
        Self { bankable_ram: vec![RamBank::new()], active_bank_number: 0 }
    }
}

impl BusAccessible for BankableWorkRam {
    const MM_DEVICE: crate::bus::MemoryTarget = crate::bus::MemoryTarget::BankableWorkRam;

    fn read(&mut self, address: Address) -> u8 {
        self.bankable_ram
            .get_mut(self.active_bank_number)
            .map(|b| b.read(Self::local(address)))
            .unwrap_or(u8::DEFAULT_BUS_VALUE)
    }

    fn write(&mut self, address: Address, value: u8) {
        self.bankable_ram
            .get_mut(self.active_bank_number)
            .map(|b| b.write(Self::local(address), value));
    }

    fn peek(&self, address: Address) -> u8 {
        self.bankable_ram
            .get(self.active_bank_number)
            .map(|b| b.peek(Self::local(address)))
            .unwrap_or(u8::DEFAULT_BUS_VALUE)
    }
}
