use crate::{
    bus::BusAccessible,
    cartridge::cartridge::{CartridgeError, CartridgeResult, PartOfCartridge},
    rom_and_ram::RomBank,
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
    fn get_number_of_banks(&self) -> usize {
        self.banks.len()
    }

    fn banks_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        self.banks.iter_mut().map(|b| b.get_data_mut())
    }
}

impl BusAccessible for BankableRoms {
    const MM_DEVICE: crate::bus::MMDevice = crate::bus::MMDevice::BankableRom;

    fn read(&mut self, address: u16) -> crate::bus::MemoryAccessResult<u8> {
        self.banks[self.active_bank_num as usize].read(address)
    }

    fn write(&mut self, address: u16, value: u8) -> crate::bus::MemoryAccessResult<()> {
        self.banks[self.active_bank_num as usize].write(address, value)
    }

    fn peek(&self, address: u16) -> crate::bus::MemoryAccessResult<u8> {
        self.banks[self.active_bank_num as usize].peek(address)
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
    fn get_number_of_banks(&self) -> usize {
        1
    }

    fn banks_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        vec![self.bank.get_data_mut()].into_iter()
    }
}

impl BusAccessible for RomBank00 {
    const MM_DEVICE: crate::bus::MMDevice = crate::bus::MMDevice::RomBank00;

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
