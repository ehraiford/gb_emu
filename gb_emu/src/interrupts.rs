use crate::bus::{Address, BusAccessOutcome, BusAccessible, MMDevice};

#[derive(Default)]
pub struct InterruptEnableRegister {
    value: u8,
}

impl InterruptEnableRegister {
    pub fn is_flag_active(&self, flag: InterruptFlag) -> bool {
        (self.value >> flag.get_index()) & 0b1 == 1
    }
}

impl BusAccessible for InterruptEnableRegister {
    const MM_DEVICE: MMDevice = MMDevice::InterruptEnableRegister;

    fn read(&mut self, _: Address) -> BusAccessOutcome<u8> {
        self.value.into()
    }

    fn write(&mut self, _: Address, value: u8) -> BusAccessOutcome<()> {
        self.value = value;
        ().into()
    }

    fn peek(&self, _: Address) -> u8 {
        self.value
    }
}

pub enum InterruptFlag {
    Joypad,
    Serial,
    Timer,
    Lcd,
    VBlank,
}

impl InterruptFlag {
    fn get_index(&self) -> u8 {
        match self {
            InterruptFlag::Joypad => 4,
            InterruptFlag::Serial => 3,
            InterruptFlag::Timer => 2,
            InterruptFlag::Lcd => 1,
            InterruptFlag::VBlank => 0,
        }
    }
}
