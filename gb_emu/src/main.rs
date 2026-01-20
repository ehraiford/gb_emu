use std::fs;

use crate::{cartridge::cartridge::Cartridge, game_boy::GameBoy};

mod bus;
mod cartridge;
mod cpu;
mod game_boy;
mod graphics;
mod helper_functions;
mod instruction_tables;
mod instructions;
mod rom_and_ram;
mod work_ram;

const TEST_ROM: &str = &r"C:\Users\evanr\code\gb_emu\test_roms\cpu_instrs.gb";

fn main() {
    // let args: Vec<String> = env::args().collect();

    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&read_test_data()).unwrap();
    game_boy.load_cartridge(cartridge);
    game_boy.test_looping(100000);
}

fn read_test_data() -> Vec<u8> {
    fs::read(TEST_ROM).unwrap()
}
