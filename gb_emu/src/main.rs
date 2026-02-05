#[cfg(not(feature = "headless"))]
use crate::{
    emulator::{Emulator, EmulatorCommand},
    helpers::disassemble_rom,
    os_interface::{
        command_line::{CommandLineArguments, CommandLineCommand},
        get_os_interface_variables,
    },
};
use clap::Parser;

mod bus;
mod cartridge;
mod emulator;
mod game_boy;
mod graphics;
mod helpers;
mod io_devices;
mod onboard_memory;
mod os_interface;
mod processor;

fn main() {
    let command_line_args = CommandLineArguments::parse();

    let emulator_command = match command_line_args.get_command() {
        CommandLineCommand::Disassemble { rom_path, output_path } => {
            return disassemble_rom(rom_path, output_path);
        },
        command => EmulatorCommand::from(command.clone()),
    };

    let (debug_sender, debug_receiver) = command_line_args
        .get_debugging_handles()
        .expect("This is only None if it was disassembly which was already handled");

    let rom_data: Vec<u8> = std::fs::read(command_line_args.get_rom_path()).unwrap();
    let (ppu_handle, window_frame_handle, button_input) = get_os_interface_variables();

    let mut emulator: Emulator = Emulator::new(ppu_handle, button_input.clone(), debug_sender);
    emulator.load_rom(&rom_data);

    #[cfg(feature = "headless")]
    emulator.run_command(emulator_command);

    #[cfg(not(feature = "headless"))]
    {
        use std::sync::{Arc, Mutex};

        let shared_command = Arc::new(Mutex::new(EmulatorCommand::Wait));
        crate::emulator::emulator_thread(emulator, shared_command.clone());

        let mut os_window =
            os_interface::window::OsWindow::new(window_frame_handle, button_input, shared_command, debug_receiver);
        os_window.start_loop(emulator_command);
    }
}
