use crate::{
    bus::{Address, BusAccessOutcome, BusAccessible, MMDevice},
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
        println!("Reading from 0x{address:04x}");
        match IoSection::from_address(address) {
            IoSection::JoypadInput => todo!(),
            IoSection::SerialTransfer => todo!(),
            IoSection::TimerAndDivider => todo!(),
            IoSection::Interrupts => todo!(),
            IoSection::Audio => todo!(),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.read(address),
            IoSection::OamDmaTransfer => todo!(),
            IoSection::Keys => todo!(),
            IoSection::VramDma => todo!(),
            IoSection::BootRomMappingControl => todo!(),
            IoSection::Ir => todo!(),
            IoSection::BgObjPalettes => todo!(),
            IoSection::ObjectPriorityMode => todo!(),
            IoSection::WramBankSelect => todo!(),
        }
    }

    fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        println!("Writing 0x{value:02x} to 0x{address:04x}");
        match IoSection::from_address(address) {
            IoSection::JoypadInput => todo!(),
            IoSection::SerialTransfer => todo!(),
            IoSection::TimerAndDivider => todo!(),
            IoSection::Interrupts => todo!(),
            IoSection::Audio => todo!(),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.write(address, value),
            IoSection::OamDmaTransfer => todo!(),
            IoSection::Keys => todo!(),
            IoSection::VramDma => todo!(),
            IoSection::BootRomMappingControl => todo!(),
            IoSection::Ir => todo!(),
            IoSection::BgObjPalettes => todo!(),
            IoSection::ObjectPriorityMode => todo!(),
            IoSection::WramBankSelect => todo!(),
        }
    }

    fn peek(&self, address: Address) -> u8 {
        match IoSection::from_address(address) {
            IoSection::JoypadInput => todo!(),
            IoSection::SerialTransfer => todo!(),
            IoSection::TimerAndDivider => todo!(),
            IoSection::Interrupts => todo!(),
            IoSection::Audio => todo!(),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.peek(address),
            IoSection::OamDmaTransfer => todo!(),
            IoSection::Keys => todo!(),
            IoSection::VramDma => todo!(),
            IoSection::Ir => todo!(),
            IoSection::BgObjPalettes => todo!(),
            IoSection::ObjectPriorityMode => todo!(),
            IoSection::WramBankSelect => todo!(),
            IoSection::BootRomMappingControl => todo!(),
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

    fn from_address(address: Address) -> Self {
        let mut i = 0;
        while i < IO_MAP.len() {
            if IO_MAP[i].1.0 <= address && IO_MAP[i].1.1 > address {
                return IO_MAP[i].0;
            }
            i += 1;
        }

        panic!("This should cover ever address that the bus would send here.")
    }
}
