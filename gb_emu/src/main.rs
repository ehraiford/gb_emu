use std::{
    env,
    fs::{self},
    path::PathBuf,
};

use crate::{cartridge::cartridge::Cartridge, game_boy::GameBoy, helpers::disassemble};

mod bus;
mod cartridge;
mod dma;
mod game_boy;
mod graphics;
mod helpers;
mod io_devices;
mod onboard_memory;
mod os_interface;
mod processor;

const TEST_ROM: &str = &r"C:\Users\evanr\OneDrive\Desktop\Games\Dr. Mario (World) (Rev 1).gb";
// const TEST_CYCLES: u64 = 0x849fc0;
const TEST_CYCLES: u64 = 1_000_000_00;

fn main() {
    #[cfg(feature = "disassemble")]
    _program_assembly();

    #[cfg(not(feature = "disassemble"))]
    {
        let mut game_boy = GameBoy::new();
        let cartridge = Cartridge::new(&read_test_data()).unwrap();
        game_boy.load_cartridge(cartridge);

        game_boy.test_looping(TEST_CYCLES);
    }
}

pub fn read_test_data() -> Vec<u8> {
    fs::read(TEST_ROM).unwrap()
}

fn _program_assembly() {
    let args: Vec<String> = env::args().collect();
    let rom_path = PathBuf::from(&args[1]);

    let mut output_path = PathBuf::from("..");
    output_path.push("disassembled_output");
    output_path.push(rom_path.file_name().unwrap());

    let assembly = disassemble(&fs::read(rom_path).unwrap());

    println!("{assembly}")
}
