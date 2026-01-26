use crate::{
    bus::{Address, BusAccessFailure, BusAccessOutcome},
    game_boy::Change,
    graphics::ppu::VideoMemory,
    io_registers::IoSection,
};
pub struct LcdRegisters {
    control_flags: u8,

    ly: u8,
    ly_compare: u8,
    status_flags: u8,

    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,

    palette: Palette,
}

impl LcdRegisters {
    const START_ADDRESS: Address = 0xFF40;

    pub fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        BusAccessOutcome(self.peek(address), vec![])
    }

    pub fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        let address = address - Self::START_ADDRESS;
        match address {
            0 => self.control_flags = value,
            1 => self.status_flags = value,
            2 => self.scy = value,
            3 => self.scx = value,
            4 => self.ly = value,
            5 => self.ly_compare = value,
            6 => return BusAccessOutcome((), vec![Change::StartOamDmaTransfer(value)]),
            7 => self.palette.dmg_palette = value,
            8 => self.palette.ogp0 = value,
            9 => self.palette.ogp1 = value,
            0xA => self.wy = value,
            0xB => self.wx = value,
            _ => unreachable!("Nothing should be able to reach to this"),
        };
        BusAccessOutcome::default_outcome()
    }

    pub fn peek(&self, address: Address) -> u8 {
        let address = address - Self::START_ADDRESS;
        match address {
            0 => self.control_flags,
            1 => self.status_flags,
            2 => self.scy,
            3 => self.scx,
            4 => self.ly,
            5 => self.ly_compare,
            6 => BusAccessFailure::TriedAccessingUnusableMemory.into(),
            7 => self.palette.dmg_palette,
            8 => self.palette.ogp0,
            9 => self.palette.ogp1,
            0xA => self.wy,
            0xB => self.wx,
            _ => unreachable!("Nothing should be able to reach to this"),
        }
    }

    fn get_control_flag(&self, flag: LcdControlFlag) -> bool {
        (self.control_flags >> flag.get_index()) & 0b1 == 1
    }
    fn set_control_flag(&mut self, flag: LcdControlFlag, value: bool) {
        let index = flag.get_index();
        self.control_flags &= 0b1 << index;
        self.control_flags |= value as u8;
    }

    fn get_status_flag(&self, flag: LcdStatusFlag) -> bool {
        let (shift, mask) = flag.get_shift_and_mask();
        (self.status_flags >> shift) & mask == 1
    }
    fn set_status_flag(&mut self, flag: LcdStatusFlag, value: bool) {
        let (shift, mask) = flag.get_shift_and_mask();
        let mut status = self.status_flags;
        status &= 0b1 << shift;
        status |= (value as u8) << shift;
        self.status_flags = status;
    }
    pub fn get_ppu_mode(&self) -> PpuMode {
        unsafe { std::mem::transmute(self.status_flags & 0b11) }
    }
    fn set_ppu_mode(&mut self, mode: PpuMode) {
        let status = self.status_flags & !0b11;
        self.status_flags = status | mode as u8;
    }
}

impl Default for LcdRegisters {
    fn default() -> Self {
        let mut this = Self {
            control_flags: Default::default(),
            ly: Default::default(),
            ly_compare: Default::default(),
            status_flags: Default::default(),
            scy: Default::default(),
            scx: Default::default(),
            wy: Default::default(),
            wx: Default::default(),
            palette: Default::default(),
        };

        for (address, value) in IoSection::Lcd.get_default_value_mapping() {
            this.write(address, value);
        }

        this
    }
}

impl VideoMemory for LcdRegisters {
    fn update_ppu_mode(&mut self, mode: PpuMode) {
        self.set_ppu_mode(mode);
    }
}

enum LcdControlFlag {
    LcdPpuEnable,
    WindowTileMap,
    WindowEnable,
    BackgroundWindowTiles,
    BackgroundTileMap,
    ObjSize,
    ObjEnable,
    BackgroundWindowEnablePriority,
}

impl LcdControlFlag {
    fn get_index(&self) -> usize {
        match self {
            LcdControlFlag::LcdPpuEnable => 7,
            LcdControlFlag::WindowTileMap => 6,
            LcdControlFlag::WindowEnable => 5,
            LcdControlFlag::BackgroundWindowTiles => 4,
            LcdControlFlag::BackgroundTileMap => 3,
            LcdControlFlag::ObjSize => 2,
            LcdControlFlag::ObjEnable => 1,
            LcdControlFlag::BackgroundWindowEnablePriority => 0,
        }
    }
}

enum LcdStatusFlag {
    LycIntSelect,
    Mode2IntSelect,
    Mode1IntSelect,
    Mode0IntSelect,
    LycEqualsLy,
    PpuMode,
}

impl LcdStatusFlag {
    fn get_shift_and_mask(&self) -> (u8, u8) {
        match self {
            LcdStatusFlag::LycIntSelect => (6, 0b1),
            LcdStatusFlag::Mode2IntSelect => (5, 0b1),
            LcdStatusFlag::Mode1IntSelect => (4, 0b1),
            LcdStatusFlag::Mode0IntSelect => (3, 0b1),
            LcdStatusFlag::LycEqualsLy => (2, 0b1),
            LcdStatusFlag::PpuMode => unreachable!("This shouldn't be called anywhere to cause this"),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PpuMode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    OamScan = 2,
    DrawingPixels = 3,
}

const DMG_PALETTE_ADDRESS: Address = 0xFF47;

pub struct Palette {
    dmg_palette: u8,
    ogp0: u8,
    ogp1: u8,
    cgb_palettes: [u8; 64],
}

impl Palette {
    pub fn get_dmg_palette(&self) -> [MonochromeColor; 4] {
        [
            MonochromeColor::from(self.dmg_palette & 0b11),
            MonochromeColor::from((self.dmg_palette >> 2) & 0b11),
            MonochromeColor::from((self.dmg_palette >> 4) & 0b11),
            MonochromeColor::from((self.dmg_palette >> 6) & 0b11),
        ]
    }

    fn get_cgb(&self) -> u8 {
        todo!()
    }
    fn set_cgb(&mut self, _value: u8) {
        todo!()
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self { dmg_palette: 0, cgb_palettes: [0; 64], ogp0: 0, ogp1: 0 }
    }
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug)]
enum MonochromeColor {
    #[default]
    White = 0,
    LightGray,
    DarkGray,
    Black,
}

impl From<u8> for MonochromeColor {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::White,
            1 => Self::LightGray,
            2 => Self::DarkGray,
            _ => Self::Black,
        }
    }
}
