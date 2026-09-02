use crate::{
    bus::Address,
    game_boy::{EventQueue, GameBoyEvent},
    io_devices::interrupts::Interrupt,
};

pub struct TimerDivider {
    divider: u16,
    timer: u8,
    timer_reset_value: u8,
    timer_control: u8,
    mode: TimerDividerMode,
    timer_interrupt_primed: bool, // used to handle delay for timer interrupts
    last_timer_signal: bool,      // last state for edge detection to tick timer
    reloading: bool,
}

impl TimerDivider {
    const START_ADDRESS: Address = 0xFF04;

    pub fn new() -> Self {
        Self {
            divider: 0,
            timer: 0,
            timer_reset_value: 0,
            timer_control: 0,
            mode: TimerDividerMode::Dmg,
            timer_interrupt_primed: false,
            last_timer_signal: false,
            reloading: false,
        }
    }

    fn get_local_address(address: Address) -> Address {
        address - Self::START_ADDRESS
    }

    pub fn tick(&mut self, events: &mut EventQueue) {
        self.reloading = false;
        self.handle_timer_interrupt_delay(events);
        let prev_counter = self.divider;

        self.divider = self.divider.wrapping_add(4);

        self.check_edge_for_div_apu(prev_counter, events);
        self.check_edge_for_timer();
    }

    pub fn reset_divider_register(&mut self) {
        self.divider = 0;
        self.check_edge_for_timer();
    }

    fn check_edge_for_div_apu(&mut self, prev_counter: u16, events: &mut EventQueue) {
        if self.falling_edge_detection(prev_counter, self.mode.div_apu_event_bit_number()) {
            events.push(GameBoyEvent::DivApuEvent)
        }
    }

    fn check_edge_for_timer(&mut self) {
        let current_state = self.get_timer_signal();
        if self.last_timer_signal && !current_state {
            self.tick_timer();
        }
        self.last_timer_signal = current_state;
    }

    fn get_timer_signal(&self) -> bool {
        let bit = match self.timer_control & 0b11 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        };
        ((self.divider >> bit) & 0b1 == 1) && (self.timer_control & 0b100 != 0)
    }

    fn tick_timer(&mut self) {
        self.timer = match self.timer.checked_add(1) {
            Some(t) => t,
            None => {
                self.timer_interrupt_primed = true;
                0
            },
        };
    }

    pub fn read(&mut self, address: Address) -> u8 {
        self.peek(address)
    }

    pub fn peek(&self, address: Address) -> u8 {
        match Self::get_local_address(address) {
            0 => (self.divider >> 8) as u8,
            1 => self.timer,
            2 => self.timer_reset_value,
            3 => self.timer_control | 0xF8,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: Address, value: u8, events: &mut EventQueue) {
        match Self::get_local_address(address) {
            0 => self.write_to_div(events),
            1 => self.write_to_timer(value),
            2 => self.write_to_tma(value),
            3 => self.write_to_tac(value),
            _ => unreachable!(),
        }
    }

    fn write_to_div(&mut self, events: &mut EventQueue) {
        let prev_counter = self.divider;

        self.divider = 0;

        self.check_edge_for_div_apu(prev_counter, events);
        self.check_edge_for_timer();
    }

    fn write_to_tac(&mut self, value: u8) {
        self.timer_control = value;
        self.check_edge_for_timer();
    }

    fn write_to_tma(&mut self, value: u8) {
        self.timer_reset_value = value;
        if self.reloading {
            self.timer = value;
        }
    }

    fn write_to_timer(&mut self, value: u8) {
        if self.reloading {
            return;
        }
        self.timer_interrupt_primed = false;
        self.timer = value;
    }

    fn handle_timer_interrupt_delay(&mut self, events: &mut EventQueue) {
        if self.timer_interrupt_primed {
            self.timer_interrupt_primed = false;
            self.timer = self.timer_reset_value;
            self.reloading = true;
            events.push(GameBoyEvent::Interrupt(Interrupt::Timer));
        }
    }

    fn falling_edge_detection(&self, prev_counter: u16, bit_number: u8) -> bool {
        (prev_counter >> bit_number) & 0b1 == 1 && (self.divider >> bit_number) & 0b1 == 0
    }
}

impl Default for TimerDivider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq)]
enum TimerDividerMode {
    Dmg,
    Cgb { double_speed: bool },
}

impl TimerDividerMode {
    fn div_apu_event_bit_number(&self) -> u8 {
        match self {
            TimerDividerMode::Cgb { double_speed: true } => 13,
            _ => 12,
        }
    }
}
