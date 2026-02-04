use crate::{
    cartridge::cartridge::Cartridge,
    game_boy::GameBoy,
    helpers::disassemble_rom,
    os_interface::command_line::{Command, CommandLineArguments},
};
use clap::Parser;

mod bus;
mod cartridge;
mod game_boy;
mod graphics;
mod helpers;
mod io_devices;
mod onboard_memory;
mod os_interface;
mod processor;

fn main() {
    let command_line_args = CommandLineArguments::parse();

    match command_line_args.get_command() {
        Command::Disassemble { output_path } => disassemble_rom(command_line_args.get_rom_path(), output_path),
        Command::Run => {
            let rom_data = std::fs::read(command_line_args.get_rom_path()).unwrap();

            let mut game_boy = GameBoy::new();
            let cartridge = Cartridge::new(&rom_data).unwrap();
            game_boy.load_cartridge(cartridge);

            todo!()
        },
        Command::RunForNumberOfCycles { cycles } => {
            let rom_data = std::fs::read(command_line_args.get_rom_path()).unwrap();

            let mut game_boy = GameBoy::new();
            let cartridge = Cartridge::new(&rom_data).unwrap();
            game_boy.load_cartridge(cartridge);
            game_boy.test_looping(*cycles);
        },
    }
}
