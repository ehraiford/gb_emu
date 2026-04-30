use crate::{
    bus::Address,
    game_boy::{GameBoyEvent, notate_event},
    io_devices::interrupts::Interrupt,
};

pub struct TimerDivider {
    registers: [u8; 4],
    is_active: bool,
    m_cycles_to_increment_timer: u16,
    divider_m_cycles: u8,
}

impl TimerDivider {
    const START_ADDRESS: Address = 0xFF04;

    pub fn tick(&mut self) {
        self.tick_divider();

        if self.is_active() {
            self.tick_timer();
        }
    }

    fn tick_divider(&mut self) {
        self.divider_m_cycles += 1;
        if self.divider_m_cycles == 64 {
            self.divider_m_cycles = 0;
            let mut divider = self.get_register(TimerDividerRegister::Divider);
            divider = divider.wrapping_add(1);
            self.registers[TimerDividerRegister::Divider.get_index()] = divider; // manually set it since the normal set method sets it to zero
        }
    }

    fn tick_timer(&mut self) {
        if self.decrement_m_cycles_to_increment_timer() {
            self.increment_timer()
        }
    }

    fn get_local_address(address: Address) -> Address {
        address - Self::START_ADDRESS
    }

    pub fn reset_divider_register(&mut self) {
        self.set_register(TimerDividerRegister::Divider, 0);
    }

    pub fn read(&self, address: Address) -> u8 {
        self.registers[TimerDivider::get_local_address(address) as usize]
    }

    pub fn write(&mut self, address: Address, value: u8) {
        self.set_register(TimerDividerRegister::from_global_address(address), value);
    }

    pub fn get_register(&self, register: TimerDividerRegister) -> u8 {
        self.registers[register.get_index()]
    }

    pub fn set_register(&mut self, register: TimerDividerRegister, value: u8) {
        match register {
            TimerDividerRegister::Divider => self.registers[TimerDividerRegister::Divider.get_index()] = 0,
            TimerDividerRegister::Control => self.set_control_register(value),
            _ => self.registers[register.get_index()] = value,
        }
    }

    fn set_control_register(&mut self, value: u8) {
        self.is_active = (value >> 2) & 0b1 == 1;
        self.registers[TimerDividerRegister::Control.get_index()] = value;
        self.update_m_cycles_to_increment_timer();
    }

    fn decrement_m_cycles_to_increment_timer(&mut self) -> bool {
        self.m_cycles_to_increment_timer -= 1;
        if self.m_cycles_to_increment_timer == 0 {
            self.update_m_cycles_to_increment_timer();
            true
        } else {
            false
        }
    }

    fn increment_timer(&mut self) {
        let timer = self.get_register(TimerDividerRegister::Counter);
        if let Some(new_val) = timer.checked_add(1) {
            self.set_register(TimerDividerRegister::Counter, new_val);
        } else {
            self.timer_overflow();
        }
    }

    fn timer_overflow(&mut self) {
        self.set_register(
            TimerDividerRegister::Counter,
            self.get_register(TimerDividerRegister::Modulo),
        );
        notate_event(GameBoyEvent::Interrupt(Interrupt::Timer));
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn update_m_cycles_to_increment_timer(&mut self) {
        self.m_cycles_to_increment_timer = match self.get_register(TimerDividerRegister::Control) & 0b11 {
            0 => 256,
            1 => 4,
            2 => 16,
            3 => 64,
            _ => unreachable!(),
        };
    }
}

impl Default for TimerDivider {
    fn default() -> Self {
        Self {
            registers: Default::default(),
            is_active: false,
            m_cycles_to_increment_timer: 256,
            divider_m_cycles: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
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
