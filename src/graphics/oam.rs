use crate::bus::{Address, BusAccessible, MemoryTarget};

pub struct ObjectAttributeMemory {
    objects: [ObjectAttributes; Self::NUM_OAM_SPRITES],
    priority_mode: PriorityMode,
}

impl ObjectAttributeMemory {
    const NUM_OAM_SPRITES: usize = 40;

    fn convert_address_to_sprite_and_byte_numbers(address: Address) -> (usize, usize) {
        let address = Self::local(address) as usize;

        let index = address / 4;
        let byte_num = address % 4;

        (index, byte_num)
    }
    pub fn set_priority_mode(&mut self, mode: PriorityMode) {
        self.priority_mode = mode;
    }

    pub fn set_from_dma_transfer(&mut self, address: Address, value: u8) {
        let (index, byte_num) = Self::convert_address_to_sprite_and_byte_numbers(address);
        self.objects[index].set_byte(byte_num, value)
    }

    pub fn get_object_attributes(&self, object_number: u8) -> ObjectAttributes {
        self.objects[object_number as usize]
    }

    fn get_row(&self, index: usize) -> [u16; 4] {
        let obj1: [u16; 2] = self.objects[index * 2].into();
        let obj2: [u16; 2] = self.objects[index * 2 + 1].into();

        [obj1[0], obj1[1], obj2[0], obj2[1]]
    }

    fn set_row(&mut self, index: usize, data: [u16; 4]) {
        self.objects[index * 2] = [data[0], data[1]].into();
        self.objects[index * 2 + 1] = [data[2], data[3]].into();
    }

    pub fn oam_corruption(&mut self, mut kind: CorruptionKind, row_index: usize) {
        if kind == CorruptionKind::ReadDuringIncreaseDecrease {
            // read during increase/decrease is complicated enough that we'll give it a dedicated function
            self.oam_corruption_read_during_increase_decrease(row_index);
            kind = CorruptionKind::Read; // read corruption always follows the extra logic
        }
        let Some(prev_index) = row_index.checked_sub(1) else {
            // corruption doesn't occur in the first row
            return;
        };
        let curr_row = self.get_row(row_index);
        let prev_row = self.get_row(prev_index);

        let algorithm = kind.get_first_word_algorithm();

        let mut corruption = prev_row; // last three words come from the previous row
        corruption[0] = algorithm(curr_row[0], prev_row[0], prev_row[2], 0);

        self.set_row(row_index, corruption);
    }

    fn oam_corruption_read_during_increase_decrease(&mut self, row_index: usize) {
        if row_index < 4 || row_index == 19 {
            return; // this extra corruption doesn't occur in the 1st 4 rows or last row 
        }

        let curr_row = self.get_row(row_index);
        let prev_row = self.get_row(row_index - 1);
        let two_rows_prior = self.get_row(row_index - 2);

        let algorithm = CorruptionKind::ReadDuringIncreaseDecrease.get_first_word_algorithm();

        let mut corruption = prev_row; // last three words come from the previous row
        corruption[0] = algorithm(two_rows_prior[0], prev_row[0], curr_row[0], prev_row[2]);

        self.set_row(row_index, corruption);
        self.set_row(row_index - 1, corruption);
        self.set_row(row_index - 2, corruption);
    }
}

impl BusAccessible for ObjectAttributeMemory {
    const MM_DEVICE: MemoryTarget = MemoryTarget::ObjectAttributeMemory;

    fn read(&mut self, address: Address) -> u8 {
        let (index, byte_num) = Self::convert_address_to_sprite_and_byte_numbers(address);

        self.objects[index].get_byte(byte_num)
    }

    fn write(&mut self, address: Address, value: u8) {
        let (index, byte_num) = Self::convert_address_to_sprite_and_byte_numbers(address);

        self.objects[index].set_byte(byte_num, value)
    }

    fn peek(&self, address: Address) -> u8 {
        let (index, byte_num) = Self::convert_address_to_sprite_and_byte_numbers(address);

        self.objects[index].get_byte(byte_num)
    }
}

