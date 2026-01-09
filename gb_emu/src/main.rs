use crate::game_boy::GameBoy;

mod bus;
mod cpu;
mod game_boy;
mod helper_functions;
mod instruction_tables;
mod instructions;

fn main() {
    let mut game_boy = GameBoy::new();
    game_boy.test_looping(10000);
}
