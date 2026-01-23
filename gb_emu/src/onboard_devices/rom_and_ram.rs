use crate::{
    bus::{Address, BusAccessFailure, BusAccessOutcome},
    cartridge::cartridge::ROM_BANK_SIZE,
};

#[derive(Clone, Copy)]
pub struct RomBank {
    data: [u8; ROM_BANK_SIZE],
}

impl RomBank {
    pub fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        self.data[address as usize].into()
    }

    pub fn write(&mut self, _: Address, _: u8) -> BusAccessOutcome<()> {
        <()>::from(BusAccessFailure::TriedWritingToRom).into()
    }

    pub fn peek(&self, address: Address) -> u8 {
        self.data[address as usize].into()
    }

    pub fn get_data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl Default for RomBank {
    fn default() -> Self {
        Self { data: [0; ROM_BANK_SIZE] }
    }
}

#[derive(Clone, Copy)]
pub struct RamBank<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> RamBank<SIZE> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        self.data[address as usize].into()
    }

    pub fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        self.data[address as usize] = value;
        BusAccessOutcome::default_outcome(())
    }

    pub fn peek(&self, address: Address) -> u8 {
        self.data[address as usize]
    }

    pub fn get_data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl<const SIZE: usize> Default for RamBank<SIZE> {
    fn default() -> Self {
        Self { data: [0; SIZE] }
    }
}

pub struct BankableRam<const SIZE: usize> {
    banks: Vec<RamBank<SIZE>>,
    active_bank_num: usize,
}

impl<const SIZE: usize> BankableRam<SIZE> {
    pub fn new(num_of_banks: usize) -> Self {
        Self {
            banks: vec![RamBank::<SIZE>::default(); num_of_banks],
            active_bank_num: 0,
        }
    }

    fn get_active_bank_number(&self) -> <usize> {
        if self.banks.is_empty() {
            Err(crate::bus::MemoryAccessError::NothingMappedToAddress)
        } else {
            Ok(self.active_bank_num)
        }
    }

    pub fn read(&mut self, address: Address) -> crate::bus::MemoryAccessResult<u8> {
        let bank_number: usize = self.get_active_bank_number()?;
        self.banks[bank_number].read(address)
    }

    pub fn write(&mut self, address: Address, value: u8) -> crate::bus::MemoryAccessResult<()> {
        let bank_number: usize = self.get_active_bank_number()?;
        self.banks[bank_number].write(address, value)
    }

    pub fn peek(&self, address: Address) -> crate::bus::MemoryAccessResult<u8> {
        let bank_number: usize = self.get_active_bank_number()?;
        self.banks[bank_number].peek(address)
    }
}

impl<const SIZE: usize> Default for BankableRam<SIZE> {
    fn default() -> Self {
        Self {
            banks: vec![RamBank::<SIZE>::new()],
            active_bank_num: Default::default(),
        }
    }
}
