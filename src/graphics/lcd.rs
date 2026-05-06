use crate::{
    bus::{Address, BusAccessFailure},
    game_boy::{GameBoyEvent, notate_event},
    graphics::{
        oam::PaletteChoice,
        ppu::PpuMode,
        video_ram::{AccessMethod, TargetTileMap},
    },
    io_devices::interrupts::Interrupt,
};

#[derive(Default)]
pub struct Lcd {
    control_flags: u8,

    ly: u8,
    ly_compare: u8,
    status_flags: u8,

    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,

    oam_dma_start_address: u8,

    bgp: u8,
    obp0: u8,
    obp1: u8,
}

impl Lcd {
    const START_ADDRESS: Address = 0xFF40;
    pub const MAX_LY: u8 = 153;

    fn get_local_address(address: Address) -> Address {
        address - Self::START_ADDRESS
    }

    pub fn is_ppu_enabled(&self) -> bool {
        self.get_control_flag(LcdControlFlag::LcdPpuEnable)
    }

    pub fn get_obp(&self, palette_choice: PaletteChoice) -> u8 {
        match palette_choice {
            PaletteChoice::OBP0 => self.obp0,
            PaletteChoice::OBP1 => self.obp1,
        }
    }

    pub fn get_bgp(&self) -> u8 {
        self.bgp
    }

    pub fn get_ly(&self) -> u8 {
        self.ly
    }

