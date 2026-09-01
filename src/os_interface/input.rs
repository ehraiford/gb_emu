use std::sync::atomic::Ordering;

use sdl2::{
    EventPump, Sdl,
    keyboard::{Keycode, Scancode},
};

use crate::{io_devices::joypad_input::ButtonInput, os_interface::controller::ControllerInput};

pub const NO_BUTTONS_PRESSED: u8 = 0xFF;

pub trait InputSource {
    fn poll(&mut self, event_pump: &EventPump) -> u8;
}

pub struct InputAggregator {
    sources: Vec<Box<dyn InputSource>>,
    button_input: ButtonInput,
    last_sent: u8,
}
impl InputAggregator {
    pub fn new(button_input: ButtonInput, sdl: &Sdl) -> Self {
        let mut this = Self {
            sources: Vec::new(),
            button_input,
            last_sent: NO_BUTTONS_PRESSED,
        };

        this.add_source(Box::new(KeyboardInput));
        if let Some(controller) = ControllerInput::new(sdl) {
            this.add_source(Box::new(controller));
        }
        
        this
    }

    pub fn poll_and_send(&mut self, event_pump: &EventPump) -> u8 {
        let combined_input = self
            .sources
            .iter_mut()
            .fold(NO_BUTTONS_PRESSED, |acc, source| acc & source.poll(event_pump));

        if combined_input != self.last_sent {
            self.button_input.store(combined_input, Ordering::Release);
            self.last_sent = combined_input;
        }
        combined_input
    }

    pub fn add_source(&mut self, source: Box<dyn InputSource>) {
        self.sources.push(source);
    }
}

const SDL_KEY_MAPPING: [Scancode; 8] = [
    Scancode::D,      // Bit 0: Right
    Scancode::A,      // Bit 1: Left
    Scancode::W,      // Bit 2: Up
    Scancode::S,      // Bit 3: Down
    Scancode::K,      // Bit 4: A
    Scancode::J,      // Bit 5: B
    Scancode::Return, // Bit 6: Select
    Scancode::Space,  // Bit 7: Start
];

pub struct KeyboardInput;
impl InputSource for KeyboardInput {
    fn poll(&mut self, event_pump: &EventPump) -> u8 {
        let keyboard_state = event_pump.keyboard_state();

        let mut buttons = NO_BUTTONS_PRESSED;
        for (bit, scancode) in SDL_KEY_MAPPING.iter().enumerate() {
            if keyboard_state.is_scancode_pressed(*scancode) {
                buttons &= !(1 << bit);
            }
        }

        buttons
    }
}
