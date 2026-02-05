use std::sync::{Arc, Mutex};

use minifb::Window;

use crate::{
    graphics::video_ram::{ColoredPixel, TileMapImage},
    os_interface::window::get_tile_map_window,
};

pub struct DebugSender {
    pub logging: Option<()>,
    pub tile_view_sender: Option<Arc<Mutex<[TileMapImage; 2]>>>,
}

pub struct DebugReceiver {
    pub logging: Option<()>,
    pub tile_view_receiver: Option<TileViewer>,
}

impl DebugReceiver {
    pub fn update(&mut self) {
        if let Some(tile_receiver) = &mut self.tile_view_receiver {
            tile_receiver.update();
        }
    }
}

pub struct TileViewer {
    window: Window,
    tile_maps: Arc<Mutex<[TileMapImage; 2]>>,
    last_image_buffer: Box<[u32; Self::WINDOW_HEIGHT * Self::WINDOW_WIDTH]>,
}

impl TileViewer {
    pub const WINDOW_WIDTH: usize = 8 * 32;
    pub const WINDOW_HEIGHT: usize = (8 * 32 * 2) + Self::EMPTY_LINES_BETWEEN_MAPS;
    pub const EMPTY_LINES_BETWEEN_MAPS: usize = 10;

    pub fn new() -> (Self, Arc<Mutex<[TileMapImage; 2]>>) {
        let this = Self {
            window: get_tile_map_window(),
            tile_maps: Arc::new(Mutex::new([TileMapImage::default(), TileMapImage::default()])),
            last_image_buffer: Box::new(
                [ColoredPixel::screen_off().to_minifb_u32(); Self::WINDOW_HEIGHT * Self::WINDOW_WIDTH],
            ),
        };

        let sender = this.tile_maps.clone();

        (this, sender)
    }

    /// todo!("This updates every frame even if there's no change to the map.
    /// We should check if there's a new one before doing all that work")
    pub fn update(&mut self) {
        if self.try_update_image_buffer() {
            self.window
                .update_with_buffer(&self.last_image_buffer[..], Self::WINDOW_WIDTH, Self::WINDOW_HEIGHT)
                .unwrap()
        } else {
            self.window.update();
        }
    }

    // Updates the last image buffer we're holding in the struct.
    // Returns whether or not we actually got a new image to display
    fn try_update_image_buffer(&mut self) -> bool {
        let (image_0, image_1) = if let Ok(buffers) = self.tile_maps.try_lock() {
            let image_0 = buffers[0].clone();
            let image_1 = buffers[1].clone();
            (image_0, image_1)
        } else {
            return false;
        };

        self.update_image_buffer(&image_0, &image_1);
        true
    }

    fn update_image_buffer(&mut self, image_0: &TileMapImage, image_1: &TileMapImage) {
        let map_len = TileMapImage::PIXELS_IN_TILEMAP;
        let middle_len = Self::EMPTY_LINES_BETWEEN_MAPS * Self::WINDOW_WIDTH;

        // 1. Write Top Map
        image_0.write_to_buffer(&mut self.last_image_buffer[..map_len]);

        // 2. Clear Middle Section
        self.last_image_buffer[map_len..map_len + middle_len].fill(0);

        // 3. Write Bottom Map
        image_1.write_to_buffer(&mut self.last_image_buffer[map_len + middle_len..]);
    }
}
