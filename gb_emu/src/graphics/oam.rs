use crate::bus::{Address, BusAccessible, MemoryTarget};

pub struct ObjectAttributeMemory {
    objects: [Sprite; Self::NUM_OAM_SPRITES],
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
}

impl BusAccessible for ObjectAttributeMemory {
    const MM_DEVICE: MemoryTarget = MemoryTarget::ObjectAttributeMemory;

    fn read(&mut self, address: Address) -> u8 {
        let (index, byte_num) = Self::convert_address_to_sprite_and_byte_numbers(address);

        self.objects[index].get_byte(byte_num).into()
    }

    fn write(&mut self, address: Address, value: u8) {
        let (index, byte_num) = Self::convert_address_to_sprite_and_byte_numbers(address);

        self.objects[index].set_byte(byte_num, value).into()
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

#[derive(Default, Clone, Copy)]
struct Sprite {
    y_position: u8,
    x_position: u8,
    tile_index: u8,
    flags: u8,
}

impl Sprite {
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

    fn get_flag(&self, flag: SpriteFlag) -> u8 {
        let (shift, mask) = flag.get_shift_and_mask();
        (self.flags >> shift) & mask
    }

    fn set_flag(&mut self, flag: SpriteFlag, mut value: u8) {
        let (shift, mut mask) = flag.get_shift_and_mask();
        value &= mask; // just to make sure we don't accidentally overwrite any other flag
        value <<= shift;

        mask <<= shift;
        mask = !mask;
        self.flags &= mask; // mask out the old value

        self.flags |= value;
    }
}

enum SpriteFlag {
    Priority,
    YFlip,
    XFlip,
    DmgPalette,
    Bank,
    CgbPalette,
}

impl SpriteFlag {
    /// Gets the right shift amount and mask needed to isolate the flag from the flag register
    fn get_shift_and_mask(&self) -> (u8, u8) {
        match self {
            SpriteFlag::Priority => (7, 0b1),
            SpriteFlag::YFlip => (6, 0b1),
            SpriteFlag::XFlip => (5, 0b1),
            SpriteFlag::DmgPalette => (4, 0b1),
            SpriteFlag::Bank => (3, 0b1),
            SpriteFlag::CgbPalette => (0, 0b111),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum PriorityMode {
    GameBoy = 0,
    GameBoyColor,
}

impl From<u8> for PriorityMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::GameBoy,
            _ => Self::GameBoyColor,
        }
    }
}
