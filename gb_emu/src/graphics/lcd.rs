use crate::{
    bus::{Address, BusAccessFailure},
    game_boy::{GameBoyEvent, notate_event},
    graphics::{
        ppu::{Ppu, PpuTickMode},
        video_ram::{AccessMethod, TargetTileMap},
    },
    io_devices::interrupts::Interrupt,
};

#[derive(Default)]
pub struct LcdRegisters {
    control_flags: u8,

    ly: u8,
    ly_compare: u8,
    status_flags: u8,

    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,

    bgp: u8,
    obp0: u8,
    obp1: u8,
}

impl LcdRegisters {
    const START_ADDRESS: Address = 0xFF40;
    pub const MAX_LY: u8 = 153;

    pub fn apply_bg_palette(&self, raw_color_id: u8) -> u8 {
        self.bgp >> (raw_color_id * 2) & 0b11
    }

    pub fn get_bgp(&self) -> u8 {
        self.bgp
    }

    pub fn get_ly(&self) -> u8 {
        self.ly
    }

    fn set_ly(&mut self, value: u8) {
        self.ly = value;
        if self.ly == self.ly_compare && self.get_status_flag(LcdStatusFlag::LycEqualsLy) {
            notate_event(GameBoyEvent::Interrupt(Interrupt::Lcd));
        }
    }

    pub fn increment_ly(&mut self) -> u8 {
        let new_value = (self.ly + 1) % (Self::MAX_LY + 1);
        self.set_ly(new_value);
        self.get_ly()
    }

    pub fn reset_ly(&mut self) {
        self.set_ly(0)
    }

    pub fn get_scx(&self) -> u8 {
        self.scx
    }
    pub fn get_scy(&self) -> u8 {
        self.scy
    }
    pub fn get_wx(&self) -> u8 {
        self.wx
    }
    pub fn get_wy(&self) -> u8 {
        self.wy
    }

    pub fn get_target_tilemap(&self, x_coordinate: u8) -> TargetTileMap {
        match self.coordinate_in_window(x_coordinate) {
            true => match self.get_control_flag(LcdControlFlag::WindowTileMap) {
                true => TargetTileMap::At0x9C00,
                false => TargetTileMap::At0x9800,
            },
            false => match self.get_control_flag(LcdControlFlag::BackgroundTileMap) {
                true => TargetTileMap::At0x9C00,
                false => TargetTileMap::At0x9800,
            },
        }
    }

    pub fn get_background_window_tiles_address_mode(&self) -> AccessMethod {
        match self.get_control_flag(LcdControlFlag::BackgroundWindowTiles) {
            true => AccessMethod::Method8000,
            false => AccessMethod::Method8800,
        }
    }

    pub fn coordinate_in_window(&self, x_coordinate: u8) -> bool {
        self.get_control_flag(LcdControlFlag::BackgroundWindowEnablePriority)
            && self.ly >= self.wy
            && x_coordinate >= self.wx.saturating_sub(7)
    }

    pub fn window_enabled(&self) -> bool {
        self.get_control_flag(LcdControlFlag::WindowEnable)
    }

    pub fn read(&mut self, address: Address) -> u8 {
        self.peek(address)
    }

    pub fn write(&mut self, address: Address, value: u8) {
        let address = address - Self::START_ADDRESS;
        match address {
            0 => self.set_control_flags(value),
            1 => self.status_flags = value,
            2 => self.scy = value,
            3 => self.scx = value,
            4 => BusAccessFailure::TriedWritingToReadOnlyMemory.into(),
            5 => self.ly_compare = value,
            6 => notate_event(GameBoyEvent::StartOamDmaTransfer(value)),
            7 => self.bgp = value,
            8 => self.obp0 = value,
            9 => self.obp1 = value,
            0xA => self.wy = value,
            0xB => self.wx = value,
            _ => unreachable!("Nothing should be able to reach to this"),
        };
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
            7 => self.bgp,
            8 => self.obp0,
            9 => self.obp1,
            0xA => self.wy,
            0xB => self.wx,
            _ => unreachable!("Nothing should be able to reach to this"),
        }
    }

    fn get_control_flag(&self, flag: LcdControlFlag) -> bool {
        (self.control_flags >> flag.get_index()) & 0b1 == 1
    }
    fn set_control_flags(&mut self, value: u8) {
        let enable_index = LcdControlFlag::LcdPpuEnable.get_index();
        let new_enable_val = value >> enable_index;
        let old_enable_val = self.control_flags >> enable_index;
        self.control_flags = value;
        if new_enable_val ^ old_enable_val == 1 {
            notate_event(GameBoyEvent::ChangeLCdPpuState(new_enable_val == 1))
        }
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
        let (shift, _) = flag.get_shift_and_mask();
        let mut status = self.status_flags;
        status &= 0b1 << shift;
        status |= (value as u8) << shift;
        self.status_flags = status;
    }

    fn set_ppu_mode(&mut self, mode: PpuTickMode) {
        let status = self.status_flags & !0b11;
        self.status_flags = status | u8::from(mode);
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

pub enum LcdRegister {
    LY,
    LYC,
    STAT,
    SCY,
    SCX,
    WY,
    WX,
    BGP,
    OBP1,
    OBP0,
}
