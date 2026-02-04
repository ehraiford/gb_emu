#[cfg(not(feature = "headless"))]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{
    cartridge::cartridge::Cartridge,
    game_boy::{GameBoy, MCycles},
    io_devices::joypad_input::ButtonInput,
    os_interface::{command_line::CommandLineCommand, window::SenderFrameHandle},
};
use spin_sleep::SpinSleeper;

pub struct Emulator {
    gameboy: GameBoy,
    executed_m_cycles: MCycles,
    ticked_frames: u32,
    spin_sleeper: SpinSleeper,
    start_time: Instant,
}

impl Emulator {
    const M_CYCLES_IN_FRAME: MCycles = MCycles(17_556);
    const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706);

    pub fn new(frame_handle: SenderFrameHandle, button_input: ButtonInput) -> Self {
        Self {
            gameboy: GameBoy::new(frame_handle, button_input),
            executed_m_cycles: MCycles(0),
            start_time: Instant::now(),
            spin_sleeper: SpinSleeper::new(100_000).with_spin_strategy(spin_sleep::SpinStrategy::YieldThread),
            ticked_frames: 0,
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        let cartridge = Cartridge::new(&rom_data).unwrap();
        self.gameboy.load_cartridge(cartridge);
    }
}

impl Emulator {
    fn run_for_num_cycles(&mut self, cycles: u64) {
        while self.executed_m_cycles < MCycles(cycles) {
            self.tick();
        }
    }

    fn run(&mut self) {
        loop {
            self.run_frame();
        }
    }

    fn run_frame(&mut self) {
        let target_m_cycle_amount = self.executed_m_cycles + Self::M_CYCLES_IN_FRAME;
        while self.executed_m_cycles < target_m_cycle_amount {
            self.tick();
        }

        self.ticked_frames += 1;

        if let Some(variance) = self.get_clock_variance() {
            self.spin_sleeper.sleep(variance);
        }
    }

    fn tick(&mut self) {
        self.executed_m_cycles += self.gameboy.tick();
    }

    pub fn run_command(&mut self, command: EmulatorCommand) {
        match command {
            EmulatorCommand::Run => self.run(),
            EmulatorCommand::RunForNumberOfCycles(cycles) => self.run_for_num_cycles(cycles),
            EmulatorCommand::Wait => (),
        }
    }

    /// Gets the difference between how long the emulator did take and hardware would have taken to execute to here.
    /// If we have somehow taken LONGER to execute than real hardware, we return None, instead.
    fn get_clock_variance(&self) -> Option<Duration> {
        let real_duration = Instant::now() - self.start_time;
        let expected_duration = self.get_expected_duration();

        expected_duration.checked_sub(real_duration)
    }

    fn get_expected_duration(&self) -> Duration {
        Self::FRAME_DURATION * self.ticked_frames
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EmulatorCommand {
    Run,
    RunForNumberOfCycles(u64),
    Wait,
}

impl From<&CommandLineCommand> for EmulatorCommand {
    fn from(command: &CommandLineCommand) -> Self {
        match command {
            CommandLineCommand::Run => Self::Run,
            CommandLineCommand::RunForNumberOfCycles { cycles } => Self::RunForNumberOfCycles(*cycles),
            CommandLineCommand::Disassemble { output_path: _ } => unreachable!(),
        }
    }
}

#[cfg(not(feature = "headless"))]
pub fn emulator_thread(mut emulator: Emulator, mutexed_command: Arc<Mutex<EmulatorCommand>>) {
    std::thread::spawn(move || {
        loop {
            if let Ok(command) = mutexed_command.lock() {
                emulator.run_command(*command);
            }
        }
    });
}
