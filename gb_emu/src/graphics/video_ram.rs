use crate::{
    bus::{Address, BusAccessible, MemoryTarget},
    graphics::pixel_fetchers::{FifoBackgroundPixel, FifoObjectPixel},
};

pub struct VideoRam {
    ram_banks: Vec<VideoRamBank>,
    tile_maps: TileMaps,
}

impl VideoRam {
    const TILE_MAP_START_ADDR: u16 = 0x9800;

    pub fn new_gb() -> Self {
        Self {
            ram_banks: vec![Default::default()],
            tile_maps: Default::default(),
        }
    }
    pub fn _new_cgb() -> Self {
        Self {
            ram_banks: vec![Default::default(); 2],
            tile_maps: Default::default(),
        }
    }

    pub fn get_tile_index_from_map(&self, map: &TargetTileMap, row: u8, column: u8) -> u8 {
        self.tile_maps.get_tile_index(map, row, column)
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

    pub fn get_tile_byte(&self, method: AccessMethod, tile_number: u8, byte_number: u8) -> u8 {
        let tile = self.get_tile(method, tile_number);
        tile.get_byte(byte_number as usize)
    }

    pub fn get_tile(&self, method: AccessMethod, tile_number: u8) -> &Tile {
        match method {
            AccessMethod::Method8000 => self.get_8000_method(tile_number),
            AccessMethod::Method8800 => self.get_8800_method(tile_number as i8),
        }
    }

    fn get_8000_method(&self, tile_number: u8) -> &Tile {
        self.get_ram_bank().get_8000_method(tile_number as usize)
    }
    fn get_8800_method(&self, tile_number: i8) -> &Tile {
        self.get_ram_bank().get_8800_method(tile_number)
    }
}

impl BusAccessible for VideoRam {
    const MM_DEVICE: MemoryTarget = MemoryTarget::VideoRam;

    fn read(&mut self, address: Address) -> u8 {
        if address < Self::TILE_MAP_START_ADDR {
            let address = Self::local(address);
            let byte_index = TileByteIndex::address_to_index(address);
            let ram_bank = self.get_ram_bank_mut();
            ram_bank.get_byte(byte_index).into()
        } else {
            self.tile_maps.get_byte(address - Self::TILE_MAP_START_ADDR).into()
        }
    }

