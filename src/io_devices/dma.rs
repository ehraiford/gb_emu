use crate::{
    bus::{Address, Bus, MemoryTarget},
    game_boy::{EventQueue, GameBoyEvent},
};

#[derive(Default)]
pub struct OamDma {
    remaining_bytes: u8,
    source_address: Address,
    state: DmaState,
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
        self.state = DmaState::Initializing;
    }

    pub fn tick(&mut self, bus: &mut Bus, events: &mut EventQueue) {
        match self.state {
            DmaState::Idle => (),
            DmaState::Initializing => self.state = DmaState::Transferring,
            DmaState::Transferring => self.transfer_byte(bus, events),
        }
    }

    fn transfer_byte(&mut self, bus: &mut Bus, events: &mut EventQueue) {
        let byte = bus.peek(self.source_address);

        let destination_address = self.get_destination_address();

        bus.oam_dma_transfer(destination_address, byte);

        self.remaining_bytes -= 1;
        self.source_address += 1;

        if self.remaining_bytes == 0 {
            self.end_transfer(events);
        }
    }

    fn end_transfer(&mut self, events: &mut EventQueue) {
        self.state = DmaState::Idle;
        events.push(GameBoyEvent::EndOamDmaTransfer);
    }
}

#[derive(Default)]
enum DmaState {
    #[default]
    Idle,
    Initializing,
    Transferring,
}
