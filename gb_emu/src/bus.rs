use crate::{
    cartridge::cartridge::{Cartridge, CartridgeDevice},
    graphics::video_ram::VideoRam,
    instruction_tables::{CBPREFIXED, UNPREFIXED},
    instructions::{Instruction, InstructionError, OpCode},
    work_ram::{BankableWorkRam, WorkRam00},
};

pub type MemoryAccessResult<T> = Result<T, MemoryAccessError>;
pub type Address = u16;

#[derive(Default)]
pub struct Bus {
    cartridge: Cartridge,
    video_ram: VideoRam,
    work_ram_00: WorkRam00,
    bankable_work_ram: BankableWorkRam,
}

impl Bus {
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
    }
    pub fn read_u16(&mut self, address: Address) -> MemoryAccessResult<u16> {
        Ok((((self.read(address + 1)?) as u16) << 8) | self.read(address)? as u16)
    }
    pub fn write_u16(&mut self, address: Address, value: Address) -> MemoryAccessResult<()> {
        self.write(address, value as u8)?;
        self.write(address + 1, (value >> 8) as u8)
    }
    pub fn read_next_instruction(&mut self, pc: Address) -> MemoryAccessResult<&'static Instruction> {
        let first_byte = self.read(pc)?;

        let unprefixed_instruction = &UNPREFIXED[first_byte as usize];
        let instruction = match unprefixed_instruction.op_code {
            OpCode::Prefix => &CBPREFIXED[self.read(pc + 1)? as usize],
            OpCode::Illegal => return Err(MemoryAccessError::NotAnOperation(first_byte)),
            _ => unprefixed_instruction,
        };

        // log("Next instruction is: {:?}", instruction.op_code);

        Ok(instruction)
    }

    pub fn read(&mut self, address: Address) -> MemoryAccessResult<u8> {
        let device = MMDevice::get_device_from_address(address);
        match device {
            MMDevice::RomBank00 => self.cartridge.read(address, CartridgeDevice::RomBank00),
            MMDevice::BankableRom => self.cartridge.read(address, CartridgeDevice::BankableRom),
            MMDevice::VideoRam => self.video_ram.read(address),
            MMDevice::ExternalRam => self.cartridge.read(address, CartridgeDevice::ExternalRam),
            MMDevice::WorkRam00 => self.work_ram_00.read(address),
            MMDevice::BankableWorkRam => self.bankable_work_ram.read(address),
            MMDevice::EchoRam => self.read(address & 0x4FFF),
            MMDevice::ObjectAttributeMemory => todo!(),
            MMDevice::Unusable => todo!(),
            MMDevice::IoRegisters => todo!(),
            MMDevice::HighRam => todo!(),
            MMDevice::InterruptEnableRegister => todo!(),
        }
    }
    pub fn peek(&self, address: Address) -> MemoryAccessResult<u8> {
        let device = MMDevice::get_device_from_address(address);
        match device {
            MMDevice::RomBank00 => self.cartridge.peek(address, CartridgeDevice::RomBank00),
            MMDevice::BankableRom => self.cartridge.peek(address, CartridgeDevice::BankableRom),
            MMDevice::VideoRam => todo!(),
            MMDevice::ExternalRam => self.cartridge.peek(address, CartridgeDevice::ExternalRam),
            MMDevice::WorkRam00 => todo!(),
            MMDevice::BankableWorkRam => todo!(),
            MMDevice::EchoRam => todo!(),
            MMDevice::ObjectAttributeMemory => todo!(),
            MMDevice::Unusable => todo!(),
            MMDevice::IoRegisters => todo!(),
            MMDevice::HighRam => todo!(),
            MMDevice::InterruptEnableRegister => todo!(),
        }
    }
    pub fn write(&mut self, address: Address, value: u8) -> MemoryAccessResult<()> {
        let device = MMDevice::get_device_from_address(address);
        match device {
            MMDevice::RomBank00 => self.cartridge.write(address, CartridgeDevice::RomBank00, value),
            MMDevice::BankableRom => self.cartridge.write(address, CartridgeDevice::BankableRom, value),
            MMDevice::VideoRam => todo!(),
            MMDevice::ExternalRam => self.cartridge.write(address, CartridgeDevice::ExternalRam, value),
            MMDevice::WorkRam00 => todo!(),
            MMDevice::BankableWorkRam => todo!(),
            MMDevice::EchoRam => todo!(),
            MMDevice::ObjectAttributeMemory => todo!(),
            MMDevice::Unusable => todo!(),
            MMDevice::IoRegisters => todo!(),
            MMDevice::HighRam => todo!(),
            MMDevice::InterruptEnableRegister => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MMDevice {
    RomBank00,
    BankableRom,
    VideoRam,
    ExternalRam,
    WorkRam00,
    BankableWorkRam, // GameBoy Color Only
    EchoRam,         // Mirror of C000-DDFF
    ObjectAttributeMemory,
    Unusable,
    IoRegisters,
    HighRam,
    InterruptEnableRegister,
}

impl MMDevice {
    pub const fn get_base_address(&self) -> Address {
        match self {
            MMDevice::RomBank00 => 0x0000,
            MMDevice::BankableRom => 0x4000,
            MMDevice::VideoRam => 0x8000,
            MMDevice::ExternalRam => 0xA000,
            MMDevice::WorkRam00 => 0xC000,
            MMDevice::BankableWorkRam => 0xD000,
            MMDevice::EchoRam => 0xE000,
            MMDevice::ObjectAttributeMemory => 0xFE00,
            MMDevice::Unusable => 0xFEA0,
            MMDevice::IoRegisters => 0xFF00,
            MMDevice::HighRam => 0xFF80,
            MMDevice::InterruptEnableRegister => 0xFFFF,
        }
    }
    pub const fn get_end_address(&self) -> Address {
        match self {
            MMDevice::RomBank00 => 0x4000,
            MMDevice::BankableRom => 0x8000,
            MMDevice::VideoRam => 0xA000,
            MMDevice::ExternalRam => 0xC000,
            MMDevice::WorkRam00 => 0xD000,
            MMDevice::BankableWorkRam => 0xE000,
            MMDevice::EchoRam => 0xFE00,
            MMDevice::ObjectAttributeMemory => 0xFEA0,
            MMDevice::Unusable => 0xFF00,
            MMDevice::IoRegisters => 0xFF80,
            MMDevice::HighRam => 0xFFFF,
            MMDevice::InterruptEnableRegister => 0xFFFF,
        }
    }
    pub const fn get_device_from_address(address: Address) -> Self {
        let enumerated_devices: &[MMDevice] = &[
            MMDevice::RomBank00,
            MMDevice::BankableRom,
            MMDevice::VideoRam,
            MMDevice::ExternalRam,
            MMDevice::WorkRam00,
            MMDevice::BankableWorkRam, // GameBoy Color Only
            MMDevice::EchoRam,         // Mirror of C000-DDFF
            MMDevice::ObjectAttributeMemory,
            MMDevice::Unusable,
            MMDevice::IoRegisters,
            MMDevice::HighRam,
        ];

        let mut i = 0;
        while i < enumerated_devices.len() {
            if enumerated_devices[i].get_end_address() > address {
                return enumerated_devices[i];
            }
            i += 1;
        }
        // only one that doesn't fit in there is IE Register
        MMDevice::InterruptEnableRegister
    }
}

pub trait BusAccessible {
    const MM_DEVICE: MMDevice;

    fn local(global: Address) -> Address {
        global - Self::MM_DEVICE.get_base_address()
    }

    fn base_address(&self) -> Address {
        Self::MM_DEVICE.get_base_address()
    }
    fn end_address(&self) -> Address {
        Self::MM_DEVICE.get_end_address()
    }

    fn read(&mut self, address: Address) -> MemoryAccessResult<u8>;
    fn write(&mut self, address: Address, value: u8) -> MemoryAccessResult<()>;
    fn peek(&self, address: Address) -> MemoryAccessResult<u8>;
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
            InstructionError::OperandCannotBeSet => todo!(),
        }
    }
}
