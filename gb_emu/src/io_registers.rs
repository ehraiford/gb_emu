use crate::{
    bus::{Address, BusAccessFailure, BusAccessOutcome, BusAccessible, MemoryTarget},
    game_boy::GameBoyStateChange,
    graphics::{lcd::LcdRegisters, oam::PriorityMode},
};

pub struct IoRegisters {
    pub lcd_registers: LcdRegisters,
}

impl IoRegisters {}

impl BusAccessible for IoRegisters {
    const MM_DEVICE: MemoryTarget = MemoryTarget::IoRegisters;

    fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };

        match section {
            IoSection::JoypadInput => BusAccessFailure::Unimplemented.into(),
            IoSection::SerialTransfer => BusAccessFailure::Unimplemented.into(),
            IoSection::TimerAndDivider => todo!(),
            IoSection::Interrupts => todo!(),
            IoSection::Audio => todo!(),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.read(address),
            IoSection::Keys => todo!(),
            IoSection::VramBankSelect => todo!(),
            IoSection::BootRomMappingControl => {
                BusAccessOutcome(u8::from(BusAccessFailure::TriedAccessingUnusableMemory), vec![])
            },
            IoSection::Ir => todo!(),
            IoSection::BgObjPalettes => BusAccessFailure::Unimplemented.into(),
            IoSection::ObjectPriorityMode => todo!(),
            IoSection::WramBankSelect => todo!(),
            IoSection::VramDma => todo!(),
        }
    }

    fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };

        match section {
            IoSection::JoypadInput => BusAccessFailure::Unimplemented.into(),
            IoSection::SerialTransfer => BusAccessFailure::Unimplemented.into(),
            IoSection::TimerAndDivider => BusAccessFailure::Unimplemented.into(),
            IoSection::Interrupts => BusAccessFailure::Unimplemented.into(),
            IoSection::Audio => BusAccessFailure::Unimplemented.into(),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.write(address, value),
            IoSection::Keys => BusAccessFailure::Unimplemented.into(),
            IoSection::VramBankSelect => BusAccessFailure::Unimplemented.into(),
            IoSection::BootRomMappingControl => BusAccessOutcome((), vec![GameBoyStateChange::UnmapBootRom]),
            IoSection::Ir => BusAccessFailure::Unimplemented.into(),
            IoSection::BgObjPalettes => BusAccessFailure::Unimplemented.into(),
            IoSection::ObjectPriorityMode => BusAccessOutcome(
                (),
                vec![GameBoyStateChange::ChangeObjectPriorityMode(PriorityMode::from(value))],
            ),
            IoSection::WramBankSelect => {
                BusAccessOutcome((), vec![GameBoyStateChange::ChangeSelectedWorkRam(value & 0b111)])
            },
            IoSection::VramDma => todo!(),
        }
    }

    fn peek(&self, address: Address) -> u8 {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };
        match section {
            IoSection::JoypadInput => todo!(),
            IoSection::SerialTransfer => todo!(),
            IoSection::TimerAndDivider => todo!(),
            IoSection::Interrupts => todo!(),
            IoSection::Audio => todo!(),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.peek(address),
            IoSection::Keys => todo!(),
            IoSection::VramBankSelect => todo!(),
            IoSection::BootRomMappingControl => u8::from(BusAccessFailure::TriedAccessingUnusableMemory),
            IoSection::Ir => BusAccessFailure::Unimplemented.into(),
            IoSection::BgObjPalettes => BusAccessFailure::Unimplemented.into(),
            IoSection::ObjectPriorityMode => BusAccessFailure::Unimplemented.into(),
            IoSection::WramBankSelect => BusAccessFailure::Unimplemented.into(),
            IoSection::VramDma => todo!(),
        }
    }
}

impl Default for IoRegisters {
    fn default() -> Self {
        Self { lcd_registers: Default::default() }
    }
}

const IO_MAP: &[(IoSection, (Address, Address))] = &[
    (IoSection::JoypadInput, (0xFF00, 0xFF01)),
    (IoSection::SerialTransfer, (0xFF01, 0xFF03)),
    (IoSection::TimerAndDivider, (0xFF04, 0xFF08)),
    (IoSection::Interrupts, (0xFF0F, 0xFF10)),
    (IoSection::Audio, (0xFF10, 0xFF27)),
    (IoSection::WavePattern, (0xFF30, 0xFF40)),
    (IoSection::Lcd, (0xFF40, 0xFF4C)),
    (IoSection::Keys, (0xFF4C, 0xFF4E)),
    (IoSection::VramBankSelect, (0xFF4F, 0xFF50)),
    (IoSection::BootRomMappingControl, (0xFF50, 0xFF51)),
    (IoSection::Ir, (0xFF56, 0xFF57)),
    (IoSection::VramDma, (0xFF51, 0xFF56)),
    (IoSection::BgObjPalettes, (0xFF68, 0xFF6C)),
    (IoSection::ObjectPriorityMode, (0xFF6C, 0xFF6D)),
    (IoSection::WramBankSelect, (0xFF70, 0xFF71)),
];

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum IoSection {
    JoypadInput = 0,
    SerialTransfer,
    TimerAndDivider,
    Interrupts,
    Audio,
    WavePattern,
    Lcd,
    Keys,
    VramBankSelect,
    BootRomMappingControl,
    VramDma,
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
