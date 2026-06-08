#![allow(dead_code)]
#![allow(clippy::module_inception)]

#[cfg(not(feature = "headless"))]
use std::sync::{Arc, Mutex, atomic::AtomicU8};

#[cfg(not(feature = "headless"))]
use crate::os_interface::get_os_interface_variables;
use crate::{
    emulator::{Emulator, EmulatorCommand},
    helpers::disassemble_rom,
    os_interface::command_line::{CommandLineArguments, CommandLineCommand},
};
use clap::Parser;

mod bus;
pub mod cartridge;
mod emulator;
pub mod game_boy;
mod graphics;
pub mod helpers;
pub mod io_devices;
mod onboard_memory;
mod os_interface;
mod processor;

pub fn run_program() {
    let command_line_args = CommandLineArguments::parse();

    let emulator_command = match command_line_args.get_command() {
        CommandLineCommand::Disassemble { rom_path, output_path } => {
            return disassemble_rom(rom_path, output_path);
        },
        command => EmulatorCommand::from(command.clone()),
    };

    let rom_data: Vec<u8> = std::fs::read(command_line_args.get_rom_path()).expect("Must provide path to ROM");

    run_emulator(&rom_data, emulator_command, command_line_args);
}

#[cfg(feature = "headless")]
fn run_emulator(rom_data: &[u8], emulator_command: EmulatorCommand, _: CommandLineArguments) {
    let mut emulator = Emulator::new();
    emulator.load_rom(rom_data);
    emulator.run_command(emulator_command);
}

#[cfg(not(feature = "headless"))]
fn run_emulator(rom_data: &[u8], emulator_command: EmulatorCommand, command_line_args: CommandLineArguments) {
    let (debug_sender, debug_receiver) = command_line_args.get_debugging_handles();
    let (ppu_handle, window_frame_handle, button_input) = get_os_interface_variables();

    let mut emulator: Emulator = Emulator::new(ppu_handle, Arc::<AtomicU8>::clone(&button_input), debug_sender);
    emulator.load_rom(rom_data);

    let shared_command = Arc::new(Mutex::new(EmulatorCommand::Wait));
    emulator.start_emulator_thread(shared_command.clone());

    let mut os_window =
        os_interface::window::OsWindow::new(window_frame_handle, button_input, shared_command, debug_receiver);
    os_window.start_loop(emulator_command);
}
