use crate::bus::{Address, BusAccessible, MMDevice};

#[derive(Default)]
pub struct VideoRam {
    ram_banks: Vec<VideoRamBank>,
}

impl VideoRam {
    pub fn new_gb() -> Self {
        Default::default()
    }
    pub fn new_cgb() -> Self {
        Self { ram_banks: vec![Default::default(); 2] }
    }

    fn get_ram_bank_mut(&mut self) -> &mut VideoRamBank {
        &mut self.ram_banks[0] //todo("This will need to point to the correct one when there's multiple"
    }
    fn get_ram_bank(&self) -> &VideoRamBank {
        &self.ram_banks[0] //todo("This will need to point to the correct one when there's multiple"
    }

    fn get_8000_method(&self, tile_number: u8) -> &Tile {
        self.get_ram_bank().get_8000_method(tile_number as usize)
    }
    fn get_8800_method(&self, tile_number: i8) -> &Tile {
        self.get_ram_bank().get_8800_method(tile_number)
    }
    fn get_8000_method_mut(&mut self, tile_number: u8) -> &mut Tile {
        self.get_ram_bank_mut().get_8000_method_mut(tile_number as usize)
    }
    fn get_8800_method_mut(&mut self, tile_number: i8) -> &mut Tile {
        self.get_ram_bank_mut().get_8800_method_mut(tile_number)
    }
}

impl BusAccessible for VideoRam {
    const MM_DEVICE: MMDevice = MMDevice::VideoRam;

    fn read(&mut self, mut address: Address) -> crate::bus::MemoryAccessResult<u8> {
        address = Self::local(address);
        todo!()
    }

    fn write(&mut self, mut address: Address, value: u8) -> crate::bus::MemoryAccessResult<()> {
        address = Self::local(address);

        todo!()
    }

    fn peek(&self, mut address: Address) -> crate::bus::MemoryAccessResult<u8> {
        address = Self::local(address);

        todo!()
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
}

impl Default for VideoRamBank {
    fn default() -> Self {
        Self { tiles: [[Default::default(); 128]; 3] }
    }
}

#[derive(Default, Copy, Clone)]
struct Tile {
    data: [[u8; 2]; 8],
}

impl Tile {
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
