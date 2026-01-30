use minifb::{Window, WindowOptions};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::graphics::ppu::{SCREEN_HEIGHT, SCREEN_SIZE, SCREEN_WIDTH};

const WINDOW_NAME: &str = &"Another GameBoy Emulator";

pub fn start_window_thread() -> Sender<u32> {
    let (tx, rx): (Sender<u32>, Receiver<u32>) = mpsc::channel();

    thread::spawn(move || {
        let mut window = Window::new(
            WINDOW_NAME,
            SCREEN_WIDTH as usize,
            SCREEN_HEIGHT as usize,
            WindowOptions { scale: minifb::Scale::X4, ..Default::default() },
        )
        .unwrap();
        let mut frame_buffer = FrameBuffer::default();

        while window.is_open() {
            for new_pixel in rx.try_iter() {
                frame_buffer.push_pixel(new_pixel);
            }
            window
                .update_with_buffer(&frame_buffer.buffer[..], SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
                .unwrap();
        }
    });

    tx
}

struct FrameBuffer {
    buffer: [u32; SCREEN_SIZE],
    update_index: usize,
}
impl FrameBuffer {
    fn push_pixel(&mut self, pixel: u32) {
        self.buffer[self.update_index] = pixel;
        self.update_index += 1;
        self.update_index %= SCREEN_SIZE;
    }
}
impl Default for FrameBuffer {
    fn default() -> Self {
        Self { buffer: [0; SCREEN_SIZE], update_index: 0 }
    }
}
