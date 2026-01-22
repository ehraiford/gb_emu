use crate::{
    cartridge::cartridge::{Cartridge, CartridgeDevice},
    game_boy::INACCESIBLE_RETURN_VALUE,
    graphics::{oam::ObjectAttributeMemory, video_ram::VideoRam},
    helper_functions::log,
    interrupts::InterruptEnableRegister,
    onboard_devices::{
        h_ram::HighRam,
        io_registers::IoRegisters,
        work_ram::{BankableWorkRam, WorkRam00},
    },
    processor::{
        instruction_tables::{CBPREFIXED, UNPREFIXED},
        instructions::{Instruction, OpCode},
    },
};

pub type MemoryAccessResult<T> = Result<T, MemoryAccessError>;
pub type Address = u16;

#[derive(Default)]
pub struct Bus {
    cartridge: Cartridge,
    v_ram: VideoRam,
    w_ram_00: WorkRam00,
    bankable_w_ram: BankableWorkRam,
    oam: ObjectAttributeMemory,
    io_registers: IoRegisters,
    h_ram: HighRam,
    ie: InterruptEnableRegister,
}

impl Bus {
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
    }
    pub fn read_u16(&mut self, address: Address) -> MemoryAccessResult<u16> {
        let little_byte = self.read(address)? as u16;
        let big_byte = self.read(address + 1)? as u16;
        Ok((big_byte << 8) | little_byte)
    }
    pub fn write_u16(&mut self, address: Address, value: Address) -> MemoryAccessResult<()> {
        let little_byte = (value & 0xFF) as u8;
        let big_byte = (value >> 8) as u8;
        self.write(address, little_byte)?;
        self.write(address + 1, big_byte)
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
        let result = match device {
            MMDevice::RomBank00 => self.cartridge.read(address, CartridgeDevice::RomBank00),
            MMDevice::BankableRom => self.cartridge.read(address, CartridgeDevice::BankableRom),
            MMDevice::VideoRam => self.v_ram.read(address),
            MMDevice::ExternalRam => self.cartridge.read(address, CartridgeDevice::ExternalRam),
            MMDevice::WorkRam00 => self.w_ram_00.read(address),
            MMDevice::BankableWorkRam => self.bankable_w_ram.read(address),
            MMDevice::EchoRam => self.read(address - 0x2000),
            MMDevice::ObjectAttributeMemory => self.oam.read(address),
            MMDevice::Unusable => Err(MemoryAccessError::TriedAccessingUnusableMemory),
            MMDevice::IoRegisters => self.io_registers.read(address),
            MMDevice::HighRam => self.h_ram.read(address),
            MMDevice::InterruptEnableRegister => self.ie.read(address),
        };

        match result {
            Ok(val) => Ok(val),
            Err(e) => match e {
                MemoryAccessError::NothingMappedToAddress
                | MemoryAccessError::InaccessibleInPpuMode
                | MemoryAccessError::TriedAccessingUnusableMemory => {
                    log(format_args!("Could not access anything at address: 0x{address:04x}."));
                    Ok(INACCESIBLE_RETURN_VALUE)
                },
                _ => return Err(e),
            },
        }
    }
    pub fn peek(&self, address: Address) -> MemoryAccessResult<u8> {
        let device = MMDevice::get_device_from_address(address);
        let result = match device {
            MMDevice::RomBank00 => self.cartridge.peek(address, CartridgeDevice::RomBank00),
            MMDevice::BankableRom => self.cartridge.peek(address, CartridgeDevice::BankableRom),
            MMDevice::VideoRam => self.v_ram.peek(address),
            MMDevice::ExternalRam => self.cartridge.peek(address, CartridgeDevice::ExternalRam),
            MMDevice::WorkRam00 => self.w_ram_00.peek(address),
            MMDevice::BankableWorkRam => self.bankable_w_ram.peek(address),
            MMDevice::EchoRam => self.peek(address - 0x2000),
            MMDevice::ObjectAttributeMemory => self.oam.peek(address),
            MMDevice::Unusable => Err(MemoryAccessError::TriedAccessingUnusableMemory),
            MMDevice::IoRegisters => self.io_registers.peek(address),
            MMDevice::HighRam => self.h_ram.peek(address),
            MMDevice::InterruptEnableRegister => self.ie.peek(address),
        };

        match result {
            Ok(val) => Ok(val),
            Err(e) => match e {
                MemoryAccessError::NothingMappedToAddress => {
                    log(format_args!("Could not access anything at address: 0x{address:04x}."));
                    Ok(0)
                },
                _ => return Err(e),
            },
        }
    }
    pub fn write(&mut self, address: Address, value: u8) -> MemoryAccessResult<()> {
        let device = MMDevice::get_device_from_address(address);
        let result = match device {
            MMDevice::RomBank00 => self.cartridge.write(address, CartridgeDevice::RomBank00, value),
            MMDevice::BankableRom => self.cartridge.write(address, CartridgeDevice::BankableRom, value),
            MMDevice::VideoRam => self.v_ram.write(address, value),
            MMDevice::ExternalRam => self.cartridge.write(address, CartridgeDevice::ExternalRam, value),
            MMDevice::WorkRam00 => self.w_ram_00.write(address, value),
            MMDevice::BankableWorkRam => self.bankable_w_ram.write(address, value),
            MMDevice::EchoRam => self.write(address - 0x2000, value),
            MMDevice::ObjectAttributeMemory => self.oam.write(address, value),
            MMDevice::Unusable => Err(MemoryAccessError::TriedAccessingUnusableMemory),
            MMDevice::IoRegisters => self.io_registers.write(address, value),
            MMDevice::HighRam => self.h_ram.write(address, value),
            MMDevice::InterruptEnableRegister => self.ie.write(address, value),
        };

        if let Err(e) = result {
            match e {
                MemoryAccessError::NothingMappedToAddress => {
                    log(format_args!("Could not access anything at address: 0x{address:04x}."))
                },
                _ => return Err(e),
            }
        }

        Ok(())
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
    NothingMappedToAddress,
    InaccessibleInPpuMode,
    TriedAccessingUnusableMemory,
}
