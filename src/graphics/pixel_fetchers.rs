use crate::{
    game_boy::TCycles,
    graphics::{
        lcd::Lcd,
        oam::{ObjectAttributes, PaletteChoice},
        ppu::{Dots, ObjectsOnThisLine},
        video_ram::{AccessMethod, ColoredPixel, RawPixel, VideoRam},
    },
    helpers::StackAllocQueue,
};

#[derive(Default)]
pub struct PixelFetchers {
    object_fetcher: ObjectFifo,
    background_fetcher: BackGroundFifo,
    mode: FetchersMode,
    pixels_displayed: u8,
}

impl PixelFetchers {
    pub fn reset_for_new_scanline(&mut self) {
        self.object_fetcher.reset_for_new_scanline();
        self.background_fetcher.reset_for_new_scanline();
        self.pixels_displayed = 0;
    }

    pub fn take_scanned_objects(&mut self, objects_on_this_line: ObjectsOnThisLine) {
        self.object_fetcher.take_scanned_objects(objects_on_this_line);
    }

    pub fn reset_window_y(&mut self) {
        self.background_fetcher.reset_window_y();
    }

    pub fn tick(&mut self, lcd: &Lcd, v_ram: &VideoRam) -> Option<ColoredPixel> {
        let pixel = match self.mode {
            FetchersMode::NormalExecution => self.tick_normal_execution(lcd, v_ram),
            FetchersMode::HandlingObjectsDisabled { remaining_penalty } => {
                self.tick_handling_objects_disabled(remaining_penalty, lcd)
            },
            FetchersMode::FetchingObject => {
                self.tick_ppu_fetching_object(lcd, v_ram);
                None
            },
        };

        if pixel.is_some() {
            self.pixels_displayed += 1;
        }

        pixel
    }

    fn tick_normal_execution(&mut self, lcd: &Lcd, v_ram: &VideoRam) -> Option<ColoredPixel> {
        if let Some(object_attributes) = self.object_fetcher.check_coordinates(self.pixels_displayed, lcd) {
            self.object_fetcher.mode = ObjectFifoMode::GetDataLow { object_attributes, sleep_cycle: true };
            self.mode = FetchersMode::FetchingObject;
            self.tick_ppu_fetching_object(lcd, v_ram);
            None
        } else {
            self.background_fetcher.tick_fetching(lcd, v_ram);
            if self.background_fetcher.queue.length() == 0 {
                return None;
            }

            if let Some(object_pixel) = self.object_fetcher.try_get_pixel() {
                let background_pixel = self.background_fetcher.pop_pixel();
                Some(Self::arbitrate_pixels(background_pixel, object_pixel, lcd))
            } else {
                self.try_pop_background_pixel(lcd)
            }
        }
    }

    fn tick_ppu_fetching_object(&mut self, lcd: &Lcd, v_ram: &VideoRam) {
        self.object_fetcher.tick_fetching(lcd, v_ram);

        if self.object_fetcher.mode == ObjectFifoMode::Inactive {
            self.mode = FetchersMode::NormalExecution;
        }
    }

    fn try_pop_background_pixel(&mut self, lcd: &Lcd) -> Option<ColoredPixel> {
        if let Some(background_pixel) = self.background_fetcher.try_pop_pixel() {
            let background_palette = lcd.get_bgp();
            let colored_pixel = ColoredPixel::from_background_pixel(background_pixel, background_palette);
            Some(colored_pixel)
        } else {
            None
        }
    }

    fn arbitrate_pixels(
        background_pixel: FifoBackgroundPixel,
        object_pixel: FifoObjectPixel,
        lcd: &Lcd,
    ) -> ColoredPixel {
        if object_pixel.is_transparent()
            || !lcd.object_enabled()
            || object_pixel.background_priority == 1 && background_pixel.color_number != 0
        {
            // Background wins
            ColoredPixel::from_background_pixel(background_pixel, lcd.get_bgp())
        } else {
            // Object Wins
            ColoredPixel::from_object_pixel(object_pixel, lcd.get_obp(object_pixel.get_palette_choice()))
        }
    }

