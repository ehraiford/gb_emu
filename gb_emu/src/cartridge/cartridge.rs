use crate::{
    bus::{BusAccessible, MemoryAccessResult},
    cartridge::{header::Header, rom_bank::RomBank00},
};

pub struct Cartridge {
    header: Header,
    rom_bank00: RomBank00,
}

impl Cartridge {
    pub fn new(data: &[u8]) -> Self {
        let header = Header::new(&data[0x100..0x150]);
        todo!()
    }
}

// A trait that all items on the bus that come from the cartridge should implement
pub trait PartOfCartridge: BusAccessible + Default {
    fn load_from_cartridge(&mut self, data_chunk: &[u8]) -> MemoryAccessResult<()> {
        *self = Self::default();

        for (i, byte) in data_chunk.iter().enumerate() {
            self.write(i as u16, *byte)?;
        }

        Ok(())
    }
}
