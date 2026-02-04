use core::time;
use minifb::{Window, WindowOptions};
use spin_sleep::SpinSleeper;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::emulator::EmulatorCommand;
use crate::graphics::ppu::{Frame, SCREEN_HEIGHT, SCREEN_SIZE, SCREEN_WIDTH};
use crate::io_devices::joypad_input::ButtonInput;

const WINDOW_NAME: &str = &"Another GameBoy Emulator";
const INPUT_POLLS_PER_SECOND: u32 = 120;
const WINDOW_FRAME_RATE: u32 = 60;

pub fn get_window() -> Window {
    Window::new(
        WINDOW_NAME,
        SCREEN_WIDTH as usize,
        SCREEN_HEIGHT as usize,
        WindowOptions { scale: minifb::Scale::X4, ..Default::default() },
    )
    .unwrap()
}

pub struct OsWindow {
    window: Window,
    button_input: ButtonInput,
    frame_handle: ReceiverFrameHandle,
    shared_command: Arc<Mutex<EmulatorCommand>>,
    spin_sleeper: SpinSleeper,
}

impl OsWindow {
    pub fn new(
        frame_handle: ReceiverFrameHandle,
        button_input: ButtonInput,
        shared_command: Arc<Mutex<EmulatorCommand>>,
    ) -> Self {
        Self {
            window: get_window(),
            button_input,
            frame_handle,
            shared_command,
            spin_sleeper: SpinSleeper::new(100_000).with_spin_strategy(spin_sleep::SpinStrategy::YieldThread),
        }
    }

    pub fn window_loop(&mut self, start_command: EmulatorCommand) {
        let inner_duration = Duration::from_secs(1) / INPUT_POLLS_PER_SECOND;
        let outer_duration = Duration::from_secs(1) / WINDOW_FRAME_RATE;

        *self.shared_command.lock().unwrap() = start_command;

        while self.window.is_open() {
            let loop_start = Instant::now();

            loop {
                let current_time = Instant::now();
                let elapsed_frame_time = current_time - loop_start;

                if elapsed_frame_time >= outer_duration {
                    break;
                }

                self.send_input_to_emulator();

                let time_left_in_frame = outer_duration - elapsed_frame_time;

                if time_left_in_frame < inner_duration {
                    self.spin_sleeper.sleep(time_left_in_frame);
                } else {
                    self.spin_sleeper.sleep(inner_duration);
                }
            }

            self.update_display();
        }
    }

    fn update_display(&mut self) {
        if self.frame_handle.has_new_frame() {
            self.frame_handle.update();
            self.window
                .update_with_buffer(
                    &self.frame_handle.get_frame()[..],
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                )
                .unwrap()
        }
    }

    fn send_input_to_emulator(&mut self) {}
}

pub struct SenderFrameHandle {
    pub buffer: Frame,
    pub shared: Arc<TripleBuffer>,
}

impl SenderFrameHandle {}

pub struct ReceiverFrameHandle {
    pixel_buffer: Box<[u32; SCREEN_SIZE]>,
    pub shared: Arc<TripleBuffer>,
}

impl ReceiverFrameHandle {
    pub fn update(&mut self) {
        if let Ok(pending_frame) = self.shared.pending_frame.lock() {
            self.shared.has_new_frame.store(false, Ordering::Release);
            pending_frame.send_to_pixel_buffer(&mut self.pixel_buffer);
        }
    }
    pub fn get_frame(&self) -> &Box<[u32; SCREEN_SIZE]> {
        &self.pixel_buffer
    }
    pub fn has_new_frame(&self) -> bool {
        self.shared.has_new_frame()
    }
}

pub struct TripleBuffer {
    pub pending_frame: Mutex<Frame>,
    pub has_new_frame: AtomicBool,
}

impl TripleBuffer {
    pub fn new() -> (SenderFrameHandle, ReceiverFrameHandle) {
        let shared_buffer = Arc::new(TripleBuffer {
            pending_frame: Mutex::new(Frame::default()),
            has_new_frame: AtomicBool::new(false),
        });

        let sender = SenderFrameHandle { buffer: Frame::default(), shared: Arc::clone(&shared_buffer) };
        let receiver = ReceiverFrameHandle {
            pixel_buffer: Box::new([0; SCREEN_SIZE]),
            shared: Arc::clone(&shared_buffer),
        };

        (sender, receiver)
    }

    fn has_new_frame(&self) -> bool {
        self.has_new_frame.load(Ordering::Acquire)
    }
}
