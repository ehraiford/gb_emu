use std::sync::{Arc, atomic::AtomicU8};

pub struct JoyPadInput {
    selected_input: SelectedInput,
    buttons_pressed: Arc<AtomicU8>,
}

impl JoyPadInput {
    pub fn write(&mut self, value: u8) {
        self.selected_input = SelectedInput::from(value);
    }

    pub fn read(&self) -> u8 {
        u8::from(self.selected_input) | self.get_input()
    }
    fn get_input(&self) -> u8 {
        let value = self.buttons_pressed.load(std::sync::atomic::Ordering::Relaxed);
        match self.selected_input {
            SelectedInput::Both => todo!(),
            SelectedInput::Buttons => todo!(),
            SelectedInput::DPad => todo!(),
            SelectedInput::Neither => todo!(),
        }
    }
    pub fn tick(&mut self) {}
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
