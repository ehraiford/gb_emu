use crate::{
    cartridge::cartridge::{ControlledMemory, MBCType},
    helper_functions::concat_2_bytes,
};

pub struct Header {
    logo: [u8; 0x30],
    cgb_flag: ColorGameBoyFlag,
    cartridge_type: CartridgeType,
    num_rom_banks: usize,
    num_ram_banks: usize,
    header_checksum: u8,
    global_checksum: u16,
}

impl Header {
    pub fn new(data: &[u8]) -> Self {
        Self {
            logo: data[0x4..0x34].try_into().unwrap(),
            cgb_flag: ColorGameBoyFlag::new(data[0x43]),
            cartridge_type: CartridgeType::new(data[0x47]),
            num_rom_banks: get_num_rom_banks(data[0x48]),
            num_ram_banks: get_num_ram_banks(data[0x49]),
            header_checksum: data[0x4D],
            global_checksum: concat_2_bytes(data[0x4E], data[0x4F]),
        }
    }

    pub fn get_expected_bank_size_in_kb(&self) -> usize {
        self.num_ram_banks * 8 + self.num_rom_banks * 16
    }

    pub fn get_num_ram_banks(&self) -> usize {
        self.num_ram_banks
    }
    pub fn get_num_rom_banks(&self) -> usize {
        self.num_rom_banks
    }

    pub fn get_header_defined_structures(&self) -> ControlledMemory {
        let mut mbc_type = None;
        let num_rom_banks = self.num_rom_banks;
        let mut num_ram_banks = 0;
        for element in self.cartridge_type.get_elements() {
            match element {
                CartridgeElement::MBC(specified_type) => mbc_type = Some(*specified_type),
                CartridgeElement::Ram => num_ram_banks = self.num_ram_banks,
                CartridgeElement::Battery => todo!(),
                CartridgeElement::MMM01 => todo!(),
                CartridgeElement::Timer => todo!(),
                CartridgeElement::Rumble => todo!(),
                CartridgeElement::Sensor => todo!(),
                CartridgeElement::PocketCamera => todo!(),
                CartridgeElement::BandaiTama5 => todo!(),
                CartridgeElement::HuC3 => todo!(),
                CartridgeElement::HuC1 => todo!(),
            }
        }

        ControlledMemory::new(mbc_type, num_rom_banks, num_ram_banks)
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

pub struct CartridgeType {
    elements: Vec<CartridgeElement>,
}

impl CartridgeType {
    fn new(value: u8) -> Self {
        Self::from(value)
    }

    pub fn get_elements(&self) -> &Vec<CartridgeElement> {
        &self.elements
    }
}

impl From<u8> for CartridgeType {
    fn from(value: u8) -> Self {
        use CartridgeElement::*;
        let elements = match value {
            0x00 => vec![],
            0x01 => vec![MBC(MBCType::MBC1)],
            0x02 => vec![MBC(MBCType::MBC1), Ram],
            0x03 => vec![MBC(MBCType::MBC1), Ram, Battery],
            0x05 => vec![MBC(MBCType::MBC2)],
            0x06 => vec![MBC(MBCType::MBC2), Battery],
            0x08 => vec![Ram],
            0x09 => vec![Ram, Battery],
            0x0B => vec![MMM01],
            0x0C => vec![MMM01, Ram],
            0x0D => vec![MMM01, Ram, Battery],
            0x0F => vec![MBC(MBCType::MBC3), Timer, Battery],
            0x10 => vec![MBC(MBCType::MBC3), Timer, Ram, Battery],
            0x11 => vec![MBC(MBCType::MBC3)],
            0x12 => vec![MBC(MBCType::MBC3), Ram],
            0x13 => vec![MBC(MBCType::MBC3), Ram, Battery],
            0x19 => vec![MBC(MBCType::MBC5)],
            0x1A => vec![MBC(MBCType::MBC5), Ram],
            0x1B => vec![MBC(MBCType::MBC5), Ram, Battery],
            0x1C => vec![MBC(MBCType::MBC5), Rumble],
            0x1D => vec![MBC(MBCType::MBC5), Rumble, Ram],
            0x1E => vec![MBC(MBCType::MBC5), Rumble, Ram, Battery],
            0x20 => vec![MBC(MBCType::MBC6)],
            0x22 => vec![MBC(MBCType::MBC7), Sensor, Rumble, Ram, Battery],
            0xFC => vec![PocketCamera],
            0xFD => vec![BandaiTama5],
            0xFE => vec![HuC3],
            0xFF => vec![HuC1, Ram, Battery],
            _ => unreachable!("Every possible type should already be accounted for above"),
        };

        Self { elements }
    }
}

#[derive(Debug)]
pub enum CartridgeElement {
    MBC(MBCType),
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

fn get_num_ram_banks(byte_code: u8) -> usize {
    match byte_code {
        0x00 => 0,
        0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,
        _ => unreachable!("Every possible type should already be accounted for above"),
    }
}

fn get_num_rom_banks(byte_code: u8) -> usize {
    2 << byte_code
}
