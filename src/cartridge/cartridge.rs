use crate::{
    bus::{Address, BusDefault, MemoryTarget},
    cartridge::{
        header::Header,
        memory_bank_controllers::{BankController, MemoryBankController},
    },
    onboard_memory::rom_and_ram::{RamBank, RomBank},
};

pub const ROM_BANK_SIZE: usize = 16 * 1024;
pub const RAM_BANK_SIZE: usize = 8 * 1024;

pub struct Cartridge {
    memory_bank_controller: Option<MemoryBankController>,
    rom_banks: Vec<RomBank>,
    ram_banks: Vec<RamBank<RAM_BANK_SIZE>>,
    has_battery: bool,
}

impl Cartridge {
    pub fn new(data: &[u8]) -> Result<Self, CartridgeError> {
        let Some(header_bytes) = data.get(0x100..0x150) else {
            return Err(CartridgeError::RomTooSmall(data.len()));
        };
        let header = Header::new(header_bytes)?;

        let memory_bank_controller = header.get_memory_bank_controller();
        if let Some(mbc) = &memory_bank_controller {
            if !mbc.is_implemented() {
                return Err(CartridgeError::UnsupportedMemoryBankController(mbc.name()));
            }
        }
        let rom_banks = vec![RomBank::new(); header.get_num_rom_banks()];
        let ram_banks = vec![RamBank::new(); header.get_num_ram_banks()];
        let has_battery = header.has_battery();

        let mut this = Self { memory_bank_controller, rom_banks, ram_banks, has_battery };

        this.initialize_cartridge_data(data, header)?;

        Ok(this)
    }

    /// Intended to be if there is no cartridge loaded into the system
    pub fn no_cartridge() -> Self {
        Cartridge {
            memory_bank_controller: None,
            rom_banks: vec![RomBank::new(), RomBank::new()],
            ram_banks: vec![RamBank::new()],
            has_battery: false,
        }
    }

    /// Sets the ROM data to the values provided
    fn initialize_cartridge_data(&mut self, data: &[u8], header: Header) -> Result<(), CartridgeError> {
        let expected_kb = header.get_expected_bank_size_in_kb();
        let actual_kb = data.len() / 1024;
        if expected_kb != actual_kb {
            return Err(CartridgeError::SizeMismatch { expected_kb, actual_kb });
        }

        let rom_chunks = data.chunks_exact(ROM_BANK_SIZE);

        for (chunk, bank) in rom_chunks.zip(&mut self.rom_banks) {
            bank.get_data_mut().copy_from_slice(chunk);
        }

        Ok(())
    }

    pub fn peek(&self, address: Address, device: CartridgeDevice) -> u8 {
        if let Some(mbc) = &self.memory_bank_controller {
            mbc.peek(address, &self.rom_banks, &self.ram_banks, device)
        } else {
            let in_device_address = (address - device.get_starting_address()) as usize;
            *self
                .get_default_bank(device)
                .and_then(|b| b.get(in_device_address))
                .unwrap_or(&u8::DEFAULT_BUS_VALUE)
        }
    }

    pub fn read(&mut self, address: Address, device: CartridgeDevice) -> u8 {
        if let Some(mbc) = &mut self.memory_bank_controller {
            mbc.read(address, &self.rom_banks, &mut self.ram_banks, device)
        } else {
            let in_device_address = (address - device.get_starting_address()) as usize;
            *self
                .get_default_bank(device)
                .and_then(|b| b.get(in_device_address))
                .unwrap_or(&u8::DEFAULT_BUS_VALUE)
        }
    }
    pub fn write(&mut self, address: Address, value: u8, device: CartridgeDevice) {
        if let Some(mbc) = &mut self.memory_bank_controller {
            mbc.write(address, value, &mut self.ram_banks, device)
        } else {
            let in_device_address = (address - device.get_starting_address()) as usize;
            self.get_default_bank_mut(device)
                .and_then(|b| b.get_mut(in_device_address).map(|v| *v = value));
        }
    }

