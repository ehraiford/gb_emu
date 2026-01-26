use std::fmt::Display;

use crate::{
    cartridge::cartridge::{Cartridge, CartridgeDevice},
    game_boy::Change,
    graphics::{
        oam::{ObjectAttributeMemory, PriorityMode},
        video_ram::VideoRam,
    },
    helper_functions::{concat_2_bytes, log},
    interrupts::InterruptEnableRegister,
    io_registers::IoRegisters,
    onboard_memory::{
        bootrom::BootRom,
        h_ram::HighRam,
        work_ram::{BankableWorkRam, WorkRam00},
    },
    processor::{
        instruction_tables::{CBPREFIXED, UNPREFIXED},
        instructions::{Instruction, InstructionOutcome, OpCode},
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
    boot_rom: BootRom,
}

impl Bus {
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
    }
    pub fn unmap_bootrom(&mut self) {
        self.boot_rom.unmap();
    }

    pub fn start_oam_dma_transfer(&mut self) {
        todo!("HRAM is the only accessible part of memory")
    }
    pub fn end_oam_dma_transfer(&mut self) {
        todo!("HRAM is no longer the only accessible part of memory")
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
        let device = MemoryTarget::get_device_from_address(address);
        match device {
            MemoryTarget::RomBank00 => match self.boot_rom.mapped() && address < BootRom::SIZE {
                true => self.boot_rom.read(address),
                false => self.cartridge.read(address, CartridgeDevice::RomBank00),
            },
            MemoryTarget::BankableRom => self.cartridge.read(address, CartridgeDevice::BankableRom),
            MemoryTarget::VideoRam => self.v_ram.read(address),
            MemoryTarget::ExternalRam => self.cartridge.read(address, CartridgeDevice::ExternalRam),
            MemoryTarget::WorkRam00 => self.w_ram_00.read(address),
            MemoryTarget::BankableWorkRam => self.bankable_w_ram.read(address),
            MemoryTarget::EchoRam => self.read(address - 0x2000),
            MemoryTarget::ObjectAttributeMemory => self.oam.read(address),
            MemoryTarget::Unusable => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            MemoryTarget::IoRegisters => self.io_registers.read(address),
            MemoryTarget::HighRam => self.h_ram.read(address),
            MemoryTarget::InterruptEnableRegister => self.ie.read(address),
        }
    }

    /// Statelessly access the value at the given address. This is for dma transfers and observational use
    pub fn peek(&self, address: Address) -> u8 {
        let device = MemoryTarget::get_device_from_address(address);
        match device {
            MemoryTarget::RomBank00 => self.cartridge.peek(address, CartridgeDevice::RomBank00),
            MemoryTarget::BankableRom => self.cartridge.peek(address, CartridgeDevice::BankableRom),
            MemoryTarget::VideoRam => self.v_ram.peek(address),
            MemoryTarget::ExternalRam => self.cartridge.peek(address, CartridgeDevice::ExternalRam),
            MemoryTarget::WorkRam00 => self.w_ram_00.peek(address),
            MemoryTarget::BankableWorkRam => self.bankable_w_ram.peek(address),
            MemoryTarget::EchoRam => self.peek(address - 0x2000),
            MemoryTarget::ObjectAttributeMemory => self.oam.peek(address),
            MemoryTarget::Unusable => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            MemoryTarget::IoRegisters => self.io_registers.peek(address),
            MemoryTarget::HighRam => self.h_ram.peek(address),
            MemoryTarget::InterruptEnableRegister => self.ie.peek(address),
        }
    }
    pub fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        let device = MemoryTarget::get_device_from_address(address);
        match device {
            MemoryTarget::RomBank00 => self.cartridge.write(address, CartridgeDevice::RomBank00, value),
            MemoryTarget::BankableRom => self.cartridge.write(address, CartridgeDevice::BankableRom, value),
            MemoryTarget::VideoRam => self.v_ram.write(address, value),
            MemoryTarget::ExternalRam => self.cartridge.write(address, CartridgeDevice::ExternalRam, value),
            MemoryTarget::WorkRam00 => self.w_ram_00.write(address, value),
            MemoryTarget::BankableWorkRam => self.bankable_w_ram.write(address, value),
            MemoryTarget::EchoRam => self.write(address - 0x2000, value),
            MemoryTarget::ObjectAttributeMemory => self.oam.write(address, value),
            MemoryTarget::Unusable => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            MemoryTarget::IoRegisters => self.io_registers.write(address, value),
            MemoryTarget::HighRam => self.h_ram.write(address, value),
            MemoryTarget::InterruptEnableRegister => self.ie.write(address, value),
        }
    }

    pub fn set_active_bank_number(&mut self, device: MemoryTarget, bank_num: u8) {
        match device {
            MemoryTarget::BankableRom => todo!(),
            MemoryTarget::VideoRam => todo!(),
            MemoryTarget::ExternalRam => todo!(),
            MemoryTarget::BankableWorkRam => self.bankable_w_ram.set_active_bank_number(bank_num),
            _ => unreachable!("There shouldn't be any instances where this is called. All bankable memory is above."),
        }
    }

    pub fn set_object_priority_mode(&mut self, mode: PriorityMode) {
        self.oam.set_priority_mode(mode)
    }

    pub fn oam_dma_transfer(&mut self, address: Address, value: u8) {
        self.oam.set_from_dma_transfer(address, value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryTarget {
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

impl MemoryTarget {
    pub const fn get_base_address(&self) -> Address {
        match self {
            MemoryTarget::RomBank00 => 0x0000,
            MemoryTarget::BankableRom => 0x4000,
            MemoryTarget::VideoRam => 0x8000,
            MemoryTarget::ExternalRam => 0xA000,
            MemoryTarget::WorkRam00 => 0xC000,
            MemoryTarget::BankableWorkRam => 0xD000,
            MemoryTarget::EchoRam => 0xE000,
            MemoryTarget::ObjectAttributeMemory => 0xFE00,
            MemoryTarget::Unusable => 0xFEA0,
            MemoryTarget::IoRegisters => 0xFF00,
            MemoryTarget::HighRam => 0xFF80,
            MemoryTarget::InterruptEnableRegister => 0xFFFF,
        }
    }
    pub const fn get_end_address(&self) -> Address {
        match self {
            MemoryTarget::RomBank00 => 0x4000,
            MemoryTarget::BankableRom => 0x8000,
            MemoryTarget::VideoRam => 0xA000,
            MemoryTarget::ExternalRam => 0xC000,
            MemoryTarget::WorkRam00 => 0xD000,
            MemoryTarget::BankableWorkRam => 0xE000,
            MemoryTarget::EchoRam => 0xFE00,
            MemoryTarget::ObjectAttributeMemory => 0xFEA0,
            MemoryTarget::Unusable => 0xFF00,
            MemoryTarget::IoRegisters => 0xFF80,
            MemoryTarget::HighRam => 0xFFFF,
            MemoryTarget::InterruptEnableRegister => 0xFFFF,
        }
    }
    pub const fn get_device_from_address(address: Address) -> Self {
        let enumerated_devices: &[MemoryTarget] = &[
            MemoryTarget::RomBank00,
            MemoryTarget::BankableRom,
            MemoryTarget::VideoRam,
            MemoryTarget::ExternalRam,
            MemoryTarget::WorkRam00,
            MemoryTarget::BankableWorkRam, // GameBoy Color Only
            MemoryTarget::EchoRam,         // Mirror of C000-DDFF
            MemoryTarget::ObjectAttributeMemory,
            MemoryTarget::Unusable,
            MemoryTarget::IoRegisters,
            MemoryTarget::HighRam,
        ];

        let mut i = 0;
        while i < enumerated_devices.len() {
            if enumerated_devices[i].get_end_address() > address {
                return enumerated_devices[i];
            }
            i += 1;
        }
        // only one that doesn't fit in there is IE Register
        MemoryTarget::InterruptEnableRegister
    }
}

pub trait BusAccessible {
    const MM_DEVICE: MemoryTarget;

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

pub struct BusAccessOutcome<T>(pub T, pub Vec<Change>);

impl<T: BusDefault> BusAccessOutcome<T> {
    pub fn default_outcome() -> Self {
        Self(T::DEFAULT_BUS_VALUE, vec![])
    }
}
pub enum BusAccessFailure {
    NothingMappedToAddress,
    InaccessbileInPpuMode,
    TriedAccessingUnusableMemory,
    TriedWritingToRom,
    Unimplemented,
}

impl Display for BusAccessFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BusAccessFailure::NothingMappedToAddress => "NothingMappedToAddress",
            BusAccessFailure::InaccessbileInPpuMode => "InaccessbileInPpuMode",
            BusAccessFailure::TriedAccessingUnusableMemory => "TriedAccessingUnusableMemory",
            BusAccessFailure::TriedWritingToRom => "TriedWritingToRom",
            BusAccessFailure::Unimplemented => "NotImplemented",
        };
        f.write_str(str)
    }
}

/// The return value for reads to inaccessible devices.
/// Just standardizing our garbage and removing mystical numbers.
pub const INACCESIBLE_RETURN_VALUE: u8 = 0xFF;

impl From<u8> for BusAccessOutcome<u8> {
    fn from(value: u8) -> Self {
        Self(value, vec![])
    }
}

impl From<u16> for BusAccessOutcome<u16> {
    fn from(value: u16) -> Self {
        Self(value, vec![])
    }
}

impl From<()> for BusAccessOutcome<()> {
    fn from(value: ()) -> Self {
        Self(value, vec![])
    }
}

impl<T: BusDefault> From<BusAccessFailure> for BusAccessOutcome<T> {
    fn from(failure: BusAccessFailure) -> Self {
        // log(&format!("{}", failure));
        Self(T::DEFAULT_BUS_VALUE, vec![])
    }
}

impl From<BusAccessFailure> for u8 {
    fn from(access_failure: BusAccessFailure) -> Self {
        // log(&format!("{}", access_failure));
        u8::DEFAULT_BUS_VALUE
    }
}

pub trait BusDefault {
    const DEFAULT_BUS_VALUE: Self;
}

impl BusDefault for u8 {
    const DEFAULT_BUS_VALUE: Self = INACCESIBLE_RETURN_VALUE;
}
impl BusDefault for u16 {
    const DEFAULT_BUS_VALUE: Self = 0xFFFF;
}
impl BusDefault for InstructionOutcome {
    const DEFAULT_BUS_VALUE: Self = InstructionOutcome::Ok;
}
impl BusDefault for () {
    const DEFAULT_BUS_VALUE: Self = ();
}
