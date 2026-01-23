use crate::{
    bus::{Address, BusAccessFailure, BusAccessOutcome, BusAccessible, MMDevice},
    game_boy::Change,
    graphics::lcd::LcdRegisters,
};

#[derive(Default)]
pub struct IoRegisters {
    lcd_registers: LcdRegisters,
}

impl IoRegisters {}

impl BusAccessible for IoRegisters {
    const MM_DEVICE: MMDevice = MMDevice::IoRegisters;

    fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };
        match section {
            IoSection::JoypadInput => BusAccessOutcome::default_outcome(),
            IoSection::SerialTransfer => BusAccessOutcome::default_outcome(),
            IoSection::TimerAndDivider => BusAccessOutcome::default_outcome(),
            IoSection::Interrupts => BusAccessOutcome::default_outcome(),
            IoSection::Audio => BusAccessOutcome::default_outcome(),
            IoSection::WavePattern => BusAccessOutcome::default_outcome(),
            IoSection::Lcd => self.lcd_registers.read(address),
            IoSection::OamDmaTransfer => BusAccessOutcome::default_outcome(),
            IoSection::Keys => BusAccessOutcome::default_outcome(),
            IoSection::VramDma => BusAccessOutcome::default_outcome(),
            IoSection::BootRomMappingControl => {
                BusAccessOutcome(u8::from(BusAccessFailure::TriedAccessingUnusableMemory), vec![])
            },
            IoSection::Ir => BusAccessOutcome::default_outcome(),
            IoSection::BgObjPalettes => BusAccessOutcome::default_outcome(),
            IoSection::ObjectPriorityMode => BusAccessOutcome::default_outcome(),
            IoSection::WramBankSelect => BusAccessOutcome::default_outcome(),
        }
    }

    fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };
        match section {
            IoSection::JoypadInput => BusAccessOutcome::default_outcome(),
            IoSection::SerialTransfer => BusAccessOutcome::default_outcome(),
            IoSection::TimerAndDivider => BusAccessOutcome::default_outcome(),
            IoSection::Interrupts => BusAccessOutcome::default_outcome(),
            IoSection::Audio => BusAccessOutcome::default_outcome(),
            IoSection::WavePattern => BusAccessOutcome::default_outcome(),
            IoSection::Lcd => self.lcd_registers.write(address, value),
            IoSection::OamDmaTransfer => BusAccessOutcome::default_outcome(),
            IoSection::Keys => BusAccessOutcome::default_outcome(),
            IoSection::VramDma => BusAccessOutcome::default_outcome(),
            IoSection::BootRomMappingControl => BusAccessOutcome((), vec![Change::UnmapBootRom]),
            IoSection::Ir => BusAccessOutcome::default_outcome(),
            IoSection::BgObjPalettes => BusAccessOutcome::default_outcome(),
            IoSection::ObjectPriorityMode => BusAccessOutcome::default_outcome(),
            IoSection::WramBankSelect => BusAccessOutcome::default_outcome(),
        }
    }

    fn peek(&self, address: Address) -> u8 {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };
        match section {
            IoSection::JoypadInput => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::SerialTransfer => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::TimerAndDivider => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::Interrupts => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::Audio => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::WavePattern => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::Lcd => self.lcd_registers.peek(address),
            IoSection::OamDmaTransfer => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::Keys => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::VramDma => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::BootRomMappingControl => u8::from(BusAccessFailure::TriedAccessingUnusableMemory),
            IoSection::Ir => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::BgObjPalettes => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::ObjectPriorityMode => BusAccessFailure::NothingMappedToAddress.into(),
            IoSection::WramBankSelect => BusAccessFailure::NothingMappedToAddress.into(),
        }
    }
}

const IO_MAP: &[(IoSection, (Address, Address))] = &[
    (IoSection::JoypadInput, (0xFF00, 0xFF01)),
    (IoSection::SerialTransfer, (0xFF01, 0xFF03)),
    (IoSection::TimerAndDivider, (0xFF04, 0xFF08)),
    (IoSection::Interrupts, (0xFF0F, 0xFF10)),
    (IoSection::Audio, (0xFF10, 0xFF27)),
    (IoSection::WavePattern, (0xFF30, 0xFF40)),
    (IoSection::OamDmaTransfer, (0xFF46, 0xFF47)),
    (IoSection::Lcd, (0xFF40, 0xFF4C)),
    (IoSection::Keys, (0xFF4C, 0xFF4E)),
    (IoSection::VramDma, (0xFF4F, 0xFF50)),
    (IoSection::BootRomMappingControl, (0xFF50, 0xFF51)),
    (IoSection::Ir, (0xFF56, 0xFF57)),
    (IoSection::BgObjPalettes, (0xFF68, 0xFF6C)),
    (IoSection::ObjectPriorityMode, (0xFF6C, 0xFF6D)),
    (IoSection::WramBankSelect, (0xFF70, 0xFF71)),
];

#[repr(u8)]
#[derive(Copy, Clone)]
enum IoSection {
    JoypadInput = 0,
    SerialTransfer,
    TimerAndDivider,
    Interrupts,
    Audio,
    WavePattern,
    Lcd,
    OamDmaTransfer,
    Keys,
    VramDma,
    BootRomMappingControl,
    Ir,
    BgObjPalettes,
    ObjectPriorityMode,
    WramBankSelect,
}

impl IoSection {
    /// Gets the range (global) for a IO Register section.
    const fn get_range(&self) -> (Address, Address) {
        IO_MAP[*self as usize].1
    }

    fn from_address(address: Address) -> Option<Self> {
        let mut i = 0;
        while i < IO_MAP.len() {
            if IO_MAP[i].1.0 <= address && IO_MAP[i].1.1 > address {
                return Some(IO_MAP[i].0);
            }
            i += 1;
        }

        None // This just means it's accessing parts of the IO space not mapped to anything
    }
}
