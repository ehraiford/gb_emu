#[cfg(not(feature = "headless"))]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{
    cartridge::cartridge::Cartridge,
    game_boy::{GameBoy, MCycles},
    graphics::video_ram::TileMapImage,
    io_devices::joypad_input::ButtonInput,
    os_interface::{command_line::CommandLineCommand, debugging::DebugSender, window::SenderFrameHandle},
};
use spin_sleep::SpinSleeper;

pub struct Emulator {
    gameboy: GameBoy,
    executed_m_cycles: MCycles,
    ticked_frames: u32,
    spin_sleeper: SpinSleeper,
    start_time: Instant,
    #[cfg(not(feature = "headless"))]
    debug_sender: DebugSender,
}

impl Emulator {
    const M_CYCLES_IN_FRAME: MCycles = MCycles(17_556);
    const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706);

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        let cartridge = Cartridge::new(rom_data).unwrap();
        self.gameboy.load_cartridge(cartridge);
    }

    fn run_for_num_cycles(&mut self, cycles: u64) {
        self.start_time = Instant::now();
        while self.executed_m_cycles < MCycles(cycles) {
            self.run_frame();
        }
    }

    fn run(&mut self) {
        self.start_time = Instant::now();
        loop {
            self.run_frame();
        }
    }

    fn run_as_fast_as_possible(&mut self) {
        loop {
            self.tick();
        }
    }

    fn run_frame(&mut self) {
        let target_m_cycle_amount = self.executed_m_cycles + Self::M_CYCLES_IN_FRAME;
        while self.executed_m_cycles < target_m_cycle_amount {
            self.tick();
        }

        self.ticked_frames += 1;

        #[cfg(not(feature = "headless"))]
        self.send_debug_data_to_ui_thread();

        if let Some(variance) = self.get_clock_variance() {
            self.spin_sleeper.sleep(variance);
        }
    }

    fn tick(&mut self) {
        self.gameboy.tick();
        self.executed_m_cycles += MCycles(1);
    }

    pub fn run_command(&mut self, command: EmulatorCommand) {
        match command {
            EmulatorCommand::Run => self.run(),
            EmulatorCommand::RunForNumberOfCycles(cycles) => self.run_for_num_cycles(cycles),
            EmulatorCommand::Wait => std::thread::sleep(Duration::from_millis(10)),
            EmulatorCommand::RunAsFastAsPossible => self.run_as_fast_as_possible(),
        }
    }

    fn get_tile_map_images(&self) -> [TileMapImage; 2] {
        self.gameboy.get_tile_map_images()
    }

    /// Gets the difference between how long the emulator did take and hardware would have taken to execute to here.
    /// If we have somehow taken LONGER to execute than real hardware, we return None, instead.
    fn get_clock_variance(&self) -> Option<Duration> {
        let real_duration: Duration = Instant::now() - self.start_time;
        let expected_duration = self.get_expected_duration();

        expected_duration.checked_sub(real_duration)
    }

    fn get_expected_duration(&self) -> Duration {
        Self::FRAME_DURATION * self.ticked_frames
    }
}

#[cfg(feature = "headless")]
impl Emulator {
    pub fn new() -> Self {
        Self {
            gameboy: GameBoy::new(),
            executed_m_cycles: MCycles(0),
            start_time: Instant::now(),
            spin_sleeper: SpinSleeper::new(100_000).with_spin_strategy(spin_sleep::SpinStrategy::YieldThread),
            ticked_frames: 0,
        }
    }
}
#[cfg(not(feature = "headless"))]
impl Emulator {
    fn send_debug_data_to_ui_thread(&mut self) {
        if let Some(()) = &self.debug_sender.logging {
            todo!("This hasn't been implemented yet")
        }
        if let Some(tile_sender) = &self.debug_sender.tile_view_sender {
            let tile_map_iamges = self.get_tile_map_images();
            if let Ok(mut tile_sender) = tile_sender.try_lock() {
                *tile_sender = tile_map_iamges;
            }
        }
    }

    pub fn new(frame_handle: SenderFrameHandle, button_input: ButtonInput, debug_sender: DebugSender) -> Self {
        Self {
            gameboy: GameBoy::new(frame_handle, button_input),
            executed_m_cycles: MCycles(0),
            start_time: Instant::now(),
            spin_sleeper: SpinSleeper::new(100_000).with_spin_strategy(spin_sleep::SpinStrategy::YieldThread),
            ticked_frames: 0,
            debug_sender,
        }
    }

    pub fn start_emulator_thread(mut self, mutexed_command: Arc<Mutex<EmulatorCommand>>) {
        std::thread::spawn(move || {
            loop {
                let Ok(command) = mutexed_command.lock() else {
                    continue;
                };

                self.run_command(*command);
            }
        });
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EmulatorCommand {
    Run,
    RunForNumberOfCycles(u64),
    Wait,
    RunAsFastAsPossible,
}

impl From<CommandLineCommand> for EmulatorCommand {
    fn from(command: CommandLineCommand) -> Self {
        match command {
            CommandLineCommand::Run { .. } => Self::Run,
            CommandLineCommand::RunForNumberOfCycles { cycles, .. } => Self::RunForNumberOfCycles(cycles),
            CommandLineCommand::RunAsFastAsPossible { .. } => Self::RunAsFastAsPossible,
            CommandLineCommand::Disassemble { .. } => unreachable!(),
        }
    }
}
