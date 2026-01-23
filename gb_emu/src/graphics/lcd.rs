use crate::{
    bus::{Address, BusAccessOutcome},
    graphics::ppu::VideoMemory,
};

#[derive(Default)]
pub struct LcdRegisters {
    lcd_control_flags: u8,

    ly_coordinate: u8,
    ly_compare: u8,
    status_flags: u8,

    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,

    background_palette: u8,
}

impl LcdRegisters {
    const START_ADDRESS: Address = 0xFF40;

    pub fn read(&mut self, address: Address) -> BusAccessOutcome<u8> {
        let address = address - Self::START_ADDRESS;
        BusAccessOutcome::default_outcome()
    }

    pub fn write(&mut self, address: Address, value: u8) -> BusAccessOutcome<()> {
        let address = address - Self::START_ADDRESS;
        BusAccessOutcome::default_outcome()
    }

    pub fn peek(&self, address: Address) -> u8 {
        let address = address - Self::START_ADDRESS;
        todo!()
    }

    fn get_control_flag(&self, flag: LcdControlFlag) -> bool {
        (self.lcd_control_flags >> flag.get_index()) & 0b1 == 1
    }
    fn set_control_flag(&mut self, flag: LcdControlFlag, value: bool) {
        let index = flag.get_index();
        self.lcd_control_flags &= 0b1 << index;
        self.lcd_control_flags |= value as u8;
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
