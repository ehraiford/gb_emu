use crate::{
    bus::{Address, BusAccessOutcome, BusAccessible, MemoryTarget},
    cartridge::{
        external_ram::ExternalRam,
        header::Header,
        rom_banks::{BankableRoms, RomBank00},
    },
};

pub const ROM_BANK_SIZE: usize = 16 * 1024;
pub const RAM_BANK_SIZE: usize = 8 * 1024;

pub struct Cartridge {
    rom_bank_00: RomBank00,
    controlled_memory: ControlledMemory,
}

impl Cartridge {
    pub fn new(data: &[u8]) -> CartridgeResult<Self> {
        let header = Header::new(&data[0x100..0x150]);
        let controlled_memory = header.get_header_defined_structures();
        let mut this = Self { rom_bank_00: RomBank00::default(), controlled_memory };

        this.initialize_cartridge_data(data, header)?;

        Ok(this)
    }

    /// Intended to be if there is no cartridge loaded into the system
    pub fn no_cartridge() -> Self {
        Cartridge {
            rom_bank_00: Default::default(),
            controlled_memory: Default::default(),
        }
    }

    fn _get_header(&self) -> Header {
        Header::new(&self.rom_bank_00.get_bank_data()[0x100..0x150])
    }

    /// Sets the ROM data to the values provided
    fn initialize_cartridge_data(&mut self, data: &[u8], header: Header) -> CartridgeResult<()> {
        if header.get_expected_bank_size_in_kb() != data.len() / 1024 {
            return Err(CartridgeError::DataSizeBankMismatch);
        }

        let mut rom_chunks = data.chunks_exact(ROM_BANK_SIZE);
        let first_chunk = rom_chunks.next().unwrap().chunks_exact(ROM_BANK_SIZE);

        self.rom_bank_00.load_from_cartridge(first_chunk);
        self.controlled_memory.bankable_roms.load_from_cartridge(rom_chunks);

        Ok(())
    }

    pub fn read(&mut self, address: Address, device: CartridgeDevice) -> BusAccessOutcome<u8> {
        match device {
            CartridgeDevice::RomBank00 => self.rom_bank_00.read(address),
            CartridgeDevice::BankableRom => self
                .controlled_memory
                .read(address, ControlledMemoryDevice::BankableRom),
            CartridgeDevice::ExternalRam => self
                .controlled_memory
                .read(address, ControlledMemoryDevice::ExternalRam),
        }
    }
    pub fn write(&mut self, address: Address, device: CartridgeDevice, value: u8) -> BusAccessOutcome<()> {
        match device {
            CartridgeDevice::RomBank00 => self.rom_bank_00.write(address, value),
            CartridgeDevice::BankableRom => {
                self.controlled_memory
                    .write(address, ControlledMemoryDevice::BankableRom, value)
            },
            CartridgeDevice::ExternalRam => {
                self.controlled_memory
                    .write(address, ControlledMemoryDevice::ExternalRam, value)
            },
        }
    }
    pub fn peek(&self, address: Address, device: CartridgeDevice) -> u8 {
        match device {
            CartridgeDevice::RomBank00 => self.rom_bank_00.peek(address),
            CartridgeDevice::BankableRom => self
                .controlled_memory
                .peek(address, ControlledMemoryDevice::BankableRom),
            CartridgeDevice::ExternalRam => self
                .controlled_memory
                .peek(address, ControlledMemoryDevice::ExternalRam),
        }
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self::no_cartridge()
    }
}

pub enum CartridgeDevice {
    RomBank00,
    BankableRom,
    ExternalRam,
}

impl CartridgeDevice {
    const fn get_starting_address(&self) -> Address {
        match self {
            CartridgeDevice::RomBank00 => MemoryTarget::RomBank00.get_base_address(),
            CartridgeDevice::BankableRom => MemoryTarget::BankableRom.get_base_address(),
            CartridgeDevice::ExternalRam => MemoryTarget::ExternalRam.get_base_address(),
        }
    }
}

// A trait that all items on the bus that come from the cartridge should implement
pub trait PartOfCartridge {
    fn load_from_cartridge(&mut self, data: std::slice::ChunksExact<'_, u8>) {
        for (chunk, bank) in data.zip(self.banks_mut()) {
            bank.copy_from_slice(chunk);
        }
    }

    fn get_number_of_banks(&mut self) -> usize {
        self.banks_mut().count()
    }

    fn banks_mut(&mut self) -> impl Iterator<Item = &mut [u8]>;
}

pub struct MemoryBankController {
    mbc_type: MBCType,
}

impl MemoryBankController {
    fn new(mbc_type: MBCType) -> Self {
        Self { mbc_type }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MBCType {
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
}

#[derive(Default)]
pub struct ControlledMemory {
    memory_bank_controller: Option<MemoryBankController>,
    bankable_roms: BankableRoms,
    external_ram: ExternalRam,
}

impl ControlledMemory {
    pub fn new(mbc_type: Option<MBCType>, num_rom_banks: usize, num_ram_banks: usize) -> Self {
        let memory_bank_controller = mbc_type.map(|t| MemoryBankController::new(t));
        Self {
            memory_bank_controller,
            bankable_roms: BankableRoms::new(num_rom_banks - 1),
            external_ram: ExternalRam::new(num_ram_banks),
        }
    }

    pub fn read(&mut self, address: Address, device: ControlledMemoryDevice) -> BusAccessOutcome<u8> {
        match device {
            ControlledMemoryDevice::BankableRom => self.bankable_roms.read(address),
            ControlledMemoryDevice::ExternalRam => self.external_ram.read(address),
        }
    }
    pub fn write(&mut self, address: Address, device: ControlledMemoryDevice, value: u8) -> BusAccessOutcome<()> {
        match device {
            ControlledMemoryDevice::BankableRom => self.bankable_roms.write(address, value),
            ControlledMemoryDevice::ExternalRam => self.external_ram.write(address, value),
        }
    }
    pub fn peek(&self, address: Address, device: ControlledMemoryDevice) -> u8 {
        match device {
            ControlledMemoryDevice::BankableRom => self.bankable_roms.peek(address),
            ControlledMemoryDevice::ExternalRam => self.external_ram.peek(address),
        }
    }
}

pub enum ControlledMemoryDevice {
    BankableRom,
    ExternalRam,
}

pub type CartridgeResult<T> = Result<T, CartridgeError>;

#[derive(Debug)]
pub enum CartridgeError {
    DataSizeBankMismatch,
}
