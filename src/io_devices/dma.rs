use crate::{
    bus::{Address, Bus, MemoryTarget},
    game_boy::{GameBoyEvent, notate_event},
};

#[derive(Default)]
pub struct OamDma {
    remaining_bytes: u8,
    source_address: Address,
    currently_transferring: bool,
}

impl OamDma {
    const TOTAL_TRANSFER_SIZE: u8 = 160;

    fn determine_source_address(input: u8) -> Address {
        input as Address * 0x100
    }

    fn get_destination_address(&self) -> Address {
        MemoryTarget::ObjectAttributeMemory.get_base_address() + Self::TOTAL_TRANSFER_SIZE as u16
            - self.remaining_bytes as u16
    }

    pub fn initiate_transfer(&mut self, input: u8) {
        self.remaining_bytes = Self::TOTAL_TRANSFER_SIZE;
        self.source_address = Self::determine_source_address(input);
        self.currently_transferring = true;
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        if !self.is_transferring() {
            return;
        }
        let byte = bus.peek(self.source_address);

        let destination_address = self.get_destination_address();

        bus.oam_dma_transfer(destination_address, byte);

        self.remaining_bytes -= 1;
        self.source_address += 1;

        if self.remaining_bytes == 0 {
            self.end_transfer();
        }
    }

    fn is_transferring(&self) -> bool {
        self.currently_transferring
    }
    fn end_transfer(&mut self) {
        self.currently_transferring = false;
        notate_event(GameBoyEvent::EndOamDmaTransfer);
    }
}

