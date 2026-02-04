use minifb::{Key, Window, WindowOptions};
use spin_sleep::SpinSleeper;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::emulator::EmulatorCommand;
use crate::graphics::ppu::{Frame, SCREEN_HEIGHT, SCREEN_SIZE, SCREEN_WIDTH};
use crate::io_devices::joypad_input::ButtonInput;

const WINDOW_NAME: &str = &"Another GameBoy Emulator";
const INPUT_POLLS_PER_SECOND: u32 = 60;
const WINDOW_FRAME_RATE: u32 = 60;

const KEY_MAPPING: [Key; 8] = [
    Key::D,     // Bit 0: Right
    Key::S,     // Bit 1: Left
    Key::W,     // Bit 2: Up
    Key::A,     // Bit 3: Down
    Key::K,     // Bit 4: A
    Key::J,     // Bit 5: B
    Key::Enter, // Bit 6: Select
    Key::Space, // Bit 7: Start
];
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
    last_sent_input: u8,
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
            last_sent_input: 0xFF,
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
        if self.frame_handle.update_frame() {
            self.window
                .update_with_buffer(
                    &self.frame_handle.get_frame()[..],
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                )
                .unwrap()
        } else {
            self.window.update();
        }
    }

    fn poll_input(&self) -> u8 {
        let mut input = 0x00;
        for i in 0..8 {
            if !self.window.is_key_down(KEY_MAPPING[i]) {
                input |= 0b1 << i
            }
        }

        input
    }

    fn send_input_to_emulator(&mut self) {
        let input_value = self.poll_input();
        if input_value != self.last_sent_input {
            self.button_input.store(input_value, Ordering::Release);
            self.last_sent_input = input_value;
        }
    }
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
    /// Updates the pixel buffer with the new frame data if there is a new one.
    /// Returns a bool of if there was a frame to update with.
    pub fn update_frame(&mut self) -> bool {
        if !self.check_for_new_frame() {
            return false;
        }

        if let Ok(pending_frame) = self.shared.pending_frame.lock() {
            pending_frame.send_to_pixel_buffer(&mut self.pixel_buffer);
        }

        true
    }
    pub fn get_frame(&self) -> &Box<[u32; SCREEN_SIZE]> {
        &self.pixel_buffer
    }
    fn check_for_new_frame(&mut self) -> bool {
        self.shared.check_for_new_frame()
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

    fn check_for_new_frame(&self) -> bool {
        self.has_new_frame.swap(false, Ordering::SeqCst)
    }
}
