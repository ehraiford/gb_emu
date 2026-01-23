use crate::{
    cartridge::cartridge::{Cartridge, CartridgeDevice},
    graphics::{oam::ObjectAttributeMemory, video_ram::VideoRam},
    helper_functions::{concat_2_bytes, log},
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
    pub fn read_u16(&mut self, address: Address) -> BusAccessOutcome<u16> {
        let BusAccessOutcome(little_byte, mut side_effects) = self.read(address);
        let BusAccessOutcome(big_byte, mut side_effects_2nd_access) = self.read(address + 1);
        side_effects.append(&mut side_effects_2nd_access);

        BusAccessOutcome(concat_2_bytes(big_byte, little_byte), side_effects)
    }
    pub fn write_u16(&mut self, address: Address, value: Address) -> BusAccessOutcome<()> {
        let little_byte = (value & 0xFF) as u8;
        let big_byte = (value >> 8) as u8;

        let mut side_effects = self.write(address, little_byte).1;

        side_effects.append(&mut self.write(address + 1, big_byte).1);

        BusAccessOutcome((), side_effects)
    }
    pub fn read_next_instruction(&mut self, pc: Address) -> BusAccessOutcome<&'static Instruction> {
        let BusAccessOutcome(first_byte, mut side_effects) = self.read(pc);

        let unprefixed_instruction = &UNPREFIXED[first_byte as usize];
        let outcome = match unprefixed_instruction.op_code {
            OpCode::Prefix => {
                let BusAccessOutcome(second_byte, mut side_effects_second_access) = self.read(pc + 1);
                let prefixed_instruction = &CBPREFIXED[second_byte as usize];
                side_effects.append(&mut side_effects_second_access);
                BusAccessOutcome(prefixed_instruction, side_effects)
            },
            _ => BusAccessOutcome(unprefixed_instruction, side_effects),
        };

        // log("Next instruction is: {:?}", instruction.op_code);

        outcome
    }

    pub fn print_graphics_data(&self) {
        println!("Graphics Data:");
        self.v_ram.print_all_tiles();
    }

    pub fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        let device = MMDevice::get_device_from_address(address);
        match device {
            MMDevice::RomBank00 => self.cartridge.read(address, CartridgeDevice::RomBank00),
            MMDevice::BankableRom => self.cartridge.read(address, CartridgeDevice::BankableRom),
            MMDevice::VideoRam => self.v_ram.read(address),
            MMDevice::ExternalRam => self.cartridge.read(address, CartridgeDevice::ExternalRam),
            MMDevice::WorkRam00 => self.w_ram_00.read(address),
            MMDevice::BankableWorkRam => self.bankable_w_ram.read(address),
            MMDevice::EchoRam => self.read(address - 0x2000),
            MMDevice::ObjectAttributeMemory => self.oam.read(address),
            MMDevice::Unusable => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            MMDevice::IoRegisters => self.io_registers.read(address),
            MMDevice::HighRam => self.h_ram.read(address),
            MMDevice::InterruptEnableRegister => self.ie.read(address),
        }
    }
    pub fn peek(&self, address: Address) -> u8 {
        let device = MMDevice::get_device_from_address(address);
        match device {
            MMDevice::RomBank00 => self.cartridge.peek(address, CartridgeDevice::RomBank00),
            MMDevice::BankableRom => self.cartridge.peek(address, CartridgeDevice::BankableRom),
            MMDevice::VideoRam => self.v_ram.peek(address),
            MMDevice::ExternalRam => self.cartridge.peek(address, CartridgeDevice::ExternalRam),
            MMDevice::WorkRam00 => self.w_ram_00.peek(address),
            MMDevice::BankableWorkRam => self.bankable_w_ram.peek(address),
            MMDevice::EchoRam => self.peek(address - 0x2000),
            MMDevice::ObjectAttributeMemory => self.oam.peek(address),
            MMDevice::Unusable => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            MMDevice::IoRegisters => self.io_registers.peek(address),
            MMDevice::HighRam => self.h_ram.peek(address),
            MMDevice::InterruptEnableRegister => self.ie.peek(address),
        }
    }
    pub fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        let device = MMDevice::get_device_from_address(address);
        match device {
            MMDevice::RomBank00 => self.cartridge.write(address, CartridgeDevice::RomBank00, value),
            MMDevice::BankableRom => self.cartridge.write(address, CartridgeDevice::BankableRom, value),
            MMDevice::VideoRam => self.v_ram.write(address, value),
            MMDevice::ExternalRam => self.cartridge.write(address, CartridgeDevice::ExternalRam, value),
            MMDevice::WorkRam00 => self.w_ram_00.write(address, value),
            MMDevice::BankableWorkRam => self.bankable_w_ram.write(address, value),
            MMDevice::EchoRam => self.write(address - 0x2000, value),
            MMDevice::ObjectAttributeMemory => self.oam.write(address, value),
            MMDevice::Unusable => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            MMDevice::IoRegisters => self.io_registers.write(address, value),
            MMDevice::HighRam => self.h_ram.write(address, value),
            MMDevice::InterruptEnableRegister => self.ie.write(address, value),
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

    fn read(&mut self, address: Address) -> BusAccessOutcome<u8>;
    fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()>;
    fn peek(&self, address: Address) -> u8;
}

pub type Address = u16;

pub enum BusAccessSideEffect {
    UnmapBootRom,
    // Add others here
}

pub struct BusAccessOutcome<T>(pub T, pub Vec<BusAccessSideEffect>);

impl<T> BusAccessOutcome<T> {
    pub fn default_outcome(value: T) -> Self {
        Self(value, vec![])
    }
}
pub enum BusAccessFailure {
    NothingMappedToAddress,
    InaccessbileInPpuMode,
    TriedAccessingUnusableMemory,
    TriedWritingToRom,
}

/// The return value for reads to inaccessible devices.
/// Just standardizing our garbage and removing mystical numbers.
pub const INACCESIBLE_RETURN_VALUE: u8 = 0xFF;

impl<T> From<T> for BusAccessOutcome<T> {
    fn from(value: T) -> Self {
        Self(value, vec![])
    }
}

impl From<BusAccessFailure> for u8 {
    fn from(access_failure: BusAccessFailure) -> Self {
        log(format_args!(access_failure));
        INACCESIBLE_RETURN_VALUE
    }
}

impl From<BusAccessFailure> for () {
    fn from(access_failure: BusAccessFailure) -> Self {
        log(format_args!(access_failure));
        ()
    }
}