    fn set_ly(&mut self, value: u8) {
        self.ly = value;
        if self.ly == self.ly_compare {
            self.status_flags |= 0b100;
            if self.get_status_flag(LcdStatusFlag::LycIntSelect) {
                notate_event(GameBoyEvent::Interrupt(Interrupt::Stat));
            }
        } else {
            self.status_flags &= !0b100;
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

    pub fn get_object_size(&self) -> u8 {
        match self.get_control_flag(LcdControlFlag::ObjSize) {
            true => 16,
            false => 8,
        }
    }

    pub fn object_enabled(&self) -> bool {
        self.get_control_flag(LcdControlFlag::ObjEnable)
    }

    pub fn get_target_tilemap(&self, x_coordinate: u8) -> TargetTileMap {
        let flag = match self.window_enabled() && self.coordinate_in_window(x_coordinate) {
            true => LcdControlFlag::WindowTileMap,
            false => LcdControlFlag::BackgroundTileMap,
        };

        match self.get_control_flag(flag) {
            true => TargetTileMap::At0x9C00,
            false => TargetTileMap::At0x9800,
        }
    }

    pub fn get_background_window_tiles_address_mode(&self) -> AccessMethod {
        match self.get_control_flag(LcdControlFlag::BackgroundWindowTiles) {
            true => AccessMethod::Method8000,
            false => AccessMethod::Method8800,
        }
    }

    pub fn coordinate_in_window(&self, x_coordinate: u8) -> bool {
        self.ly >= self.wy && x_coordinate >= self.wx.saturating_sub(7)
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
            1 => self.status_flags = (value & !0b111) | (self.status_flags & 0b111),
            2 => self.scy = value,
            3 => self.scx = value,
            4 => BusAccessFailure::TriedWritingToReadOnlyMemory.into(),
            5 => {
                self.ly_compare = value;
                if self.ly == self.ly_compare {
                    self.status_flags |= 0b100;
                } else {
                    self.status_flags &= !0b100;
                }
            },
            6 => {
                self.oam_dma_start_address = value;
                notate_event(GameBoyEvent::StartOamDmaTransfer(value))
            },
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
            6 => self.oam_dma_start_address,
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
        LcdControlFlag::check_for_change_in_lcd_ppu_state(self.control_flags, value);
        LcdControlFlag::check_for_change_in_object_enable(self.control_flags, value);
        self.control_flags = value;
    }

    fn get_status_flag(&self, flag: LcdStatusFlag) -> bool {
        let (shift, mask) = flag.get_shift_and_mask();
        (self.status_flags >> shift) & mask == 1
    }

    fn mode_transition_raises_interrupt(&self, mode: PpuMode) -> bool {
        if mode == PpuMode::DrawingPixels {
            return false;
        }

        let mode_number = u8::from(mode);
        let flag_index = mode_number + 3;
        let isolated_flag = (self.status_flags >> flag_index) & 0b1;

        isolated_flag == 1
    }

    pub fn set_ppu_mode(&mut self, mode: PpuMode) {
        if self.mode_transition_raises_interrupt(mode) {
            notate_event(GameBoyEvent::Interrupt(Interrupt::Stat));
        }

        let status = self.status_flags & !0b11;
        self.status_flags = status | u8::from(mode);
    }

    pub fn object_on_scanline(&self, object_y_position: u8) -> bool {
        let sprite_base = (object_y_position.wrapping_add(self.get_object_size())) as u16;
        let adjusted_ly = (self.get_ly() + 16) as u16;

        (object_y_position as u16..sprite_base).contains(&adjusted_ly)
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

    fn check_for_change_in_lcd_ppu_state(old_register_value: u8, new_register_value: u8) {
        let enable_index = LcdControlFlag::LcdPpuEnable.get_index();
        let new_enable_val = new_register_value >> enable_index;
        let old_enable_val = old_register_value >> enable_index;
        if new_enable_val ^ old_enable_val == 1 {
            notate_event(GameBoyEvent::ChangeLcdPpuEnabled(new_enable_val == 1))
        }
    }

    fn check_for_change_in_object_enable(old_register_value: u8, new_register_value: u8) {
        let enable_index = LcdControlFlag::ObjEnable.get_index();
        let new_enable_val = (new_register_value >> enable_index) & 0b1;
        if new_enable_val == 0 {
            let old_enable_val = old_register_value >> (enable_index & 0b1);
            if old_enable_val == 1 {
                notate_event(GameBoyEvent::ObjectsDisabled);
            }
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
    Control,
    YCoordinate,
    YCompare,
    Status,
    ScrollY,
    ScrollX,
    WindowY,
    WindowX,
    BackgroundPalette,
    ObjectPalette1,
    ObjectPalette0,

    // GameBoy Color Only
    GameBoyColor(GBCColorPaletteRegister),
}

impl LcdRegister {
    fn get_index(&self) -> usize {
        match self {
            LcdRegister::Control => 0,
            LcdRegister::YCoordinate => 1,
            LcdRegister::YCompare => 2,
            LcdRegister::Status => 3,
            LcdRegister::ScrollY => 4,
            LcdRegister::ScrollX => 5,
            LcdRegister::WindowY => 7,
            LcdRegister::WindowX => 8,
            LcdRegister::BackgroundPalette => 9,
            LcdRegister::ObjectPalette0 => 0xA,
            LcdRegister::ObjectPalette1 => 0xB,
            LcdRegister::GameBoyColor(gbccolor_palette_register) => gbccolor_palette_register.get_index(),
        }
    }

    fn from_global_address(address: Address) -> Option<Self> {
        match Lcd::get_local_address(address) {
            0 => Some(LcdRegister::Control),
            1 => Some(LcdRegister::YCoordinate),
            2 => Some(LcdRegister::YCompare),
            3 => Some(LcdRegister::Status),
            4 => Some(LcdRegister::ScrollY),
            5 => Some(LcdRegister::ScrollX),
            7 => Some(LcdRegister::WindowY),
            8 => Some(LcdRegister::WindowX),
            9 => Some(LcdRegister::BackgroundPalette),
            0xA => Some(LcdRegister::ObjectPalette0),
            0xB => Some(LcdRegister::ObjectPalette1),
            _ => GBCColorPaletteRegister::from_global_address(address).map(Self::GameBoyColor),
        }
    }
}

pub enum GBCColorPaletteRegister {
    BackgroundSpec,
    BackgroundData,
    ObjectSpec,
    ObjectData,
}

impl GBCColorPaletteRegister {
    fn get_index(&self) -> usize {
        match self {
            GBCColorPaletteRegister::BackgroundSpec => 0xC,
            GBCColorPaletteRegister::BackgroundData => 0xD,
            GBCColorPaletteRegister::ObjectSpec => 0xE,
            GBCColorPaletteRegister::ObjectData => 0xF,
        }
    }
    fn from_global_address(address: Address) -> Option<Self> {
        match Lcd::get_local_address(address) {
            0x28 => Some(Self::BackgroundSpec),
            0x29 => Some(Self::BackgroundData),
            0x2a => Some(Self::ObjectSpec),
            0x2b => Some(Self::ObjectData),
            _ => None,
        }
    }
}
