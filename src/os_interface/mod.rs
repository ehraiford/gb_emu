use crate::{
    io_devices::joypad_input::{ButtonInput, new_button_input},
    os_interface::window::{ReceiverFrameHandle, SenderFrameHandle, TripleBuffer},
};

pub mod command_line;
pub mod debugging;
pub mod profiling;
pub mod window;

pub fn get_os_interface_variables() -> (SenderFrameHandle, ReceiverFrameHandle, ButtonInput) {
    let (sender, receiver) = TripleBuffer::new();

    (sender, receiver, new_button_input())
}
pub struct WindowThreadHandles {}

pub struct EmulatorThreadHandles {}
