use std::fmt::Display;

use crate::{
    cartridge::cartridge::{Cartridge, CartridgeDevice},
    game_boy::GameBoyStateChange,
    graphics::{
        lcd::LcdRegisters,
        oam::{ObjectAttributeMemory, PriorityMode},
        ppu::PpuTickMode,
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
    memory_map: MemoryMap,
}

impl Bus {
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
    }

    pub fn handle_memory_map_event(&mut self, event: MemoryMapEvent) {
        self.memory_map.handle_memory_map_event(event)
    }

    fn get_cpu_accessible_device_from_address(&self, address: Address) -> Option<MemoryTarget> {
        self.memory_map.get_cpu_accessible_device_from_address(address)
    }

    fn get_device_from_address(&self, address: Address) -> MemoryTarget {
        self.memory_map.get_device_from_address(address)
    }

    pub fn get_ppu_context_mem(&mut self) -> (&mut VideoRam, &mut ObjectAttributeMemory, &mut LcdRegisters) {
        (&mut self.v_ram, &mut self.oam, &mut self.io_registers.lcd_registers)
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
        let Some(device) = self.get_cpu_accessible_device_from_address(address) else {
            return BusAccessFailure::InaccessibleByCpu.into();
        };
        match device {
            MemoryTarget::BootRom => self.boot_rom.read(address),
            MemoryTarget::RomBank00 => self.cartridge.read(address, CartridgeDevice::RomBank00),
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
        let device = self.get_device_from_address(address);
        match device {
            MemoryTarget::BootRom => self.boot_rom.read(address).0,
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
        let Some(device) = self.get_cpu_accessible_device_from_address(address) else {
            return BusAccessFailure::InaccessibleByCpu.into();
        };
        // if device == MemoryTarget::VideoRam {
        //     println!("VRAM Address: {address:04x}");
        // }
        match device {
            MemoryTarget::BootRom => self.boot_rom.write(address, value),
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
    BootRom,
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
            MemoryTarget::BootRom => 0x0000,
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
    pub const fn get_end_address_inclusive(&self) -> Address {
        match self {
            MemoryTarget::BootRom => 0x0FF,
            MemoryTarget::RomBank00 => 0x3FFF,
            MemoryTarget::BankableRom => 0x7FFF,
            MemoryTarget::VideoRam => 0x9FFF,
            MemoryTarget::ExternalRam => 0xBFFF,
            MemoryTarget::WorkRam00 => 0xCFFF,
            MemoryTarget::BankableWorkRam => 0xDFFF,
            MemoryTarget::EchoRam => 0xFDFF,
            MemoryTarget::ObjectAttributeMemory => 0xFE9F,
            MemoryTarget::Unusable => 0xFEFF,
            MemoryTarget::IoRegisters => 0xFF7F,
            MemoryTarget::HighRam => 0xFFFE,
            MemoryTarget::InterruptEnableRegister => 0xFFFF,
        }
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
        Self::MM_DEVICE.get_end_address_inclusive()
    }

    fn read(&mut self, address: Address) -> BusAccessOutcome<u8>;
    fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()>;
    fn peek(&self, address: Address) -> u8;
}

struct MemoryMap {
    map: [MemoryMapEntry; 13],
}

impl MemoryMap {
    fn handle_memory_map_event(&mut self, event: MemoryMapEvent) {
        match event {
            MemoryMapEvent::UnmapBootRom => {
                self.map[0].device = MemoryTarget::RomBank00;
            },
            MemoryMapEvent::UpdatePpuMode(ppu_tick_mode) => {
                let inaccessible = ppu_tick_mode.get_cpu_inaccessible_video_targets();
                let accessible = ppu_tick_mode.get_cpu_accessible_video_targets();
                self.set_devices_ppu_inaccessible(&inaccessible, true);
                self.set_devices_ppu_inaccessible(&accessible, false);
            },
            MemoryMapEvent::StartOamDataTransfer => self.set_devices_dma_inaccessible(
                &[
                    MemoryTarget::BootRom,
                    MemoryTarget::RomBank00,
                    MemoryTarget::BankableRom,
                    MemoryTarget::VideoRam,
                    MemoryTarget::ExternalRam,
                    MemoryTarget::WorkRam00,
                    MemoryTarget::BankableWorkRam,
                    MemoryTarget::EchoRam,
                    MemoryTarget::ObjectAttributeMemory,
                    MemoryTarget::Unusable,
                    MemoryTarget::IoRegisters,
                ],
                true,
            ),
            MemoryMapEvent::EndOamDataTransfer => self.set_devices_dma_inaccessible(
                &[
                    MemoryTarget::BootRom,
                    MemoryTarget::RomBank00,
                    MemoryTarget::BankableRom,
                    MemoryTarget::VideoRam,
                    MemoryTarget::ExternalRam,
                    MemoryTarget::WorkRam00,
                    MemoryTarget::BankableWorkRam,
                    MemoryTarget::EchoRam,
                    MemoryTarget::ObjectAttributeMemory,
                    MemoryTarget::Unusable,
                    MemoryTarget::IoRegisters,
                ],
                false,
            ),
        }
    }

    fn get_mapping_from_address(&self, address: Address) -> &MemoryMapEntry {
        let index = match address {
            0x0000..=0x00FF => 0,
            0x0100..=0x3FFF => 1,
            0x4000..=0x7FFF => 2,
            0x8000..=0x9FFF => 3,
            0xA000..=0xBFFF => 4,
            0xC000..=0xCFFF => 5,
            0xD000..=0xDFFF => 6,
            0xE000..=0xFDFF => 7,
            0xFE00..=0xFE9F => 8,
            0xFEA0..=0xFEFF => 9,
            0xFF00..=0xFF7F => 10,
            0xFF80..=0xFFFE => 11,
            0xFFFF => 12,
        };
        &self.map[index]
    }

    fn get_device_from_address(&self, address: Address) -> MemoryTarget {
        self.get_mapping_from_address(address).device
    }

    fn get_cpu_accessible_device_from_address(&self, address: Address) -> Option<MemoryTarget> {
        let device = self.get_mapping_from_address(address);
        if device.inaccessible() {
            return None;
        } else {
            return Some(device.device);
        }
    }

    fn get_device_index(device: MemoryTarget) -> usize {
        match device {
            MemoryTarget::BootRom => 0,
            MemoryTarget::RomBank00 => 1,
            MemoryTarget::BankableRom => 2,
            MemoryTarget::VideoRam => 3,
            MemoryTarget::ExternalRam => 4,
            MemoryTarget::WorkRam00 => 5,
            MemoryTarget::BankableWorkRam => 6,
            MemoryTarget::EchoRam => 7,
            MemoryTarget::ObjectAttributeMemory => 8,
            MemoryTarget::Unusable => 9,
            MemoryTarget::IoRegisters => 10,
            MemoryTarget::HighRam => 11,
            MemoryTarget::InterruptEnableRegister => 12,
        }
    }

    fn get_mapping_mut(&mut self, device: MemoryTarget) -> &mut MemoryMapEntry {
        &mut self.map[Self::get_device_index(device)]
    }
    fn set_devices_ppu_inaccessible(&mut self, devices: &[MemoryTarget], inaccessible: bool) {
        for device in devices {
            self.set_device_ppu_inaccessible(*device, inaccessible);
        }
    }
    fn set_devices_dma_inaccessible(&mut self, devices: &[MemoryTarget], inaccessible: bool) {
        for device in devices {
            self.set_device_dma_inaccessible(*device, inaccessible);
        }
    }
    fn set_device_ppu_inaccessible(&mut self, device: MemoryTarget, inaccessible: bool) {
        self.get_mapping_mut(device).inaccessible_due_to_ppu = inaccessible
    }
    fn set_device_dma_inaccessible(&mut self, device: MemoryTarget, inaccessible: bool) {
        self.get_mapping_mut(device).inaccessible_due_to_dma = inaccessible
    }
}
impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            map: [
                MemoryMapEntry::new(MemoryTarget::BootRom),
                MemoryMapEntry::new(MemoryTarget::RomBank00),
                MemoryMapEntry::new(MemoryTarget::BankableRom),
                MemoryMapEntry::new(MemoryTarget::VideoRam),
                MemoryMapEntry::new(MemoryTarget::ExternalRam),
                MemoryMapEntry::new(MemoryTarget::WorkRam00),
                MemoryMapEntry::new(MemoryTarget::BankableWorkRam),
                MemoryMapEntry::new(MemoryTarget::EchoRam),
                MemoryMapEntry::new(MemoryTarget::ObjectAttributeMemory),
                MemoryMapEntry::new(MemoryTarget::Unusable),
                MemoryMapEntry::new(MemoryTarget::IoRegisters),
                MemoryMapEntry::new(MemoryTarget::HighRam),
                MemoryMapEntry::new(MemoryTarget::InterruptEnableRegister),
            ],
        }
    }
}
struct MemoryMapEntry {
    device: MemoryTarget,
    inaccessible_due_to_ppu: bool,
    inaccessible_due_to_dma: bool,
}
impl MemoryMapEntry {
    fn new(target: MemoryTarget) -> Self {
        Self {
            device: target,
            inaccessible_due_to_ppu: false,
            inaccessible_due_to_dma: false,
        }
    }
    fn inaccessible(&self) -> bool {
        self.inaccessible_due_to_dma | self.inaccessible_due_to_ppu
    }
}

pub type Address = u16;

pub struct BusAccessOutcome<T>(pub T, pub Vec<GameBoyStateChange>);

impl<T: BusDefault> BusAccessOutcome<T> {
    pub fn default_outcome() -> Self {
        Self(T::DEFAULT_BUS_VALUE, vec![])
    }
}
pub enum BusAccessFailure {
    NothingMappedToAddress,
    InaccessibleByCpu,
    TriedAccessingUnusableMemory,
    TriedWritingToReadOnlyMemory,
    Unimplemented,
}

impl Display for BusAccessFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BusAccessFailure::NothingMappedToAddress => "NothingMappedToAddress",
            BusAccessFailure::InaccessibleByCpu => "InaccessibleByCpu",
            BusAccessFailure::TriedAccessingUnusableMemory => "TriedAccessingUnusableMemory",
            BusAccessFailure::TriedWritingToReadOnlyMemory => "TriedWritingToRom",
            BusAccessFailure::Unimplemented => "NotImplemented",
        };
        f.write_str(str)
    }
}

#[derive(Debug)]
pub enum MemoryMapEvent {
    UnmapBootRom,
    UpdatePpuMode(PpuTickMode),
    StartOamDataTransfer,
    EndOamDataTransfer,
}

enum CpuAccessibilityTarget {
    Cartridge,
    VideoRam,
    WorkRam,
    ObjectAttributeMemory,
    IoRegisters,
}

/// The return value for reads to inaccessible devices.
/// Just standardizing our garbage and removing mystical numbers.
pub const INACCESSIBLE_RETURN_VALUE: u8 = 0xFF;

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
        // log(format_args!("{}", access_failure));
        Self(T::DEFAULT_BUS_VALUE, vec![])
    }
}

impl From<BusAccessFailure> for u8 {
    fn from(failure: BusAccessFailure) -> Self {
        // log(format_args!("{}", access_failure));
        u8::DEFAULT_BUS_VALUE
    }
}

pub trait BusDefault {
    const DEFAULT_BUS_VALUE: Self;
}

impl BusDefault for u8 {
    const DEFAULT_BUS_VALUE: Self = INACCESSIBLE_RETURN_VALUE;
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
