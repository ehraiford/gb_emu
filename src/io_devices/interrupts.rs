use crate::bus::{Address, BusAccessible, MemoryTarget};

#[derive(Default)]
pub struct InterruptEnableRegister {
    value: u8,
}

impl InterruptEnableRegister {
    pub fn is_interrupt_triggerable(&self, interrupt: &Interrupt) -> bool {
        (self.value >> interrupt.get_index()) & 0b1 == 1
    }
    pub fn get_value(&self) -> u8 {
        self.value
    }
}

impl BusAccessible for InterruptEnableRegister {
    const MM_DEVICE: MemoryTarget = MemoryTarget::InterruptEnableRegister;

    fn read(&mut self, _: Address) -> u8 {
        self.value
    }

    fn write(&mut self, _: Address, value: u8) {
        self.value = value & 0b0001_1111;
    }

    fn peek(&self, _: Address) -> u8 {
        self.value
    }
}

#[derive(Default)]
pub struct InterruptFlagRegister {
    value: u8,
}

impl InterruptFlagRegister {
    pub fn read(&self) -> u8 {
        self.value
    }

    pub fn write(&mut self, value: u8) {
        self.value = value & 0b0001_1111;
    }

    pub fn raise_flag(&mut self, flag: &Interrupt) {
        self.set_flag(flag, true);
    }

    pub fn lower_flag(&mut self, flag: &Interrupt) {
        self.set_flag(flag, false);
    }

    fn set_flag(&mut self, flag: &Interrupt, flag_value: bool) {
        let index = flag.get_index();
        let mut value = self.value;
        value &= !(1 << index);
        value |= (flag_value as u8) << index;
        self.value = value;
    }

    pub fn try_get_interrupt(&self, ie: u8) -> Option<Interrupt> {
        if self.value & ie == 0 {
            return None;
        }

        for index in 0..5 {
            if ((self.value & ie) >> index) & 0b1 == 1 {
                return Some(Interrupt::from_index(index));
            }
        }
        unreachable!()
    }
}

#[derive(Debug)]
pub enum Interrupt {
    Joypad,
    Serial,
    Timer,
    Stat,
    VBlank,
}

impl Interrupt {
    fn get_index(&self) -> u8 {
        match self {
            Interrupt::Joypad => 4,
            Interrupt::Serial => 3,
            Interrupt::Timer => 2,
            Interrupt::Stat => 1,
            Interrupt::VBlank => 0,
        }
    }
    fn from_index(index: u8) -> Self {
        match index {
            4 => Interrupt::Joypad,
            3 => Interrupt::Serial,
            2 => Interrupt::Timer,
            1 => Interrupt::Stat,
            0 => Interrupt::VBlank,
            _ => unreachable!(),
        }
    }
    pub fn get_isr_address(&self) -> Address {
        match self {
            Interrupt::Joypad => 0x60,
            Interrupt::Serial => 0x58,
            Interrupt::Timer => 0x50,
            Interrupt::Stat => 0x48,
            Interrupt::VBlank => 0x40,
        }
    }
}
