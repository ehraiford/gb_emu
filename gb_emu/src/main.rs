use crate::game_boy::GameBoy;

mod cpu;
mod game_boy;
mod bus;
mod instructions;

fn main() {
    let mut game_boy = GameBoy::new();
    game_boy.test_looping(10000);
 }

