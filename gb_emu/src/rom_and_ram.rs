use crate::{
    bus::{Address, MemoryAccessResult},
    cartridge::cartridge::ROM_BANK_SIZE,
    helper_functions::log,
};

#[derive(Clone, Copy)]
pub struct RomBank {
    data: [u8; ROM_BANK_SIZE],
}

impl RomBank {
    pub fn read(&mut self, mut address: Address) -> MemoryAccessResult<u8> {
        Ok(self.data[address as usize])
    }

    pub fn write(&mut self, address: Address, value: u8) -> MemoryAccessResult<()> {
        log(&format!("Tried to write to ROM: {address:08x}: {value:02x}"));
        Ok(())
    }

    pub fn peek(&self, address: Address) -> MemoryAccessResult<u8> {
        Ok(self.data[address as usize])
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

    pub fn read(&mut self, address: Address) -> MemoryAccessResult<u8> {
        Ok(self.data[address as usize])
    }

    pub fn write(&mut self, address: Address, value: u8) -> MemoryAccessResult<()> {
        self.data[address as usize] = value;
        Ok(())
    }

    pub fn peek(&self, address: Address) -> MemoryAccessResult<u8> {
        Ok(self.data[address as usize])
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

    pub fn read(&mut self, address: Address) -> crate::bus::MemoryAccessResult<u8> {
        self.banks[self.active_bank_num as usize].read(address)
    }

    pub fn write(&mut self, address: Address, value: u8) -> crate::bus::MemoryAccessResult<()> {
        self.banks[self.active_bank_num as usize].write(address, value)
    }

    pub fn peek(&self, address: Address) -> crate::bus::MemoryAccessResult<u8> {
        self.banks[self.active_bank_num as usize].peek(address)
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
