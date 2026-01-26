use crate::{
    bus::{Address, BusAccessFailure, BusAccessOutcome, BusAccessible, INACCESIBLE_RETURN_VALUE, MemoryTarget},
    game_boy::Change,
    graphics::{lcd::LcdRegisters, oam::PriorityMode},
};

pub struct IoRegisters {
    lcd_registers: LcdRegisters,
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
            IoSection::BootRomMappingControl => BusAccessOutcome((), vec![Change::UnmapBootRom]),
            IoSection::Ir => BusAccessFailure::Unimplemented.into(),
            IoSection::BgObjPalettes => BusAccessFailure::Unimplemented.into(),
            IoSection::ObjectPriorityMode => {
                BusAccessOutcome((), vec![Change::ChangeObjectPriorityMode(PriorityMode::from(value))])
            },
            IoSection::WramBankSelect => BusAccessOutcome((), vec![Change::ChangeSelectedWorkRam(value & 0b111)]),
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

    pub fn get_default_value_mapping(&self) -> Vec<(Address, u8)> {
        match self {
            IoSection::JoypadInput => vec![(0xFF00, 0xCF)],
            IoSection::SerialTransfer => vec![(0xFF01, 0x00), (0xFF02, 0x7E)],
            IoSection::TimerAndDivider => vec![(0xFF04, 0x18), (0xFF05, 0x00), (0xFF06, 0x00), (0xFF07, 0xF8)],
            IoSection::Interrupts => vec![(0xFF0F, 0xE1)],
            IoSection::Audio => vec![
                (0xFF10, 0x80),
                (0xFF11, 0xBF),
                (0xFF12, 0xF3),
                (0xFF13, 0xFF),
                (0xFF14, 0xBF),
                (0xFF16, 0x3F),
                (0xFF17, 0x00),
                (0xFF18, 0xFF),
                (0xFF19, 0xBF),
                (0xFF1A, 0x7F),
                (0xFF1B, 0xFF),
                (0xFF1C, 0x9F),
                (0xFF1D, 0xFF),
                (0xFF1E, 0xBF),
                (0xFF20, 0xFF),
                (0xFF21, 0x00),
                (0xFF22, 0x00),
                (0xFF23, 0xBF),
                (0xFF24, 0x77),
                (0xFF25, 0xF3),
                (0xFF26, 0xF1),
            ],
            IoSection::WavePattern => vec![],
            IoSection::Lcd => vec![
                (0xFF40, 0x91),
                (0xFF41, 0x81),
                (0xFF42, 0x00),
                (0xFF43, 0x00),
                (0xFF44, 0x91),
                (0xFF45, 0x00),
                (0xFF46, 0xFF),
                (0xFF47, 0xFC),
                (0xFF48, INACCESIBLE_RETURN_VALUE),
                (0xFF49, INACCESIBLE_RETURN_VALUE),
                (0xFF4A, 0x00),
                (0xFF4B, 0x00),
            ],
            IoSection::Keys => vec![(0xFF4C, INACCESIBLE_RETURN_VALUE), (0xFF4D, INACCESIBLE_RETURN_VALUE)],
            IoSection::VramBankSelect => vec![(0xFF4F, INACCESIBLE_RETURN_VALUE)],
            IoSection::BootRomMappingControl => vec![(0xFF50, INACCESIBLE_RETURN_VALUE)],
            IoSection::VramDma => vec![
                (0xFF51, INACCESIBLE_RETURN_VALUE),
                (0xFF52, INACCESIBLE_RETURN_VALUE),
                (0xFF53, INACCESIBLE_RETURN_VALUE),
                (0xFF54, INACCESIBLE_RETURN_VALUE),
                (0xFF55, INACCESIBLE_RETURN_VALUE),
            ],
            IoSection::Ir => vec![(0xFF56, INACCESIBLE_RETURN_VALUE)],
            IoSection::BgObjPalettes => vec![
                (0xFF68, INACCESIBLE_RETURN_VALUE),
                (0xFF69, INACCESIBLE_RETURN_VALUE),
                (0xFF6A, INACCESIBLE_RETURN_VALUE),
                (0xFF6B, INACCESIBLE_RETURN_VALUE),
            ],
            IoSection::ObjectPriorityMode => vec![],
            IoSection::WramBankSelect => vec![(0xFF70, INACCESIBLE_RETURN_VALUE)],
        }
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
