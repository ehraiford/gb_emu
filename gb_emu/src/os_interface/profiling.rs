use std::time::{Duration, Instant};

use crate::game_boy::{EXPECTED_CLOCK_SPEED, GameBoy, TCycles};

pub struct TrackedData {
    start_time: Instant,
}

impl TrackedData {
    pub fn new() -> Self {
        Self { start_time: Instant::now() }
    }

    pub fn log_from_gameboy(&self, game_boy: &GameBoy) {
        let duration = self.start_time.elapsed();
        let cycles = game_boy.get_elapsed_cycles();
        let our_frequency = convert_to_megahertz(cycles, duration);
        println!(
            "It took us {:.2} seconds to run through {} T cycles.",
            duration.as_secs_f32(),
            cycles.0
        );
        println!(
            "That's {} MHz. Hardware is {EXPECTED_CLOCK_SPEED:.2} MHz.",
            our_frequency
        );
        println!("We're running {:.2}x hardware", our_frequency / EXPECTED_CLOCK_SPEED);
    }
}

fn convert_to_megahertz(cycles: TCycles, duration: Duration) -> f64 {
    cycles.0 as f64 / duration.as_secs_f64() / 1_000_000_f64
}
