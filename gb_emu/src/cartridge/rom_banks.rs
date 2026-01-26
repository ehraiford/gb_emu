use crate::{
    bus::{Address, BusAccessible},
    cartridge::cartridge::PartOfCartridge,
    onboard_memory::rom_and_ram::RomBank,
};

pub struct BankableRoms {
    banks: Vec<RomBank>,
    active_bank_num: u8,
}

impl BankableRoms {
    pub fn new(num_of_banks: usize) -> Self {
        Self {
            banks: vec![RomBank::default(); num_of_banks],
            active_bank_num: 0,
        }
    }
}

impl PartOfCartridge for BankableRoms {
    fn banks_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        self.banks.iter_mut().map(|b: &mut RomBank| b.get_data_mut())
    }
}

impl BusAccessible for BankableRoms {
    const MM_DEVICE: crate::bus::MemoryTarget = crate::bus::MemoryTarget::BankableRom;

    fn read(&mut self, address: Address) -> crate::bus::BusAccessOutcome<u8> {
        self.banks[self.active_bank_num as usize].read(Self::local(address))
    }

    fn write(&mut self, address: Address, value: u8) -> crate::bus::BusAccessOutcome<()> {
        self.banks[self.active_bank_num as usize].write(Self::local(address), value)
    }

    fn peek(&self, address: Address) -> u8 {
        self.banks[self.active_bank_num as usize].peek(Self::local(address))
    }
}

impl Default for BankableRoms {
    fn default() -> Self {
        Self {
            banks: vec![RomBank::default()],
            active_bank_num: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct RomBank00 {
    bank: RomBank,
}

impl RomBank00 {
    pub fn get_bank_data(&self) -> &[u8] {
        self.bank.get_data()
    }
}

impl PartOfCartridge for RomBank00 {
    fn banks_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        vec![self.bank.get_data_mut()].into_iter()
    }
}

impl BusAccessible for RomBank00 {
    const MM_DEVICE: crate::bus::MemoryTarget = crate::bus::MemoryTarget::RomBank00;

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
