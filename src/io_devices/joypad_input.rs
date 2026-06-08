use std::sync::{Arc, atomic::AtomicU8};

use crate::game_boy::EventQueue;
#[cfg(not(feature = "headless"))]
use crate::{game_boy::GameBoyEvent, io_devices::interrupts::Interrupt};

pub struct JoyPadInput {
    selected_input: SelectedInput,
    // we're packing all button values into a single byte so the input thread can easily send all the data
    #[cfg(not(feature = "headless"))]
    button_input: ButtonInput,
    currently_pressed: u8,
}

impl JoyPadInput {
    const DEFAULT_INPUT_VALUE: u8 = 0xFF;

    pub fn write(&mut self, value: u8) {
        self.selected_input = SelectedInput::from(value);
    }

    pub fn read(&self) -> u8 {
        u8::from(self.selected_input) | self.get_input_nibble()
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

    fn new_button_pressed(prev_nibble: u8, new_nibble: u8) -> bool {
        prev_nibble & !new_nibble != 0
    }
}

#[cfg(feature = "headless")]
impl Default for JoyPadInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "headless")]
impl JoyPadInput {
    pub fn new() -> Self {
        Self { selected_input: Default::default(), currently_pressed: 0xFF }
    }
    pub fn tick(&mut self, _events: &mut EventQueue) {}
}
#[cfg(not(feature = "headless"))]
impl JoyPadInput {
    pub fn new(button_input: ButtonInput) -> Self {
        Self {
            button_input,
            selected_input: Default::default(),
            currently_pressed: 0xFF,
        }
    }

    /// sets the currently_pressed field to the value taken from the atomic u8 and returns whether it changed.
    /// Note: This is NOT enough to say a new button was pressed because it has both possible lower nibbles packed into it.
    fn ingest_input(&mut self) -> bool {
        let old_value = self.currently_pressed;

        self.currently_pressed = self.button_input.load(std::sync::atomic::Ordering::Acquire);
        self.currently_pressed != old_value
    }
    pub fn tick(&mut self, events: &mut EventQueue) {
        let prev_nibble = self.get_input_nibble();
        if self.ingest_input() {
            let new_nibble = self.get_input_nibble();
            if Self::new_button_pressed(prev_nibble, new_nibble) {
                events.push(GameBoyEvent::Interrupt(Interrupt::Joypad));
            }
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
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
            SelectedInput::Neither => 0xF0,
            SelectedInput::DPad => 0xE0,
            SelectedInput::Buttons => 0xD0,
            SelectedInput::Both => 0xC0,
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

