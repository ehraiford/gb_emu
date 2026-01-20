use crate::bus::{Address, BusAccessible, MMDevice, MemoryAccessResult};

#[derive(Default)]
pub struct IoRegisters {
    lcd_registers: LcdRegisters,
}

impl IoRegisters {}

impl BusAccessible for IoRegisters {
    const MM_DEVICE: MMDevice = MMDevice::IoRegisters;

    fn read(&mut self, address: Address) -> MemoryAccessResult<u8> {
        todo!()
    }

    fn write(&mut self, address: Address, value: u8) -> MemoryAccessResult<()> {
        todo!()
    }

    fn peek(&self, address: Address) -> MemoryAccessResult<u8> {
        todo!()
    }
}

enum IoSection {
    JoypadInput,
    SerialTransfer,
    TimerAndDivider,
    Interrupts,
    Audio,
    WavePattern,
    Lcd,
    OamDmaTransfer,
    Keys,
    VramDma,
    Ir,
    BgObjPalettes,
    ObjectPriorityMode,
    WramBankSelect,
}

impl IoSection {
    /// Gets the range (global) for a IO Register section.
    const fn get_range(&self) -> (u16, u16) {
        match self {
            IoSection::JoypadInput => (0xFF00, 0xFF01),
            IoSection::SerialTransfer => (0xFF01, 0xFF03),
            IoSection::TimerAndDivider => (0xFF04, 0xFF08),
            IoSection::Interrupts => (0xFF0F, 0xFF10),
            IoSection::Audio => (0xFF10, 0xFF27),
            IoSection::WavePattern => (0xFF30, 0xFF40),
            IoSection::Lcd => (0xFF40, 0xFF4C),
            IoSection::OamDmaTransfer => (0xFF46, 0xFF47),
            IoSection::Keys => (0xFF4C, 0xFF4E),
            IoSection::VramDma => (0xFF4F, 0xFF50),
            IoSection::Ir => (0xFF56, 0xFF57),
            IoSection::BgObjPalettes => (0xFF68, 0xFF6C),
            IoSection::ObjectPriorityMode => (0xFF6C, 0xFF6D),
            IoSection::WramBankSelect => (0xFF70, 0xFF71),
        }
    }
}

#[derive(Default)]
pub struct LcdRegisters {
    lcd_control_flags: u8,
}

impl LcdRegisters {
    const START_ADDRESS: Address = 0xFF40;

    fn get_flag(&self, flag: LcdControlFlag) -> bool {
        (self.lcd_control_flags >> flag.get_index()) & 0b1 == 1
    }
    fn set_flag(&mut self, flag: LcdControlFlag, value: bool) {
        let index = flag.get_index();
        self.lcd_control_flags &= 0b1 << index;
        self.lcd_control_flags |= value as u8;
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
    const fn get_index(&self) -> usize {
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

struct LcdStatusRegisters {
    ly_coordinate: u8,
    ly_compare: u8,
    status: u8,
}

enum LcdStatusFlag {}
