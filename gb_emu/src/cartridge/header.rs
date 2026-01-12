use crate::helper_functions::concat_2_bytes;

pub struct Header {
    logo: [u8; 0x30],
    cgb_flag: ColorGameBoyFlag,
    cartridge_type: CartridgeType,
    num_rom_banks: u8,
    num_ram_banks: u8,
    header_checksum: u8,
    global_checksum: u16,
}

impl Header {
    pub fn new(data: &[u8]) -> Self {
        Self {
            logo: data[4..34].try_into().unwrap(),
            cgb_flag: ColorGameBoyFlag::new(data[0x43]),
            cartridge_type: CartridgeType::from(data[0x47]),
            num_rom_banks: data[0x48] << 2,
            num_ram_banks: get_num_ram_banks(data[0x49]),
            header_checksum: data[0x4D],
            global_checksum: concat_2_bytes(data[0x4E], data[0x4F]),
        }
    }
}

struct ColorGameBoyFlag {
    flag_value: u8,
}

impl ColorGameBoyFlag {
    fn new(val: u8) -> Self {
        Self { flag_value: val }
    }
}

struct CartridgeType {
    elements: Vec<CartridgeElement>,
}

impl CartridgeType {
    fn new(value: u8) -> Self {
        Self::from(value)
    }
}

impl From<u8> for CartridgeType {
    fn from(value: u8) -> Self {
        use CartridgeElement::*;
        let elements = match value {
            0x00 => vec![],
            0x01 => vec![MemoryBankController(1)],
            0x02 => vec![MemoryBankController(1), Ram],
            0x03 => vec![MemoryBankController(1), Ram, Battery],
            0x05 => vec![MemoryBankController(2)],
            0x06 => vec![MemoryBankController(2), Battery],
            0x08 => vec![Ram],
            0x09 => vec![Ram, Battery],
            0x0B => vec![MMM01],
            0x0C => vec![MMM01, Ram],
            0x0D => vec![MMM01, Ram, Battery],
            0x0F => vec![MemoryBankController(3), Timer, Battery],
            0x10 => vec![MemoryBankController(3), Timer, Ram, Battery],
            0x11 => vec![MemoryBankController(3)],
            0x12 => vec![MemoryBankController(3), Ram],
            0x13 => vec![MemoryBankController(3), Ram, Battery],
            0x19 => vec![MemoryBankController(5)],
            0x1A => vec![MemoryBankController(5), Ram],
            0x1B => vec![MemoryBankController(5), Ram, Battery],
            0x1C => vec![MemoryBankController(5), Rumble],
            0x1D => vec![MemoryBankController(5), Rumble, Ram],
            0x1E => vec![MemoryBankController(5), Rumble, Ram, Battery],
            0x20 => vec![MemoryBankController(6)],
            0x22 => vec![MemoryBankController(7), Sensor, Rumble, Ram, Battery],
            0xFC => vec![PocketCamera],
            0xFD => vec![BandaiTama5],
            0xFE => vec![HuC3],
            0xFF => vec![HuC1, Ram, Battery],
            _ => unreachable!("Every possible type should already be accounted for above"),
        };

        Self { elements }
    }
}

enum CartridgeElement {
    MemoryBankController(u8),
    Battery,
    Ram,
    MMM01,
    Timer,
    Rumble,
    Sensor,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1,
}

fn get_num_ram_banks(byte_code: u8) -> u8 {
    match byte_code {
        0x00 => 0,
        0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,
        _ => unreachable!("Every possible type should already be accounted for above"),
    }
}
