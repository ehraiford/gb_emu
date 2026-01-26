use crate::{
    bus::{Address, BusAccessFailure, BusAccessible, MemoryTarget},
    graphics::{lcd::PpuMode, ppu::VideoMemory},
};

pub struct VideoRam {
    ram_banks: Vec<VideoRamBank>,
    tile_maps: TileMaps,
    cpu_accessible: bool,
}

impl VideoRam {
    const TILE_MAP_START_ADDR: u16 = 0x9800;

    pub fn new_gb() -> Self {
        Self {
            ram_banks: vec![Default::default()],
            tile_maps: Default::default(),
            cpu_accessible: true,
        }
    }
    pub fn new_cgb() -> Self {
        Self {
            ram_banks: vec![Default::default(); 2],
            tile_maps: Default::default(),
            cpu_accessible: true,
        }
    }

    fn is_cpu_accessible(&self) -> bool {
        self.cpu_accessible
    }
    fn set_cpu_accessibility(&mut self, setting: bool) {
        self.cpu_accessible = setting
    }

    /// todo!("This will need to point to the correct one when there's multiple"
    fn active_bank_num(&self) -> usize {
        0
    }

    fn get_ram_bank_mut(&mut self) -> &mut VideoRamBank {
        let bank_num = self.active_bank_num();
        &mut self.ram_banks[bank_num]
    }
    fn get_ram_bank(&self) -> &VideoRamBank {
        let bank_num = self.active_bank_num();
        &self.ram_banks[bank_num]
    }

    pub(super) fn get_8000_method(&self, tile_number: u8) -> &Tile {
        self.get_ram_bank().get_8000_method(tile_number as usize)
    }
    pub(super) fn get_8800_method(&self, tile_number: i8) -> &Tile {
        self.get_ram_bank().get_8800_method(tile_number)
    }
    pub(super) fn get_8000_method_mut(&mut self, tile_number: u8) -> &mut Tile {
        self.get_ram_bank_mut().get_8000_method_mut(tile_number as usize)
    }
    pub(super) fn get_8800_method_mut(&mut self, tile_number: i8) -> &mut Tile {
        self.get_ram_bank_mut().get_8800_method_mut(tile_number)
    }

    pub fn print_all_tiles(&self) {
        for block in self.ram_banks[self.active_bank_num()].tiles {
            for tile in block {
                if !tile.is_blank() {
                    tile.display_in_terminal();
                }
            }
        }
    }
}

impl BusAccessible for VideoRam {
    const MM_DEVICE: MemoryTarget = MemoryTarget::VideoRam;

    fn read(&mut self, address: Address) -> crate::bus::BusAccessOutcome<u8> {
        if !self.is_cpu_accessible() {
            return u8::from(BusAccessFailure::InaccessbileInPpuMode).into();
        }

        if address < Self::TILE_MAP_START_ADDR {
            let address = Self::local(address);
            let byte_index = TileByteIndex::address_to_index(address);
            let ram_bank = self.get_ram_bank_mut();
            ram_bank.get_byte(byte_index).into()
        } else {
            self.tile_maps.get_byte(address - Self::TILE_MAP_START_ADDR).into()
        }
    }

    fn write(&mut self, address: Address, value: u8) -> crate::bus::BusAccessOutcome<()> {
        if !self.is_cpu_accessible() {
            return BusAccessFailure::InaccessbileInPpuMode.into();
        }

        if address < Self::TILE_MAP_START_ADDR {
            let address = Self::local(address);

            let byte_index = TileByteIndex::address_to_index(address);
            let ram_bank = self.get_ram_bank_mut();

            ram_bank.set_byte(byte_index, value).into()
        } else {
            self.tile_maps
                .set_byte(address - Self::TILE_MAP_START_ADDR, value)
                .into()
        }
    }

    fn peek(&self, address: Address) -> u8 {
        if address < Self::TILE_MAP_START_ADDR {
            let address = Self::local(address);

            let byte_index = TileByteIndex::address_to_index(address);
            let ram_bank = self.get_ram_bank();

            ram_bank.get_byte(byte_index)
        } else {
            self.tile_maps.get_byte(address - Self::TILE_MAP_START_ADDR)
        }
    }
}

impl Default for VideoRam {
    fn default() -> Self {
        Self::new_gb()
    }
}

impl VideoMemory for VideoRam {
    fn update_ppu_mode(&mut self, mode: PpuMode) {
        match mode {
            PpuMode::HorizontalBlank | PpuMode::VerticalBlank | PpuMode::DrawingPixels => {
                self.set_cpu_accessibility(true)
            },
            PpuMode::OamScan => self.set_cpu_accessibility(false),
        }
    }
}

#[derive(Clone, Copy)]
struct VideoRamBank {
    tiles: [[Tile; 128]; 3],
}

