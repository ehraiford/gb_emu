use crate::{
    io_devices::joypad_input::{ButtonInput, new_button_input},
    os_interface::window::{ReceiverFrameHandle, SenderFrameHandle, TripleBuffer},
};

pub mod command_line;
pub mod controller;
pub mod debugging;
pub mod input;
pub mod profiling;
pub mod save_files;
pub mod window;

pub fn get_os_interface_variables() -> (SenderFrameHandle, ReceiverFrameHandle, ButtonInput) {
    let (sender, receiver) = TripleBuffer::create();

    (sender, receiver, new_button_input())
}
pub struct WindowThreadHandles {}

pub struct EmulatorThreadHandles {}
