use std::sync::{Arc, atomic::AtomicU8};

use crate::{
    game_boy::{GameBoyEvent, notate_event},
    io_devices::interrupts::Interrupt,
};

pub struct JoyPadInput {
    selected_input: SelectedInput,
    // we're packing all button values into a single byte so the input thread can easily send all the data
    #[cfg(not(feature = "headless"))]
    button_input: ButtonInput,
    currently_pressed: u8,
}

impl JoyPadInput {
    const DEFAULT_INPUT_VALUE: u8 = 0xFF;

    pub fn new(button_input: ButtonInput) -> Self {
        Self {
            button_input,
            selected_input: Default::default(),
            currently_pressed: Default::default(),
        }
    }

    pub fn write(&mut self, value: u8) {
        self.selected_input = SelectedInput::from(value);
    }

    pub fn read(&self) -> u8 {
        u8::from(self.selected_input) | self.currently_pressed
    }

    fn get_input_nibble(&self) -> u8 {
        let byte = self.currently_pressed;
        match self.selected_input {
            SelectedInput::Both => (byte | (byte >> 4)) & 0x0F,
            SelectedInput::Buttons => (byte & 0xF0) >> 4,
            SelectedInput::DPad => byte & 0x0F,
            SelectedInput::Neither => 0x0F,
        }
    }

    fn new_button_pressed(prev_input: u8, new_input: u8) -> bool {
        new_input & !prev_input != 0
    }
}

#[cfg(feature = "headless")]
impl JoyPadInput {
    pub fn tick(&mut self) {}
}
#[cfg(not(feature = "headless"))]
impl JoyPadInput {
    fn ingest_input(&mut self) {
        self.currently_pressed = self.button_input.load(std::sync::atomic::Ordering::Relaxed);
    }
    pub fn tick(&mut self) {
        let prev_input = self.get_input_nibble();
        self.ingest_input();
        let new_input = self.get_input_nibble();
        if Self::new_button_pressed(prev_input, new_input) {
            notate_event(GameBoyEvent::Interrupt(Interrupt::Joypad));
        }
    }
}

#[derive(Copy, Clone, Default)]
enum SelectedInput {
    #[default]
    Both,
    Buttons,
    DPad,
    Neither,
}

impl From<SelectedInput> for u8 {
    fn from(value: SelectedInput) -> Self {
        match value {
            SelectedInput::Neither => 0x30,
            SelectedInput::DPad => 0x20,
            SelectedInput::Buttons => 0x10,
            SelectedInput::Both => 0x00,
        }
    }
}
impl From<u8> for SelectedInput {
    fn from(value: u8) -> Self {
        match value & 0b00110000 {
            0x00 => Self::Both,
            0x10 => Self::Buttons,
            0x20 => Self::DPad,
            0x30 => Self::Neither,
            _ => unreachable!(),
        }
    }
}

pub type ButtonInput = Arc<AtomicU8>;
pub fn new_button_input() -> ButtonInput {
    Arc::new(AtomicU8::new(JoyPadInput::DEFAULT_INPUT_VALUE))
}
