use sdl2::{
    EventPump, Sdl,
    event::Event,
    keyboard::Scancode,
    pixels::PixelFormatEnum,
    render::{Texture, WindowCanvas},
};
use spin_sleep::SpinSleeper;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::emulator::EmulatorCommand;
use crate::graphics::ppu::{Frame, SCREEN_HEIGHT, SCREEN_SIZE, SCREEN_WIDTH};
use crate::io_devices::joypad_input::ButtonInput;
use crate::os_interface::debugging::DebugReceiver;
use crate::os_interface::input::InputAggregator;

const WINDOW_NAME: &str = "Another GameBoy Emulator";
const INPUT_POLLS_PER_SECOND: u32 = 60;
const WINDOW_FRAME_RATE: u32 = 60;
const WINDOW_SCALE: u32 = 4;
const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32 * WINDOW_SCALE;
const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32 * WINDOW_SCALE;

pub fn get_main_canvas(sdl: &Sdl) -> WindowCanvas {
    let video = sdl.video().unwrap();

    let window = video
        .window(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position(100, 100)
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas
        .set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .unwrap();
    canvas.set_integer_scale(true).unwrap();

    canvas
}

// pub fn get_tile_map_window() -> Window {
//     let mut window = Window::new(
//         "Tile Viewer",
//         TileViewer::WINDOW_WIDTH,
//         TileViewer::WINDOW_HEIGHT,
//         WindowOptions { scale: minifb::Scale::X2, ..Default::default() },
//     )
//     .unwrap();

//     window.set_position(1000, 100);

//     window
// }

pub struct OsWindow {
    main_canvas: WindowCanvas,
    event_pump: EventPump,
    input_aggregator: InputAggregator,
    frame_handle: ReceiverFrameHandle,
    shared_command: Arc<Mutex<EmulatorCommand>>,
    spin_sleeper: SpinSleeper,
    debug_receiver: DebugReceiver,
    _sdl: Sdl,
}

impl OsWindow {
    pub fn new(
        frame_handle: ReceiverFrameHandle,
        button_input: ButtonInput,
        shared_command: Arc<Mutex<EmulatorCommand>>,
        debug_receiver: DebugReceiver,
    ) -> Self {
        let sdl = sdl2::init().unwrap();

        Self {
            main_canvas: get_main_canvas(&sdl),
            event_pump: sdl.event_pump().unwrap(),
            frame_handle,
            shared_command,
            spin_sleeper: SpinSleeper::new(100_000).with_spin_strategy(spin_sleep::SpinStrategy::YieldThread),
            debug_receiver,
            input_aggregator: InputAggregator::new(button_input, &sdl),
            _sdl: sdl,
        }
    }

    pub fn start_loop(&mut self, start_command: EmulatorCommand) {
        let texture_creator = self.main_canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB888, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .unwrap();

        let inner_duration = Duration::from_secs(1) / INPUT_POLLS_PER_SECOND;
        let outer_duration = Duration::from_secs(1) / WINDOW_FRAME_RATE;

        *self.shared_command.lock().unwrap() = start_command;

        'main: loop {
            let loop_start = Instant::now();

            loop {
                let elapsed_frame_time = Instant::now() - loop_start;
                if elapsed_frame_time >= outer_duration {
                    break;
                }

                if self.handle_events() {
                    break 'main;
                }
                self.send_input_to_emulator();

                let time_left_in_frame = outer_duration - elapsed_frame_time;
                if time_left_in_frame < inner_duration {
                    self.spin_sleeper.sleep(time_left_in_frame);
                } else {
                    self.spin_sleeper.sleep(inner_duration);
                }
            }

            self.update_display(&mut texture);
            self.debug_receiver.update()
        }
    }

    fn handle_events(&mut self) -> bool {
        let mut should_quit = false;

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { scancode: Some(Scancode::Escape), .. } => should_quit = true,
                _ => {},
            }
        }

        should_quit
    }

    fn update_display(&mut self, texture: &mut Texture) {
        if self.frame_handle.update_frame() {
            let frame = self.frame_handle.get_frame();
            let bytes = unsafe { std::slice::from_raw_parts(frame.as_ptr().cast::<u8>(), SCREEN_SIZE * 4) };

            texture.update(None, bytes, SCREEN_WIDTH as usize * 4).unwrap();
        }

        self.main_canvas.clear();
        self.main_canvas.copy(texture, None, None).unwrap();
        self.main_canvas.present();
    }

    fn send_input_to_emulator(&mut self) {
        self.input_aggregator.poll_and_send(&self.event_pump);
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
    pub fn get_frame(&self) -> &[u32; SCREEN_SIZE] {
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
    pub fn create() -> (SenderFrameHandle, ReceiverFrameHandle) {
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