    fn get_default_bank(&self, device: CartridgeDevice) -> Option<&[u8]> {
        match device {
            CartridgeDevice::LowerRomBank => self.rom_banks.get(0).and_then(|b| Some(b.get_data())),
            CartridgeDevice::UpperRomBank => self.rom_banks.get(1).and_then(|b| Some(b.get_data())),
            CartridgeDevice::ExternalRam => self.ram_banks.get(0).and_then(|b| Some(b.get_data())),
        }
    }
    fn get_default_bank_mut(&mut self, device: CartridgeDevice) -> Option<&mut [u8]> {
        match device {
            CartridgeDevice::LowerRomBank => self.rom_banks.get_mut(0).and_then(|b| Some(b.get_data_mut())),
            CartridgeDevice::UpperRomBank => self.rom_banks.get_mut(1).and_then(|b| Some(b.get_data_mut())),
            CartridgeDevice::ExternalRam => self.ram_banks.get_mut(0).and_then(|b| Some(b.get_data_mut())),
        }
    }

    pub fn get_save_ram(&self) -> Option<Vec<u8>> {
        if !self.has_battery {
            return None;
        }

        let mut data = Vec::new();
        for bank in &self.ram_banks {
            data.extend_from_slice(bank.get_data());
        }
        if let Some(mbc) = &self.memory_bank_controller {
            data.extend_from_slice(&mbc.retrieve_save_data());
        }

        Some(data)
    }
    pub fn load_save_ram(&mut self, bytes: &[u8]) -> Result<(), CartridgeError> {
        let mut chunks = bytes.chunks_exact(RAM_BANK_SIZE);

        for (chunk, bank) in chunks.by_ref().zip(&mut self.ram_banks) {
            bank.get_data_mut().copy_from_slice(chunk);
        }

        let remainder = chunks.remainder();
        if let Some(mbc) = &mut self.memory_bank_controller {
            mbc.load_save_data(remainder)?;
        }

        Ok(())
    }
}

/// Why a ROM image could not be turned into a `Cartridge`. These are all "this file is not something
/// we can run" conditions, so callers can skip the ROM instead of taking the whole process down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartridgeError {
    /// Smaller than the 0x150-byte header, so there is nothing to parse.
    RomTooSmall(usize),
    /// The header's declared size disagrees with the file on disk.
    SizeMismatch {
        expected_kb: usize,
        actual_kb: usize,
    },
    /// Byte 0x147 is not a cartridge type we know about.
    UnknownCartridgeType(u8),
    /// Byte 0x148 is not a defined ROM size.
    UnknownRomSize(u8),
    /// Byte 0x149 is not a defined RAM size.
    UnknownRamSize(u8),
    /// The header names a controller we recognise but have not implemented yet.
    UnsupportedMemoryBankController(&'static str),
    MisMatchedRamSaveSize(String),
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RomTooSmall(len) => write!(f, "ROM is {len} bytes, too small to contain a header"),
            Self::SizeMismatch { expected_kb, actual_kb } => {
                write!(
                    f,
                    "ROM size doesn't match header: header says {expected_kb}KB, file is {actual_kb}KB"
                )
            },
            Self::UnknownCartridgeType(code) => write!(f, "unknown cartridge type {code:#04X}"),
            Self::UnknownRomSize(code) => write!(f, "unknown ROM size code {code:#04X}"),
            Self::UnknownRamSize(code) => write!(f, "unknown RAM size code {code:#04X}"),
            Self::UnsupportedMemoryBankController(name) => write!(f, "{name} is not implemented yet"),
            Self::MisMatchedRamSaveSize(string) => write!(f, "{}", string),
        }
    }
}

impl std::error::Error for CartridgeError {}

impl Default for Cartridge {
    fn default() -> Self {
        Self::no_cartridge()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CartridgeDevice {
    LowerRomBank,
    UpperRomBank,
    ExternalRam,
}

impl CartridgeDevice {
    pub const fn get_starting_address(&self) -> Address {
        match self {
            CartridgeDevice::LowerRomBank => MemoryTarget::RomBank00.get_base_address(),
            CartridgeDevice::UpperRomBank => MemoryTarget::BankableRom.get_base_address(),
            CartridgeDevice::ExternalRam => MemoryTarget::ExternalRam.get_base_address(),
        }
    }
}
