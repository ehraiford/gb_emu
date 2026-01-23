use crate::{
    bus::{Address, BusAccessible},
    cartridge::cartridge::RAM_BANK_SIZE,
    onboard_devices::rom_and_ram::BankableRam,
};

#[derive(Default)]
pub struct ExternalRam {
    bankable_ram: BankableRam<RAM_BANK_SIZE>,
}

impl ExternalRam {
    pub fn new(num_of_banks: usize) -> Self {
        Self {
            bankable_ram: BankableRam::<RAM_BANK_SIZE>::new(num_of_banks),
        }
    }
}

impl BusAccessible for ExternalRam {
    const MM_DEVICE: crate::bus::MMDevice = crate::bus::MMDevice::ExternalRam;

    fn read(&mut self, address: Address) -> crate::bus::BusAccessOutcome<u8> {
        self.bankable_ram.read(Self::local(address))
    }

    fn write(&mut self, address: Address, value: u8) -> crate::bus::BusAccessOutcome<()> {
        self.bankable_ram.write(Self::local(address), value)
    }

    fn peek(&self, address: Address) -> u8 {
        self.bankable_ram.peek(Self::local(address))
    }
}