    fn tick_handling_objects_disabled(&mut self, mut remaining_penalty: u8, lcd: &Lcd) -> Option<ColoredPixel> {
        remaining_penalty -= 1;
        if remaining_penalty == 0 {
            self.mode = FetchersMode::NormalExecution;
            self.try_pop_background_pixel(lcd)
        } else {
            self.mode = FetchersMode::HandlingObjectsDisabled { remaining_penalty };
            None
        }
    }

    pub fn handle_objects_disabled(&mut self, instruction_length: TCycles) {
        if self.mode == FetchersMode::FetchingObject {
            let remaining_penalty =
                1 + instruction_length.0 as u8 + self.object_fetcher.mode.get_remaining_dots_in_loop() as u8;
            self.mode = FetchersMode::HandlingObjectsDisabled { remaining_penalty }
        }
    }
}

#[derive(Default, PartialEq, Eq)]
enum FetchersMode {
    #[default]
    NormalExecution,
    HandlingObjectsDisabled {
        remaining_penalty: u8,
    },
    FetchingObject,
}

#[derive(Default, Clone, Copy)]
pub struct FifoObjectPixel {
    pub color_number: u8,
    palette: PaletteChoice,
    background_priority: u8,
}

impl FifoObjectPixel {
    fn from_raw_pixel_and_attributes(raw_pixel: RawPixel, attributes: ObjectAttributes) -> Self {
        Self {
            color_number: raw_pixel.color_number,
            palette: attributes.get_palette_choice(),
            background_priority: attributes.get_background_priority(),
        }
    }
    fn is_transparent(&self) -> bool {
        self.color_number == 0
    }
    pub fn get_palette_choice(&self) -> PaletteChoice {
        self.palette
    }
}