    fn write(&mut self, address: Address, value: u8) {
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

#[derive(Clone, Copy)]
struct VideoRamBank {
    tiles: [[Tile; 128]; 3],
}

impl VideoRamBank {
    fn get_8000_method(&self, tile_number: usize) -> &Tile {
        &self.tiles[tile_number / 128][tile_number % 128]
    }
    fn get_8800_method(&self, tile_number: i8) -> &Tile {
        let adjusted_index = (tile_number as i16 + 128) as usize;
        &self.tiles[1 + (adjusted_index / 128)][adjusted_index % 128]
    }

    fn set_byte(&mut self, byte_index: TileByteIndex, value: u8) {
        self.tiles[byte_index.block_number][byte_index.tile_index].set_byte(byte_index.byte_index, value);
    }
    fn get_byte(&self, byte_index: TileByteIndex) -> u8 {
        self.tiles[byte_index.block_number][byte_index.tile_index].get_byte(byte_index.byte_index)
    }
}

impl Default for VideoRamBank {
    fn default() -> Self {
        Self { tiles: [[Default::default(); 128]; 3] }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub enum AccessMethod {
    #[default]
    Method8000,
    Method8800,
}

#[derive(Default, Copy, Clone)]
pub struct Tile {
    data: [u8; 16],
}

impl Tile {
    fn set_byte(&mut self, byte_index: usize, value: u8) {
        self.data[byte_index] = value;
    }

    fn get_byte(&self, byte_index: usize) -> u8 {
        if byte_index == 32 {
            panic!();
        }
        self.data[byte_index]
    }
}

#[derive(Copy, Clone)]
pub struct RawPixel {
    pub color_number: u8,
}

impl RawPixel {
    pub fn new(color_number: u8) -> Self {
        Self { color_number }
    }

    pub fn from_bytes(low_byte: u8, high_byte: u8) -> [Self; 8] {
        std::array::from_fn(|i| {
            let bit_index = 7 - i;
            let low_bit = (low_byte >> bit_index) & 1;
            let high_bit = (high_byte >> bit_index) & 1;
            let color = (high_bit << 1) | low_bit;
            RawPixel::new(color)
        })
    }
}

impl Default for RawPixel {
    fn default() -> Self {
        Self { color_number: Default::default() }
    }
}
impl From<RawPixel> for u32 {
    fn from(value: RawPixel) -> Self {
        let shade = match value.color_number {
            0 => 0xFF,
            1 => 0xAA,
            2 => 0x55,
            3 => 0x00,
            _ => unreachable!(),
        };
        // minifb wants 0x00RRGGBB
        ((shade as u32) << 16) | ((shade as u32) << 8) | (shade as u32)
    }
}

pub struct ColoredPixel {
    pub color: u8,
}

impl ColoredPixel {
    pub fn from_background_pixel(raw_pixel: FifoBackgroundPixel, background_palette: u8) -> Self {
        Self::from_raw_color_and_palette(raw_pixel.color_number, background_palette)
    }
    pub fn from_object_pixel(raw_pixel: FifoObjectPixel, palette: u8) -> Self {
        Self::from_raw_color_and_palette(raw_pixel.color_number, palette)
    }

    fn from_raw_color_and_palette(color_number: u8, palette: u8) -> Self {
        Self { color: palette >> (color_number * 2) & 0b11 }
    }
}

#[derive(Clone, Copy)]
struct TileMap {
    tile_map: [[u8; 32]; 32],
}

impl TileMap {
    fn get_byte(&self, index: usize) -> u8 {
        self.tile_map[index / 32][index % 32]
    }
    fn set_byte(&mut self, index: usize, value: u8) {
        self.tile_map[index / 32][index % 32] = value;
    }
    fn get_byte_from_coords(&self, row: u8, column: u8) -> u8 {
        self.tile_map[row as usize][column as usize]
    }
}

impl Default for TileMap {
    fn default() -> Self {
        Self { tile_map: [[Default::default(); 32]; 32] }
    }
}

#[derive(Default, Clone, Copy)]
struct TileMaps {
    tile_map_bank: [TileMap; 2],
}

impl TileMaps {
    const TILE_MAP_SIZE: usize = 0x400;

    fn get_byte(&self, address: Address) -> u8 {
        let tile_map_index = (address as usize) / Self::TILE_MAP_SIZE;
        let in_map_index = (address as usize) % Self::TILE_MAP_SIZE;

        self.tile_map_bank[tile_map_index].get_byte(in_map_index)
    }
    fn set_byte(&mut self, address: Address, value: u8) {
        let tile_map_index = (address as usize) / Self::TILE_MAP_SIZE;
        let in_map_index = (address as usize) % Self::TILE_MAP_SIZE;

        self.tile_map_bank[tile_map_index].set_byte(in_map_index, value);
    }

    pub fn get_tile_index(&self, map: &TargetTileMap, row: u8, column: u8) -> u8 {
        self.tile_map_bank[usize::from(*map)].get_byte_from_coords(row, column)
    }
}

#[derive(Default, Clone, Copy)]
pub enum TargetTileMap {
    #[default]
    At0x9800,
    At0x9C00,
}
impl From<TargetTileMap> for usize {
    fn from(map: TargetTileMap) -> usize {
        match map {
            TargetTileMap::At0x9800 => 0,
            TargetTileMap::At0x9C00 => 1,
        }
    }
}
#[derive(Debug)]
struct TileByteIndex {
    block_number: usize,
    tile_index: usize,
    byte_index: usize,
}

impl TileByteIndex {
    const BLOCK_SIZE: usize = 128;
    const BYTES_IN_TILE: usize = 16;

    /// Translates a local address to the indeces required to access it
    fn address_to_index(address: Address) -> Self {
        let address = address as usize;

        let block_number = address / (Self::BYTES_IN_TILE * Self::BLOCK_SIZE);
        let in_block_address = address % (Self::BYTES_IN_TILE * Self::BLOCK_SIZE);
        let tile_index = in_block_address / Self::BYTES_IN_TILE;
        let byte_index = in_block_address % Self::BYTES_IN_TILE;

        Self { block_number, tile_index, byte_index }
    }
}
