use crate::bus::{Address, BusAccessFailure};

pub struct TimerDivider {
    registers: [u8; 4],
}

impl TimerDivider {
    const START_ADDRESS: Address = 0xFF04;
    fn get_local_address(address: Address) -> Address {
        address - Self::START_ADDRESS
    }
    pub fn get(&self, address: Address) -> u8 {
        self.registers[TimerDivider::get_local_address(address) as usize]
    }
    pub fn set(&mut self, address: Address, value: u8) {
        self.set_register(TimerDividerRegister::from_global_address(address), value);
    }
    pub fn get_register(&self, register: TimerDividerRegister) -> u8 {
        self.registers[register.get_index() as usize]
    }

    pub fn set_register(&mut self, register: TimerDividerRegister, value: u8) {
        match register {
            TimerDividerRegister::Divider => BusAccessFailure::TriedWritingToReadOnlyMemory.into(),
            TimerDividerRegister::Counter => BusAccessFailure::TriedWritingToReadOnlyMemory.into(),
            _ => self.registers[register.get_index()] = value,
        }
    }
}

impl Default for TimerDivider {
    fn default() -> Self {
        Self { registers: Default::default() }
    }
}

pub enum TimerDividerRegister {
    Divider,
    Counter,
    Modulo,
    Control,
}

impl TimerDividerRegister {
    fn get_index(&self) -> usize {
        match self {
            TimerDividerRegister::Divider => 0,
            TimerDividerRegister::Counter => 1,
            TimerDividerRegister::Modulo => 2,
            TimerDividerRegister::Control => 3,
        }
    }
    fn from_global_address(address: Address) -> Self {
        match TimerDivider::get_local_address(address) {
            0 => Self::Divider,
            1 => Self::Counter,
            2 => Self::Modulo,
            3 => Self::Control,
            _ => unreachable!(),
        }
    }
}

impl From<Address> for TimerDividerRegister {
    fn from(address: Address) -> Self {
        Self::from_global_address(address)
    }
}