impl Default for ObjectAttributeMemory {
    fn default() -> Self {
        Self {
            objects: [Default::default(); 40],
            priority_mode: PriorityMode::GameBoy,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct ObjectAttributes {
    y_position: u8,
    x_position: u8,
    tile_index: u8,
    flags: u8,
}

impl ObjectAttributes {
    fn get_byte(&self, byte_number: usize) -> u8 {
        match byte_number {
            0 => self.y_position,
            1 => self.x_position,
            2 => self.tile_index,
            3 => self.flags,
            _ => unreachable!("This is only called where it's been properly modulus-ed"),
        }
    }
    fn set_byte(&mut self, byte_number: usize, value: u8) {
        match byte_number {
            0 => self.y_position = value,
            1 => self.x_position = value,
            2 => self.tile_index = value,
            3 => self.flags = value,
            _ => unreachable!("This is only called where it's been properly modulus-ed"),
        }
    }

    pub fn get_tile_index(&self) -> u8 {
        self.tile_index
    }

    pub fn get_flag(&self, flag: ObjectFlag) -> u8 {
        let (shift, mask) = flag.get_shift_and_mask();
        (self.flags >> shift) & mask
    }

    pub fn is_x_flipped(&self) -> bool {
        self.get_flag(ObjectFlag::XFlip) == 1
    }

    pub fn get_palette_choice(&self) -> PaletteChoice {
        match self.get_flag(ObjectFlag::DmgPalette) {
            0 => PaletteChoice::OBP0,
            1 => PaletteChoice::OBP1,
            _ => unreachable!(),
        }
    }

    pub fn get_background_priority(&self) -> u8 {
        self.get_flag(ObjectFlag::Priority)
    }

    pub fn get_tile_index_and_byte_number(&self, ly: u8, obj_size: u8) -> (u8, u8) {
        let object_y = self.get_y_position().wrapping_sub(16);
        let mut line_in_sprite = ly.wrapping_sub(object_y);
        if self.get_flag(ObjectFlag::YFlip) == 1 {
            line_in_sprite = obj_size - 1 - line_in_sprite;
        }

        let mut tile_index = self.get_tile_index();

        if obj_size == 16 {
            tile_index &= 0xFE;

            if line_in_sprite >= 8 {
                tile_index |= 1;
                line_in_sprite -= 8;
            }
        }
        line_in_sprite &= 7;
        let byte_number = line_in_sprite * 2;
        (tile_index, byte_number)
    }

    fn set_flag(&mut self, flag: ObjectFlag, mut value: u8) {
        let (shift, mut mask) = flag.get_shift_and_mask();
        value &= mask; // just to make sure we don't accidentally overwrite any other flag
        value <<= shift;

        mask <<= shift;
        mask = !mask;
        self.flags &= mask; // mask out the old value

        self.flags |= value;
    }

    pub fn get_y_position(&self) -> u8 {
        self.y_position
    }

    pub fn is_at_x_position(&self, x_address: u8) -> bool {
        self.x_position == x_address + 8
    }
}

impl From<[u16; 2]> for ObjectAttributes {
    fn from(value: [u16; 2]) -> Self {
        Self {
            y_position: value[0] as u8,
            x_position: (value[0] >> 8) as u8,
            tile_index: value[1] as u8,
            flags: (value[1] >> 8) as u8,
        }
    }
}
impl From<ObjectAttributes> for [u16; 2] {
    fn from(value: ObjectAttributes) -> Self {
        [
            ((value.x_position as u16) << 8) | (value.y_position as u16),
            ((value.flags as u16) << 8) | (value.tile_index as u16),
        ]
    }
}

pub enum ObjectFlag {
    Priority,
    YFlip,
    XFlip,
    DmgPalette,
    Bank,
    CgbPalette,
}

impl ObjectFlag {
    /// Gets the right shift amount and mask needed to isolate the flag from the flag register
    fn get_shift_and_mask(&self) -> (u8, u8) {
        match self {
            ObjectFlag::Priority => (7, 0b1),
            ObjectFlag::YFlip => (6, 0b1),
            ObjectFlag::XFlip => (5, 0b1),
            ObjectFlag::DmgPalette => (4, 0b1),
            ObjectFlag::Bank => (3, 0b1),
            ObjectFlag::CgbPalette => (0, 0b111),
        }
    }
}

// CGB: OBJ priority mode. DMG always uses GameBoy (OAM order). GameBoyColor variant is for CGB coordinate-order mode.
#[derive(Debug)]
#[repr(u8)]
pub enum PriorityMode {
    GameBoy = 0,
    GameBoyColor, // CGB
}

impl From<u8> for PriorityMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::GameBoy,
            _ => Self::GameBoyColor,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub enum PaletteChoice {
    #[default]
    OBP0,
    OBP1,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CorruptionKind {
    Read,
    Write,
    ReadDuringIncreaseDecrease,
}

impl CorruptionKind {
    const fn get_first_word_algorithm(&self) -> fn(u16, u16, u16, u16) -> u16 {
        match self {
            CorruptionKind::Read => |a, b, c, _| b | (a & c),
            CorruptionKind::Write => |a, b, c, _| ((a ^ c) & (b ^ c)) ^ c,
            CorruptionKind::ReadDuringIncreaseDecrease => |a, b, c, d| (b & (a | c | d)) | (a & c & d),
        }
    }
}
