type MemoryAccessResult<T> = Result<T, MemoryAccessError>;


pub struct Bus {

}

impl Bus {
    pub fn read(&mut self, address: u16) -> MemoryAccessResult<u8> {
        todo!()
    }
    pub fn write(&mut self, address: u16, value: u8) -> MemoryAccessResult<()> {
        todo!()
    }
    pub fn peek(&self, address: u16) -> MemoryAccessResult<u8> {
        todo!()
    }
}

struct MMTableEntry {
    device: MMDevice,
    base_address: u16, 
    size: u16,
}

impl MMTableEntry {
    const fn new(device: MMDevice, base_address: u16, end_address: u16) -> Self {
        Self {
            device,
            base_address,
            size: end_address - base_address,
        }
    }
}


const MEMORY_MAP: &[MMTableEntry] = &[
    MMTableEntry::new(MMDevice::RomBank00, 0x0000, 0x4000),
    MMTableEntry::new(MMDevice::CartridgeRomBank, 0x4000, 0x8000),
    MMTableEntry::new(MMDevice::VideoRam, 0x8000, 0xA000), 
    MMTableEntry::new(MMDevice::VideoRam, 0xA000, 0xC000), 
    MMTableEntry::new(MMDevice::WorkRam00, 0xC000, 0xD000), 
    MMTableEntry::new(MMDevice::SwitchableBankWorkRam, 0xD000, 0xE000),
    MMTableEntry::new(MMDevice::EchoRam, 0xE000, 0xFE00), 
    MMTableEntry::new(MMDevice::ObjectAttributeMemory, 0xFE00, 0xFEA0), 
    MMTableEntry::new(MMDevice::Unusable, 0xFEA0, 0xFF00),
    MMTableEntry::new(MMDevice::IoRegisters, 0xFF00, 0xFF80),
    MMTableEntry::new(MMDevice::HighRam, 0xFF80, 0xFFFF),
    MMTableEntry {
        device: MMDevice::InterruptEnableRegister,
        base_address: 0xFFFF,
        size: 0x1,
    } 
];

fn get_mm_table_entry_for_device(device: MMDevice) -> &'static MMTableEntry {
    MEMORY_MAP.iter().find(|e| e.device == device).expect("There is no device not in the map.")
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MMDevice {
    RomBank00,
    CartridgeRomBank,
    VideoRam,
    ExternalRam,
    WorkRam00,
    SwitchableBankWorkRam, // GameBoy Color Only
    EchoRam, // Mirror of C000-DDFF
    ObjectAttributeMemory, 
    Unusable, 
    IoRegisters,
    HighRam,
    InterruptEnableRegister, 
}




pub trait BusAccessible {
    // This would be better as a const but you can't make trait objects of traits with associated consts
    fn get_enum_device(&self) -> MMDevice; 
    fn read(&mut self, address: u16) -> MemoryAccessResult<u8>;
    fn write(&mut self, address: u16, value: u8) -> MemoryAccessResult<()>;
    fn peek(&self, address: u16) -> MemoryAccessResult<u8>;
}

pub enum MemoryAccessError {

}

