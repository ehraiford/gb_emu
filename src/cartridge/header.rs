use crate::{
    cartridge::{
        cartridge::CartridgeError,
        memory_bank_controllers::{
            MemoryBankController, MemoryBankController1, MemoryBankController2, MemoryBankController3,
        },
    },
    helpers::concat_2_bytes,
};

pub struct Header {
    logo: [u8; 0x30],
    cgb_flag: ColorGameBoyFlag,
    cartridge_elements: CartridgeElements,
    num_rom_banks: usize,
    num_ram_banks: usize,
    header_checksum: u8,
    global_checksum: u16,
}

impl Header {
    pub fn new(data: &[u8]) -> Result<Self, CartridgeError> {
        let num_rom_banks = get_num_rom_banks(data[0x48])?;
        let num_ram_banks = get_num_ram_banks(data[0x49])?;
        Ok(Self {
            logo: data[0x4..0x34].try_into().unwrap(),
            cgb_flag: ColorGameBoyFlag::new(data[0x43]),
            cartridge_elements: CartridgeElements::new(data[0x47], num_rom_banks, num_ram_banks)?,
            num_rom_banks,
            num_ram_banks,
            header_checksum: data[0x4D],
            global_checksum: concat_2_bytes(data[0x4E], data[0x4F]),
        })
    }

    pub fn get_expected_bank_size_in_kb(&self) -> usize {
        self.num_rom_banks * 16
    }

    pub fn get_num_ram_banks(&self) -> usize {
        self.num_ram_banks
    }
    pub fn get_num_rom_banks(&self) -> usize {
        self.num_rom_banks
    }
    pub fn get_memory_bank_controller(&self) -> Option<MemoryBankController> {
        self.cartridge_elements.mbc.clone()
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

#[derive(Debug)]
/// elemenets are options in case we support them in the future and want to instantiate a struct here
pub struct CartridgeElements {
    mbc: Option<MemoryBankController>,
    battery: Option<()>,
    ram: Option<()>,
    mmm01: Option<()>,
    rumble: Option<()>,
    sensor: Option<()>,
    pocket_camera: Option<()>,
    bandai_tama5: Option<()>,
    hu_c3: Option<()>,
    hu_c1: Option<()>,
}

impl CartridgeElements {
    fn new(value: u8, num_rom_banks: usize, num_ram_banks: usize) -> Result<Self, CartridgeError> {
        Ok(match value {
            0x00 => Self { ..Default::default() },
            0x01 => Self {
                mbc: Some(MemoryBankController::MBC1(MemoryBankController1::new(
                    num_rom_banks,
                    num_ram_banks,
                ))),
                ..Default::default()
            },
            0x02 => Self {
                mbc: Some(MemoryBankController::MBC1(MemoryBankController1::new(
                    num_rom_banks,
                    num_ram_banks,
                ))),
                ram: Some(()),
                ..Default::default()
            },
            0x03 => Self {
                mbc: Some(MemoryBankController::MBC1(MemoryBankController1::new(
                    num_rom_banks,
                    num_ram_banks,
                ))),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            0x05 => Self {
                mbc: Some(MemoryBankController::MBC2(MemoryBankController2::new(num_rom_banks))),
                ..Default::default()
            },
            0x06 => Self {
                mbc: Some(MemoryBankController::MBC2(MemoryBankController2::new(num_rom_banks))),
                battery: Some(()),
                ..Default::default()
            },
            0x08 => Self { ram: Some(()), ..Default::default() },
            0x09 => Self { ram: Some(()), battery: Some(()), ..Default::default() },
            0x0B => Self { mmm01: Some(()), ..Default::default() },
            0x0C => Self { mmm01: Some(()), ram: Some(()), ..Default::default() },
            0x0D => Self {
                mmm01: Some(()),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            0x0F => Self {
                mbc: Some(MemoryBankController::MBC3(MemoryBankController3::new(
                    num_rom_banks,
                    num_ram_banks,
                    true,
                ))),
                battery: Some(()),
                ..Default::default()
            },
            0x10 => Self {
                mbc: Some(MemoryBankController::MBC3(MemoryBankController3::new(
                    num_rom_banks,
                    num_ram_banks,
                    true,
                ))),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            0x11 => Self {
                mbc: Some(MemoryBankController::MBC3(MemoryBankController3::new(
                    num_rom_banks,
                    num_ram_banks,
                    false,
                ))),
                ..Default::default()
            },
            0x12 => Self {
                mbc: Some(MemoryBankController::MBC3(MemoryBankController3::new(
                    num_rom_banks,
                    num_ram_banks,
                    false,
                ))),
                ram: Some(()),
                ..Default::default()
            },
            0x13 => Self {
                mbc: Some(MemoryBankController::MBC3(MemoryBankController3::new(
                    num_rom_banks,
                    num_ram_banks,
                    false,
                ))),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            0x19 => Self { mbc: Some(MemoryBankController::MBC5), ..Default::default() },
            0x1A => Self {
                mbc: Some(MemoryBankController::MBC5),
                ram: Some(()),
                ..Default::default()
            },
            0x1B => Self {
                mbc: Some(MemoryBankController::MBC5),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            0x1C => Self {
                mbc: Some(MemoryBankController::MBC5),
                rumble: Some(()),
                ..Default::default()
            },
            0x1D => Self {
                mbc: Some(MemoryBankController::MBC5),
                rumble: Some(()),
                ram: Some(()),
                ..Default::default()
            },
            0x1E => Self {
                mbc: Some(MemoryBankController::MBC5),
                rumble: Some(()),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            0x20 => Self { mbc: Some(MemoryBankController::MBC6), ..Default::default() },
            0x22 => Self {
                mbc: Some(MemoryBankController::MBC7),
                sensor: Some(()),
                rumble: Some(()),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            0xFC => Self { pocket_camera: Some(()), ..Default::default() },
            0xFD => Self { bandai_tama5: Some(()), ..Default::default() },
            0xFE => Self { hu_c3: Some(()), ..Default::default() },
            0xFF => Self {
                hu_c1: Some(()),
                ram: Some(()),
                battery: Some(()),
                ..Default::default()
            },
            unknown => return Err(CartridgeError::UnknownCartridgeType(unknown)),
        })
    }
}

impl Default for CartridgeElements {
    fn default() -> Self {
        Self {
            mbc: None,
            battery: None,
            ram: None,
            mmm01: None,
            rumble: None,
            sensor: None,
            pocket_camera: None,
            bandai_tama5: None,
            hu_c3: None,
            hu_c1: None,
        }
    }
}

fn get_num_ram_banks(byte_code: u8) -> Result<usize, CartridgeError> {
    Ok(match byte_code {
        0x00 => 0,
        // 0x01 is listed as unused, but real files do carry it; treat it as the 2KB part it
        // originally meant, which we round up to our single 8KB bank.
        0x01 => 1,
        0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,
        unknown => return Err(CartridgeError::UnknownRamSize(unknown)),
    })
}

/// Codes 0x00-0x08 encode 32KB << code, i.e. 2 banks doubling up to 512 banks. Anything above that
/// is either the unofficial 0x52-0x54 codes or garbage; both would overshift `2 << byte_code`.
fn get_num_rom_banks(byte_code: u8) -> Result<usize, CartridgeError> {
    if byte_code > 0x08 {
        return Err(CartridgeError::UnknownRomSize(byte_code));
    }
    Ok(2 << byte_code)
}
