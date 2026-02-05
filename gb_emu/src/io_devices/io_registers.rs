use std::sync::{Arc, atomic::AtomicU8};

use crate::{
    bus::{Address, BusAccessFailure, BusAccessible, MemoryTarget},
    game_boy::{GameBoyEvent, notate_event},
    graphics::{lcd::Lcd, oam::PriorityMode},
    io_devices::{
        audio::Audio,
        interrupts::InterruptFlagRegister,
        joypad_input::{ButtonInput, JoyPadInput},
        timer_divider::TimerDivider,
    },
};

pub struct IoRegisters {
    pub lcd_registers: Lcd,
    pub interrupt_flag_register: InterruptFlagRegister,
    pub audio: Audio,
    pub timer_divider: TimerDivider,
    pub joypad: JoyPadInput,
}

impl IoRegisters {
    pub fn new(button_input: ButtonInput) -> Self {
        Self {
            joypad: JoyPadInput::new(button_input),
            lcd_registers: Default::default(),
            interrupt_flag_register: Default::default(),
            audio: Default::default(),
            timer_divider: Default::default(),
        }
    }
}

impl BusAccessible for IoRegisters {
    const MM_DEVICE: MemoryTarget = MemoryTarget::IoRegisters;

    fn read(&mut self, address: Address) -> u8 {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };

        match section {
            IoSection::JoypadInput => self.joypad.read(),
            IoSection::SerialTransfer => BusAccessFailure::Unimplemented.into(),
            IoSection::TimerAndDivider => self.timer_divider.get(address),
            IoSection::Interrupts => self.interrupt_flag_register.get(),
            IoSection::Audio => self.audio.get(address),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.read(address),
            IoSection::Keys => todo!(),
            IoSection::VramBankSelect => todo!(),
            IoSection::BootRomMappingControl => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            IoSection::Ir => todo!(),
            IoSection::BgObjPalettes => todo!(),
            IoSection::ObjectPriorityMode => todo!(),
            IoSection::WramBankSelect => todo!(),
            IoSection::VramDma => todo!(),
        }
    }

    fn write(&mut self, address: Address, value: u8) {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };

        match section {
            IoSection::JoypadInput => self.joypad.write(value),
            IoSection::SerialTransfer => BusAccessFailure::Unimplemented.into(),
            IoSection::TimerAndDivider => self.timer_divider.set(address, value),
            IoSection::Interrupts => self.interrupt_flag_register.set(value),
            IoSection::Audio => self.audio.set(address, value),
            IoSection::WavePattern => BusAccessFailure::Unimplemented.into(),
            IoSection::Lcd => self.lcd_registers.write(address, value),
            IoSection::Keys => BusAccessFailure::Unimplemented.into(),
            IoSection::VramBankSelect => BusAccessFailure::Unimplemented.into(),
            IoSection::BootRomMappingControl => notate_event(GameBoyEvent::UnmapBootRom),
            IoSection::Ir => BusAccessFailure::Unimplemented.into(),
            IoSection::BgObjPalettes => BusAccessFailure::Unimplemented.into(),
            IoSection::ObjectPriorityMode => {
                notate_event(GameBoyEvent::ChangeObjectPriorityMode(PriorityMode::from(value)))
            },
            IoSection::WramBankSelect => BusAccessFailure::Unimplemented.into(),
            IoSection::VramDma => BusAccessFailure::Unimplemented.into(),
        }
    }

    fn peek(&self, address: Address) -> u8 {
        let Some(section) = IoSection::from_address(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };
        match section {
            IoSection::JoypadInput => self.joypad.read(),
            IoSection::SerialTransfer => BusAccessFailure::Unimplemented.into(),
            IoSection::TimerAndDivider => self.timer_divider.get(address),
            IoSection::Interrupts => self.interrupt_flag_register.get(),
            IoSection::Audio => self.audio.get(address),
            IoSection::WavePattern => todo!(),
            IoSection::Lcd => self.lcd_registers.peek(address),
            IoSection::Keys => todo!(),
            IoSection::VramBankSelect => todo!(),
            IoSection::BootRomMappingControl => u8::from(BusAccessFailure::TriedAccessingUnusableMemory),
            IoSection::Ir => todo!(),
            IoSection::BgObjPalettes => todo!(),
            IoSection::ObjectPriorityMode => todo!(),
            IoSection::WramBankSelect => todo!(),
            IoSection::VramDma => todo!(),
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