impl VideoRamBank {
    fn get_8000_method(&self, tile_number: usize) -> &Tile {
        &self.tiles[tile_number / 128][tile_number % 128]
    }
    fn get_8800_method(&self, tile_number: i8) -> &Tile {
        let block_number = (tile_number >= 0) as usize + 1;
        let tile_index = tile_number.abs() as usize;
        &self.tiles[block_number][tile_index]
    }
    fn get_8000_method_mut(&mut self, tile_number: usize) -> &mut Tile {
        &mut self.tiles[tile_number / 128][tile_number % 128]
    }
    fn get_8800_method_mut(&mut self, tile_number: i8) -> &mut Tile {
        let block_number = (tile_number >= 0) as usize + 1;
        let tile_index = tile_number.abs() as usize;
        &mut self.tiles[block_number][tile_index]
    }

    fn set_byte(&mut self, byte_index: TileByteIndex, value: u8) {
        self.tiles[byte_index.block_number][byte_index.tile_index].set_byte(byte_index.row, byte_index.column, value);
    }
    fn get_byte(&self, byte_index: TileByteIndex) -> u8 {
        self.tiles[byte_index.block_number][byte_index.tile_index].get_byte(byte_index.row, byte_index.column)
    }
}

impl Default for VideoRamBank {
    fn default() -> Self {
        Self { tiles: [[Default::default(); 128]; 3] }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Tile {
    data: [[u8; 2]; 8],
}

impl Tile {
    fn set_byte(&mut self, row: usize, column: usize, value: u8) {
        self.data[column][row] = value
    }

    fn get_byte(&self, row: usize, column: usize) -> u8 {
        self.data[row][column]
    }

    fn get_pixels(&self) -> [[Pixel; 8]; 8] {
        let mut pixels = [[Default::default(); 8]; 8];
        for (row_index, row) in self.data.iter().enumerate() {
            let left = Pixel::from_byte(row[0]);
            let right = Pixel::from_byte(row[1]);
            pixels[row_index] = [
                left[0], left[1], left[2], left[3], right[0], right[1], right[2], right[3],
            ]
        }

        pixels
    }

    fn display_in_terminal(&self) {
        let pixels = self.get_pixels();
        for row in pixels {
            for pixel in row {
                print!("\x1b[48;5;{}m  \x1b[0m", TEST_COLORS[pixel.color_number as usize]);
            }
        }
    }

    fn is_blank(&self) -> bool {
        for line in self.data {
            for byte in line {
                if byte != 0 {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Default, Copy, Clone)]
struct Pixel {
    pub color_number: u8,
}

impl Pixel {
    fn new(color_number: u8) -> Self {
        Self { color_number }
    }

    fn from_byte(byte: u8) -> [Self; 4] {
        [
            Self::new(byte >> 6 & 0b11),
            Self::new(byte >> 4 & 0b11),
            Self::new(byte >> 2 & 0b11),
            Self::new(byte & 0b11),
        ]
    }
}

pub const TEST_COLORS: [u8; 4] = [0, 82, 28, 22];

#[derive(Clone, Copy)]
struct TileMap {
    tile_map: [u8; 32 * 32],
}

impl TileMap {
    fn get_byte(&self, index: usize) -> u8 {
        self.tile_map[index]
    }
    fn set_byte(&mut self, index: usize, value: u8) {
        self.tile_map[index] = value;
    }
}

impl Default for TileMap {
    fn default() -> Self {
        Self { tile_map: [Default::default(); 32 * 32] }
    }
}

#[derive(Default, Clone, Copy)]
struct TileMaps {
    tile_maps: [TileMap; 2],
}

impl TileMaps {
    const TILE_MAP_SIZE: usize = 0x400;

    fn get_byte(&self, address: Address) -> u8 {
        let tile_map_index = (address as usize) / Self::TILE_MAP_SIZE;
        let in_map_index = (address as usize) % Self::TILE_MAP_SIZE;

        self.tile_maps[tile_map_index].get_byte(in_map_index)
    }
    fn set_byte(&mut self, address: Address, value: u8) {
        let tile_map_index = (address as usize) / Self::TILE_MAP_SIZE;
        let in_map_index = (address as usize) % Self::TILE_MAP_SIZE;

        self.tile_maps[tile_map_index].set_byte(in_map_index, value);
    }
}

#[derive(Debug)]
struct TileByteIndex {
    block_number: usize,
    tile_index: usize,
    column: usize,
    row: usize,
}

impl TileByteIndex {
    const TILES_IN_BLOCK: usize = 128;
    const BYTES_IN_TILE: usize = 16;
    const ROWS_IN_TILE: usize = 8;
    const COLS_IN_TYLE: usize = 2;

    /// Translates a local address to the indeces required to access it
    fn address_to_index(address: Address) -> Self {
        let address = address as usize;

        let block_number = address / (Self::BYTES_IN_TILE * Self::TILES_IN_BLOCK);
        let tile_index = (address % Self::TILES_IN_BLOCK) / Self::BYTES_IN_TILE;
        let row = (address % Self::BYTES_IN_TILE) / Self::ROWS_IN_TILE;
        let column = address % Self::COLS_IN_TYLE;

        Self { block_number, tile_index, column, row }
    }
}
