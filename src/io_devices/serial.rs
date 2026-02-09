use crate::{
    bus::Address,
    game_boy::{GameBoyEvent, notate_event},
    helpers::{Bit, RingBuffer},
    io_devices::interrupts::Interrupt,
};

const OUTPUT_BUFFER_LENGTH: usize = 0x100;

pub struct Serial {
    data: u8,

    clock_speed: bool, // Only exists on CGB
    clock_select: bool,

    m_cycles_until_transfer: u8,
    output_buffer: RingBuffer<Bit, OUTPUT_BUFFER_LENGTH>,
    bits_left_in_transfer: u8, // This doubles as the TRANSFER ENABLE Flag
}

impl Serial {
    const START_ADDRESS: Address = 0xFF01;
    const M_CYCLES_IN_A_TICK: u8 = 128;
    const NON_FLAG_REG_VALUES: u8 = 0b0111_1100;

    fn get_local_address(address: Address) -> Address {
        address - Self::START_ADDRESS
    }

    pub fn tick(&mut self) {
        if !self.get_flag(ControlFlag::TransferEnable) || !self.is_master() {
            return;
        }
        if !self.decrement_m_cycles_until_transfer() {
            return;
        }
        self.do_transfer();
    }

    fn do_transfer(&mut self) {
        self.transfer_out();
        self.transfer_in();

        self.bits_left_in_transfer -= 1;

        if self.bits_left_in_transfer == 0 {
            notate_event(GameBoyEvent::Interrupt(Interrupt::Serial));
        }
    }

    fn transfer_out(&mut self) {
        let outgoing_bit: Bit = Bit::from((self.data & 0x80) != 0); // get the leftmost bit
        self.output_buffer.push(outgoing_bit);
        self.data <<= 1;
    }

    fn transfer_in(&mut self) {
        let incoming_bit = self.get_incoming_bit();
        self.data |= incoming_bit.as_u8();
    }

    fn get_incoming_bit(&mut self) -> Bit {
        // todo!("If we ever actually hook up a second gameboy, this would need to read in the data")
        Bit::default()
    }

    /// Ticks what is basically out clock divider.
    /// Returns whether or not we should transfer data.
    fn decrement_m_cycles_until_transfer(&mut self) -> bool {
        self.m_cycles_until_transfer -= 1;
        if self.m_cycles_until_transfer != 0 {
            return false;
        }

        self.reset_m_cycles_until_transfer();
        true
    }

    fn reset_m_cycles_until_transfer(&mut self) {
        self.m_cycles_until_transfer = Self::M_CYCLES_IN_A_TICK;
    }

    pub fn read(&self, address: Address) -> u8 {
        self.get_register(SerialRegister::from(address))
    }

    fn get_register(&self, register: SerialRegister) -> u8 {
        match register {
            SerialRegister::Data => self.data,
            SerialRegister::Control => self.get_control_register(),
        }
    }

    pub fn write(&mut self, address: Address, value: u8) {
        self.set_register(SerialRegister::from(address), value)
    }

    fn set_register(&mut self, register: SerialRegister, value: u8) {
        match register {
            SerialRegister::Data => self.data = value,
            SerialRegister::Control => self.set_control_register(value),
        }
    }

    fn set_control_register(&mut self, value: u8) {
        self.clock_speed = ((value >> ControlFlag::ClockSpeed.get_index()) & 0b1) == 1;
        self.clock_select = ((value >> ControlFlag::ClockSelect.get_index()) & 0b1) == 1;

        self.bits_left_in_transfer = if (value >> ControlFlag::TransferEnable.get_index() & 0b1) == 1 {
            self.reset_m_cycles_until_transfer();
            8
        } else {
            0
        };
    }

    fn get_control_register(&self) -> u8 {
        let mut register = Self::NON_FLAG_REG_VALUES;

        register |= ((self.bits_left_in_transfer > 0) as u8) << ControlFlag::TransferEnable.get_index();
        register |= (self.clock_speed as u8) << ControlFlag::ClockSpeed.get_index();
        register |= (self.clock_select as u8) << ControlFlag::ClockSelect.get_index();

        register
    }

    fn get_flag(&self, flag: ControlFlag) -> bool {
        match flag {
            ControlFlag::TransferEnable => self.bits_left_in_transfer > 0,
            ControlFlag::ClockSpeed => self.clock_speed,
            ControlFlag::ClockSelect => self.clock_select,
        }
    }

    fn is_master(&self) -> bool {
        self.clock_select
    }

    pub fn get_serial_output(&self) -> Vec<&Bit> {
        self.output_buffer.as_vec()
    }
}

pub fn turn_output_to_string(mut bits: Vec<&Bit>) -> String {
    let mut message = String::new();
    for chunk in bits.chunks_exact_mut(8) {
        chunk.reverse();
        message.push(Bit::to_char(chunk.try_into().unwrap()));
    }
    message
}

impl Default for Serial {
    fn default() -> Self {
        Self {
            data: Default::default(),

            clock_speed: Default::default(),
            clock_select: Default::default(),

            m_cycles_until_transfer: Self::M_CYCLES_IN_A_TICK,
            output_buffer: Default::default(),
            bits_left_in_transfer: 0,
        }
    }
}

enum SerialRegister {
    Data,
    Control,
}

impl From<Address> for SerialRegister {
    fn from(address: Address) -> Self {
        let local_address = Serial::get_local_address(address);
        match local_address {
            0 => Self::Data,
            1 => Self::Control,
            _ => unreachable!(),
        }
    }
}

enum ControlFlag {
    TransferEnable,
    ClockSpeed,
    ClockSelect,
}
impl ControlFlag {
    fn get_index(&self) -> u8 {
        match self {
            ControlFlag::TransferEnable => 7,
            ControlFlag::ClockSpeed => 1,
            ControlFlag::ClockSelect => 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const ENABLE_TRANSFER: u8 = 0x81; // SC values to enable transfer and set as master

    fn send_out_bit(serial: &mut Serial) {
        for _ in 0..Serial::M_CYCLES_IN_A_TICK {
            serial.tick();
        }
    }

    fn send_out_byte(serial: &mut Serial, byte: u8) {
        serial.write(0xFF01, byte);
        serial.write(0xFF02, ENABLE_TRANSFER);
        for _ in 0..8 {
            send_out_bit(serial);
        }
    }

    #[test]
    fn test_serial_out() {
        let mut serial = Serial::default();

        let test_message = "This is a test!";

        for byte in test_message.as_bytes().iter() {
            send_out_byte(&mut serial, *byte);
        }

        let recomposed_message = turn_output_to_string(serial.get_serial_output());

        assert_eq!(test_message, recomposed_message)
    }
}
