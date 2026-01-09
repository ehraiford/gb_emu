use crate::{
    instructions::{Instruction, InstructionError},
    rom_bank::RomBank00,
};

pub type MemoryAccessResult<T> = Result<T, MemoryAccessError>;

#[derive(Default)]
pub struct Bus {
    rom_bank_00: RomBank00,
}

impl Bus {
    pub fn read(&mut self, address: u16) -> MemoryAccessResult<u8> {
        self.get_mut_device_from_address(address).read(address)
    }
    pub fn read_u16(&mut self, address: u16) -> MemoryAccessResult<u16> {
        Ok((((self.read(address + 1)?) as u16) << 8) | self.read(address)? as u16)
    }
    pub fn write(&mut self, address: u16, value: u8) -> MemoryAccessResult<()> {
        self.get_mut_device_from_address(address).write(address, value)
    }
    pub fn write_u16(&mut self, address: u16, value: u16) -> MemoryAccessResult<()> {
        self.get_mut_device_from_address(address).write(address, value as u8)?;
        self.get_mut_device_from_address(address)
            .write(address + 1, (value >> 8) as u8)
    }
    pub fn peek(&self, address: u16) -> MemoryAccessResult<u8> {
        self.get_device_from_address(address).peek(address)
    }

    /// Peeks the three bytes starting at a given address.
    /// A convenient way to get all the bytes we might need when decoding an instruction.
    fn peek_3_byte_slice(&self, address: u16) -> MemoryAccessResult<[u8; 3]> {
        Ok([self.peek(address)?, self.peek(address + 1)?, self.peek(address + 2)?])
    }

    pub fn read_next_instruction(&mut self, pc: u16) -> MemoryAccessResult<(&'static Instruction, [u8; 3])> {
        let bytes = self.peek_3_byte_slice(pc)?;
        let instruction = <&Instruction>::try_from(bytes)?;

        for i in 0..instruction.bytes {
            // go ahead and mutably access the bytes now that we know how many are in the instruction
            self.read(pc + i as u16)?;
        }

        Ok((instruction, bytes))
    }

    fn get_mut_device_from_address(&mut self, address: u16) -> &mut dyn BusAccessible {
        self.get_mut_device(get_table_entry_for_address(address).device)
    }

    fn get_device_from_address(&self, address: u16) -> &dyn BusAccessible {
        self.get_device(get_table_entry_for_address(address).device)
    }

    fn get_device(&self, device: MMDevice) -> &dyn BusAccessible {
        match device {
            MMDevice::RomBank00 => &self.rom_bank_00,
            _ => todo!("Haven't done {device:?} yet"),
        }
    }

    fn get_mut_device(&mut self, device: MMDevice) -> &mut dyn BusAccessible {
        match device {
            MMDevice::RomBank00 => &mut self.rom_bank_00,
            _ => todo!("Haven't done {device:?} yet"),
        }
    }
}

struct MMTableEntry {
    device: MMDevice,
    base_address: u16,
    size: u16,
}

impl MMTableEntry {
    const fn new(device: MMDevice, base_address: u16, end_address: u16) -> Self {
        Self { device, base_address, size: end_address - base_address }
    }
}

const MEMORY_MAP: &[MMTableEntry] = &[
    MMTableEntry::new(MMDevice::RomBank00, 0x0000, 0x4000),
    MMTableEntry::new(MMDevice::CartridgeRomBank, 0x4000, 0x8000),
    MMTableEntry::new(MMDevice::VideoRam, 0x8000, 0xA000),
    MMTableEntry::new(MMDevice::ExternalRam, 0xA000, 0xC000),
    MMTableEntry::new(MMDevice::WorkRam00, 0xC000, 0xD000),
    MMTableEntry::new(MMDevice::SwitchableBankWorkRam, 0xD000, 0xE000),
    MMTableEntry::new(MMDevice::EchoRam, 0xE000, 0xFE00),
    MMTableEntry::new(MMDevice::ObjectAttributeMemory, 0xFE00, 0xFEA0),
    MMTableEntry::new(MMDevice::Unusable, 0xFEA0, 0xFF00),
    MMTableEntry::new(MMDevice::IoRegisters, 0xFF00, 0xFF80),
    MMTableEntry::new(MMDevice::HighRam, 0xFF80, 0xFFFF),
    MMTableEntry {
        device: MMDevice::InterruptEnableRegister,
        base_address: 0xFFFF,
        size: 0x1,
    },
];

fn get_table_entry_for_address(address: u16) -> &'static MMTableEntry {
    MEMORY_MAP
        .iter()
        .find(|e| address < e.base_address + e.size)
        .expect("Every device should be accounted for in the map")
}

fn get_mm_table_entry_for_device(device: MMDevice) -> &'static MMTableEntry {
    MEMORY_MAP
        .iter()
        .find(|e| e.device == device)
        .expect("Every device should be accounted for in the map")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MMDevice {
    RomBank00,
    CartridgeRomBank,
    VideoRam,
    ExternalRam,
    WorkRam00,
    SwitchableBankWorkRam, // GameBoy Color Only
    EchoRam,               // Mirror of C000-DDFF
    ObjectAttributeMemory,
    Unusable,
    IoRegisters,
    HighRam,
    InterruptEnableRegister,
}

pub trait BusAccessible {
    // This would be better as a const but you can't make trait objects of traits with associated consts
    fn _get_enum_device(&self) -> MMDevice;
    fn read(&mut self, address: u16) -> MemoryAccessResult<u8>;
    fn write(&mut self, address: u16, value: u8) -> MemoryAccessResult<()>;
    fn peek(&self, address: u16) -> MemoryAccessResult<u8>;
}

#[derive(Debug)]
pub enum MemoryAccessError {
    NotAnOperation(u8),
    FailedToReadAddress,
}

impl From<InstructionError> for MemoryAccessError {
    fn from(value: InstructionError) -> Self {
        match value {
            InstructionError::LdhLowValue(_) | InstructionError::InvalidOperand => {
                unreachable!("There shouldn't be any place this conversion happens")
            },
            InstructionError::InvalidOperation(byte) => Self::NotAnOperation(byte),
            InstructionError::MemoryAccessError(_) => todo!(),
            InstructionError::OperandCannotBeSet => todo!(),
        }
    }
}