impl<'a> FifoObjectPixel {
    fn determine_dominant_pixel(pixel_a: &'a Self, pixel_b: &'a Self) -> &'a Self {
        if pixel_a.is_transparent() && !pixel_b.is_transparent() {
            pixel_b
        } else {
            pixel_a
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct FifoBackgroundPixel {
    pub color_number: u8,
}

impl From<RawPixel> for FifoBackgroundPixel {
    fn from(value: RawPixel) -> Self {
        Self { color_number: value.color_number }
    }
}

#[derive(Default)]
struct ObjectFifo {
    queue: StackAllocQueue<FifoObjectPixel, 16>,
    objects_on_this_line: ObjectsOnThisLine,
    mode: ObjectFifoMode,
    tiles_considered_for_penalty: Vec<()>,
}

impl ObjectFifo {
    fn reset_for_new_scanline(&mut self) {
        self.queue.clear_queue();
        self.mode = ObjectFifoMode::Inactive;
        self.objects_on_this_line.reset_for_new_scanline();
    }

    pub fn take_scanned_objects(&mut self, objects_on_this_line: ObjectsOnThisLine) {
        self.objects_on_this_line = objects_on_this_line
    }

    fn determine_object_penalty(&self, _leftmost_pixel_x_coord: u8) -> Dots {
        todo!()
    }

    fn check_coordinates(&mut self, fetcher_x: u8, lcd: &Lcd) -> Option<ObjectAttributes> {
        if !lcd.object_enabled() {
            return None;
        }
        self.objects_on_this_line.take_object_at_x(fetcher_x)
    }

    fn try_get_pixel(&mut self) -> Option<FifoObjectPixel> {
        self.queue.try_pop()
    }

    fn tick_fetching(&mut self, lcd: &Lcd, v_ram: &VideoRam) {
        if self.mode.should_sleep() {
            self.mode.sleep();
            return;
        }

        match self.mode {
            ObjectFifoMode::GetDataLow { object_attributes, .. } => {
                let (tile_index, byte_number) =
                    object_attributes.get_tile_index_and_byte_number(lcd.get_ly(), lcd.get_object_size());
                let low_byte = v_ram.get_tile_byte(AccessMethod::Method8000, tile_index, byte_number);
                self.mode = ObjectFifoMode::GetDataHigh {
                    low_byte,
                    object_attributes,
                    sleep_cycle: true,
                    tile_index,
                    byte_number: byte_number + 1,
                };
            },
            ObjectFifoMode::GetDataHigh {
                low_byte,
                object_attributes,
                tile_index,
                byte_number,
                sleep_cycle: _,
            } => {
                let high_byte = v_ram.get_tile_byte(AccessMethod::Method8000, tile_index, byte_number);
                let pixels = RawPixel::from_bytes(low_byte, high_byte);
                self.push_pixels(pixels, object_attributes);
                self.mode = ObjectFifoMode::Inactive;
            },
            ObjectFifoMode::Inactive => unreachable!("Tick shouldn't even be called in Inactive mode"),
        }
    }

    fn push_pixels(&mut self, mut pixels: [RawPixel; 8], attributes: ObjectAttributes) {
        if attributes.is_x_flipped() {
            pixels.reverse();
        }
        for (i, pixel) in pixels.iter().enumerate() {
            let new_pixel = FifoObjectPixel::from_raw_pixel_and_attributes(*pixel, attributes);

            if let Some(existing_pixel) = self.queue.try_get_entry_mut(i) {
                *existing_pixel = *FifoObjectPixel::determine_dominant_pixel(existing_pixel, &new_pixel);
            } else {
                self.queue.push(new_pixel);
            }
        }
    }
}

#[derive(Default, PartialEq)]
enum ObjectFifoMode {
    #[default]
    Inactive,
    GetDataLow {
        object_attributes: ObjectAttributes,
        sleep_cycle: bool,
    },
    GetDataHigh {
        low_byte: u8,
        tile_index: u8,
        byte_number: u8,
        object_attributes: ObjectAttributes,
        sleep_cycle: bool,
    },
}

impl ObjectFifoMode {
    fn get_remaining_dots_in_loop(&self) -> Dots {
        todo!()
    }
    fn should_sleep(&self) -> bool {
        match self {
            ObjectFifoMode::Inactive => false,
            ObjectFifoMode::GetDataLow { sleep_cycle, .. } | ObjectFifoMode::GetDataHigh { sleep_cycle, .. } => {
                *sleep_cycle
            },
        }
    }
    fn sleep(&mut self) {
        match self {
            ObjectFifoMode::Inactive => unreachable!("Sleep shouldn't even be called in Inactive mode"),
            ObjectFifoMode::GetDataLow { sleep_cycle, .. } | ObjectFifoMode::GetDataHigh { sleep_cycle, .. } => {
                *sleep_cycle = false
            },
        }
    }
}

#[derive(Default)]
struct BackGroundFifo {
    queue: StackAllocQueue<FifoBackgroundPixel, 16>,
    mode: BackGroundFifoMode,
    pixels_popped: u8,
    tiles_fetched: u8,

    window_y: u8,
    window_displayed_this_line: bool,
}

impl BackGroundFifo {
    fn current_tile_is_window_tile(&self, lcd: &Lcd, fetcher_x: u8) -> bool {
        lcd.window_enabled() && lcd.coordinate_in_window(fetcher_x)
    }

    fn get_tile_location(&mut self, lcd: &Lcd) -> (u8, u8, u8) {
        // Use the fetcher's progress (tiles_fetched * 8) rather than the pixels already on screen.
        // This ensures the fetcher switches to the Window tilemap at the correct dot.
        let fetcher_x = self.tiles_fetched * 8;

        if self.current_tile_is_window_tile(lcd, fetcher_x) {
            let window_x = fetcher_x.saturating_sub(lcd.get_wx().saturating_sub(7));
            self.window_displayed_this_line = true;
            (window_x / 8, self.window_y / 8, self.window_y % 8)
        } else {
            let scx = lcd.get_scx();
            let calced_y = lcd.get_ly().wrapping_add(lcd.get_scy());
            let fetcher_bg_x = scx.wrapping_add(fetcher_x);

            (fetcher_bg_x / 8, calced_y >> 3, calced_y & 7)
        }
    }

    fn tick_fetching(&mut self, lcd: &Lcd, v_ram: &VideoRam) {
        // first check if we should act this cycle.
        // This is used to force the modes to take two cycles
        if self.mode.should_sleep() {
            self.mode.sleep();
            return;
        }

        match self.mode {
            BackGroundFifoMode::Tile { .. } => {
                let access_method = lcd.get_background_window_tiles_address_mode();

                let fetcher_x = self.tiles_fetched * 8;
                let map = lcd.get_target_tilemap(fetcher_x);

                let (column, row, in_sprite_row) = self.get_tile_location(lcd);
                let tile_number = v_ram.get_tile_index_from_map(&map, row, column);

                self.tiles_fetched = self.tiles_fetched.wrapping_add(1);

                self.mode = BackGroundFifoMode::TileDataLow {
                    sleep_cycle: true,
                    access_method,
                    tile_number,
                    byte_number: in_sprite_row << 1,
                }
            },
            BackGroundFifoMode::TileDataLow { sleep_cycle: _, access_method, tile_number, byte_number } => {
                let low_byte = v_ram.get_tile_byte(access_method, tile_number, byte_number);
                self.mode = BackGroundFifoMode::TileDataHigh {
                    sleep_cycle: true,
                    access_method,
                    tile_number,
                    byte_number: byte_number + 1,
                    low_byte,
                }
            },
            BackGroundFifoMode::TileDataHigh {
                sleep_cycle: _,
                access_method,
                tile_number,
                low_byte,
                byte_number,
            } => {
                let high_byte = v_ram.get_tile_byte(access_method, tile_number, byte_number);
                self.try_push_pixels(low_byte, high_byte);
            },
        }
    }

    fn try_push_pixels(&mut self, low_byte: u8, high_byte: u8) {
        if self.queue.length() > 8 {
            return; // we stall if there isn't enough space in the queue
        }
        for pixel in RawPixel::from_bytes(low_byte, high_byte) {
            self.queue.push(pixel.into());
        }
        self.mode = BackGroundFifoMode::Tile { sleep_cycle: true };
    }

    fn try_pop_pixel(&mut self) -> Option<FifoBackgroundPixel> {
        let maybe_pixel = self.queue.try_pop();
        if maybe_pixel.is_some() {
            self.pixels_popped += 1;
        }
        maybe_pixel
    }

    fn pop_pixel(&mut self) -> FifoBackgroundPixel {
        let pixel = self.queue.pop_unchecked();
        self.pixels_popped += 1;
        pixel
    }

    fn reset_for_new_scanline(&mut self) {
        if self.window_displayed_this_line {
            self.window_y += 1;
            self.window_displayed_this_line = false;
        }
        self.queue.clear_queue();
        self.mode = BackGroundFifoMode::Tile { sleep_cycle: true };
        self.pixels_popped = 0;
        self.tiles_fetched = 0;
    }

    pub fn reset_window_y(&mut self) {
        self.window_y = 0;
    }
}

#[derive(Debug)]
enum BackGroundFifoMode {
    Tile {
        sleep_cycle: bool,
    },
    TileDataLow {
        sleep_cycle: bool,
        access_method: AccessMethod,
        tile_number: u8,
        byte_number: u8,
    },
    TileDataHigh {
        sleep_cycle: bool,
        access_method: AccessMethod,
        tile_number: u8,
        byte_number: u8,
        low_byte: u8,
    },
}

impl Default for BackGroundFifoMode {
    fn default() -> Self {
        Self::Tile { sleep_cycle: true }
    }
}

impl BackGroundFifoMode {
    fn should_sleep(&self) -> bool {
        match self {
            BackGroundFifoMode::Tile { sleep_cycle }
            | BackGroundFifoMode::TileDataLow { sleep_cycle, .. }
            | BackGroundFifoMode::TileDataHigh { sleep_cycle, .. } => *sleep_cycle,
        }
    }
    fn sleep(&mut self) {
        match self {
            BackGroundFifoMode::Tile { sleep_cycle }
            | BackGroundFifoMode::TileDataLow { sleep_cycle, .. }
            | BackGroundFifoMode::TileDataHigh { sleep_cycle, .. } => *sleep_cycle = false,
        }
    }
}
