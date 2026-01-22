use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::{cartridge::cartridge::Cartridge, game_boy::GameBoy, helper_functions::disassemble};

mod bus;
mod cartridge;
mod game_boy;
mod graphics;
mod helper_functions;
mod interrupts;
mod onboard_devices;
mod os_interface;
mod processor;

const TEST_ROM: &str = &r"C:\Users\evanr\OneDrive\Desktop\Games\Dr. Mario (World) (Rev 1).gb";
const TEST_CYCLES: usize = 100_000_000;

fn main() {
    #[cfg(feature = "disassemble")]
    program_assembly();

    #[cfg(not(feature = "disassemble"))]
    {
        let mut game_boy = GameBoy::new();
        let cartridge = Cartridge::new(&read_test_data()).unwrap();
        game_boy.load_cartridge(cartridge);

        game_boy.test_looping(TEST_CYCLES);

    }
}

fn read_test_data() -> Vec<u8> {
    fs::read(TEST_ROM).unwrap()
}

fn program_assembly() {
    let args: Vec<String> = env::args().collect();
    let rom_path = PathBuf::from(&args[1]);

    let mut output_path = PathBuf::from("..");
    output_path.push("disassembled_output");
    output_path.push(rom_path.file_name().unwrap());

    let assembly = disassemble(&fs::read(rom_path).unwrap());

    let mut file = File::create(&output_path).unwrap();
    file.write_all(assembly.as_bytes()).unwrap();
}
