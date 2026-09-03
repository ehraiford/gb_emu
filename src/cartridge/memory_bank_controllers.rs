use std::time::{Duration, SystemTime};

use crate::cartridge::cartridge::{CartridgeDevice, CartridgeError, RAM_BANK_SIZE};
use crate::cartridge::save_data::{SaveData, SaveDataReader};
use crate::{
    bus::{Address, BusDefault},
    onboard_memory::rom_and_ram::{RamBank, RomBank},
};
const RAMG_OPEN: u8 = 0b0000_1010;

#[derive(Debug, Clone)]
pub enum MemoryBankController {
    MBC1(MemoryBankController1),
    MBC2(MemoryBankController2),
    MBC3(MemoryBankController3),
    MBC5(MemoryBankController5),
    MBC6,
    MBC7,
}

impl MemoryBankController {
    /// Which controllers actually have their behaviour wired up. The rest are still parsed out of
    /// the header so a caller can report them by name, but every accessor on them is `todo!()`, so
    /// loading one has to be refused up front rather than panicking on the first ROM read.
    pub fn is_implemented(&self) -> bool {
        matches!(self, Self::MBC1(_) | Self::MBC2(_) | Self::MBC3(_) | Self::MBC5(_))
    }
}
impl BankController for MemoryBankController {
    fn write(
        &mut self,
        address: Address,
        value: u8,
        ram_banks: &mut [RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) {
        match self {
            MemoryBankController::MBC1(mbc) => mbc.write(address, value, ram_banks, device),
            MemoryBankController::MBC2(mbc) => mbc.write(address, value, ram_banks, device),
            MemoryBankController::MBC3(mbc) => mbc.write(address, value, ram_banks, device),
            MemoryBankController::MBC5(mbc) => mbc.write(address, value, ram_banks, device),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    fn read(
        &mut self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        match self {
            MemoryBankController::MBC1(mbc) => mbc.read(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC2(mbc) => mbc.read(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC3(mbc) => mbc.read(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC5(mbc) => mbc.read(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    fn peek(
        &self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        match self {
            MemoryBankController::MBC1(mbc) => mbc.peek(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC2(mbc) => mbc.peek(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC3(mbc) => mbc.peek(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC5(mbc) => mbc.peek(address, rom_banks, ram_banks, device),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize> {
        match self {
            MemoryBankController::MBC1(mbc) => mbc.get_bank_number(device),
            MemoryBankController::MBC2(mbc) => mbc.get_bank_number(device),
            MemoryBankController::MBC3(mbc) => mbc.get_bank_number(device),
            MemoryBankController::MBC5(mbc) => mbc.get_bank_number(device),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    fn bank_name(&self) -> &'static str {
        match self {
            MemoryBankController::MBC1(mbc) => mbc.bank_name(),
            MemoryBankController::MBC2(mbc) => mbc.bank_name(),
            MemoryBankController::MBC3(mbc) => mbc.bank_name(),
            MemoryBankController::MBC5(mbc) => mbc.bank_name(),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    fn load_save_data(&mut self, data: &mut SaveDataReader) -> Result<(), CartridgeError> {
        match self {
            MemoryBankController::MBC1(mbc) => mbc.load_save_data(data),
            MemoryBankController::MBC2(mbc) => mbc.load_save_data(data),
            MemoryBankController::MBC3(mbc) => mbc.load_save_data(data),
            MemoryBankController::MBC5(mbc) => mbc.load_save_data(data),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    fn append_save_data(&self, save_data: &mut SaveData) {
        match self {
            MemoryBankController::MBC1(mbc) => mbc.append_save_data(save_data),
            MemoryBankController::MBC2(mbc) => mbc.append_save_data(save_data),
            MemoryBankController::MBC3(mbc) => mbc.append_save_data(save_data),
            MemoryBankController::MBC5(mbc) => mbc.append_save_data(save_data),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }
}

pub(crate) trait BankController {
    fn bank_name(&self) -> &'static str;

    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize>;

    fn read(
        &mut self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8;
    fn write(&mut self, address: Address, value: u8, ram_banks: &mut [RamBank<RAM_BANK_SIZE>], device: CartridgeDevice);
    fn peek(
        &self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8;

    fn load_save_data(&mut self, _data: &mut SaveDataReader) -> Result<(), CartridgeError> {
        Ok(())
    }
    fn append_save_data(&self, _save_data: &mut SaveData) {}
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryBankController1 {
    // Registers
    ram_gate_register: u8,
    bank_register_1: u8,
    bank_register_2: u8,
    bank_2_mode_register: u8,

    // Registers Interpretations
    upper_rom_bank_number: usize,
    lower_rom_bank_number: usize,
    ram_bank_number: Option<usize>,

    // size masks from the cartridge header
    rom_bank_mask: usize,
    ram_bank_mask: Option<usize>,
}

impl MemoryBankController1 {
    const RAMG_REGISTER_MASK: u8 = 0x0F;
    const BANK_REGISTER_1_MASK: u8 = 0b0001_1111;
    const BANK_REGISTER_2_MASK: u8 = 0b0000_0011;
    const BANK_2_MODE_REGISTER_MASK: u8 = 0b0000_0001;

    pub fn new(num_rom_banks: usize, num_ram_banks: usize) -> Self {
        debug_assert!(
            num_rom_banks.is_power_of_two(),
            "bank masking assumes a power-of-two bank count"
        );
        debug_assert!(num_ram_banks == 0 || num_ram_banks.is_power_of_two());
        Self {
            bank_register_1: 1,
            ram_gate_register: 0,
            bank_register_2: 0,
            bank_2_mode_register: 0,

            lower_rom_bank_number: 0,
            upper_rom_bank_number: 1,
            ram_bank_number: None,

            rom_bank_mask: num_rom_banks - 1,
            ram_bank_mask: (num_ram_banks > 0).then(|| num_ram_banks - 1),
        }
    }

    fn write_ram_gate_register(&mut self, value: u8) {
        self.ram_gate_register = value & Self::RAMG_REGISTER_MASK;
    }
    fn write_bank_register_1(&mut self, mut value: u8) {
        value &= Self::BANK_REGISTER_1_MASK;
        if value == 0 {
            value = 1;
        }
        self.bank_register_1 = value;
    }
    fn write_bank_register_2(&mut self, value: u8) {
        self.bank_register_2 = value & Self::BANK_REGISTER_2_MASK;
    }
    fn write_mode_register(&mut self, value: u8) {
        self.bank_2_mode_register = value & Self::BANK_2_MODE_REGISTER_MASK;
    }

    fn recalculate_bank_numbers(&mut self) {
        self.recalculate_lower_rom_bank_number();
        self.recalculate_upper_rom_bank_number();
        self.recalculate_ram_bank_number();
    }
    fn recalculate_upper_rom_bank_number(&mut self) {
        let bank = ((self.bank_register_2 << 5) | (self.bank_register_1)) as usize;
        self.upper_rom_bank_number = bank & self.rom_bank_mask;
    }
    fn recalculate_lower_rom_bank_number(&mut self) {
        let bank = if self.bank_2_mode_register == 0b1 {
            self.bank_register_2 << 5
        } else {
            0
        } as usize;
        self.lower_rom_bank_number = bank & self.rom_bank_mask;
    }
    fn recalculate_ram_bank_number(&mut self) {
        self.ram_bank_number = if self.ram_gate_register != RAMG_OPEN || self.ram_bank_mask.is_none() {
            None
        } else if self.bank_2_mode_register == 0b1 {
            Some((self.bank_register_2 as usize) & self.ram_bank_mask.expect("Already checked above"))
        } else {
            Some(0)
        };
    }
}

impl BankController for MemoryBankController1 {
    fn write(
        &mut self,
        address: Address,
        value: u8,
        ram_banks: &mut [RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) {
        if device == CartridgeDevice::ExternalRam {
            let ram_address = address - CartridgeDevice::ExternalRam.get_starting_address();
            let Some(ram_bank) = self.ram_bank_number.and_then(|i| ram_banks.get_mut(i)) else {
                return;
            };
            ram_bank.write(ram_address, value);
            return;
        }

        match address {
            0x0000..0x2000 => self.write_ram_gate_register(value),
            0x2000..0x4000 => self.write_bank_register_1(value),
            0x4000..0x6000 => self.write_bank_register_2(value),
            0x6000..0x8000 => self.write_mode_register(value),
            _ => unreachable!("RAM addresses were already handled"),
        }

        // cost is negligible to recalculate all of them for any write.
        self.recalculate_bank_numbers();
    }

    fn read(
        &mut self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        self.peek(address, rom_banks, ram_banks, device)
    }

    fn peek(
        &self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        let in_device_address = address - device.get_starting_address();
        let bank_number = self.get_bank_number(device);

        let read_val = match device {
            CartridgeDevice::LowerRomBank | CartridgeDevice::UpperRomBank => {
                rom_banks.get(bank_number.unwrap()).map(|b| b.peek(in_device_address))
            },
            CartridgeDevice::ExternalRam => bank_number
                .and_then(|bank_num| ram_banks.get(bank_num))
                .map(|b| b.peek(in_device_address)),
        };

        read_val.unwrap_or(u8::DEFAULT_BUS_VALUE)
    }

    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize> {
        match device {
            CartridgeDevice::LowerRomBank => Some(self.lower_rom_bank_number),
            CartridgeDevice::UpperRomBank => Some(self.upper_rom_bank_number),
            CartridgeDevice::ExternalRam => self.ram_bank_number,
        }
    }

    fn bank_name(&self) -> &'static str {
        "MBC1"
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryBankController2 {
    // MBC2 is simple enough that we're not gonna bother storing the actual registers
    rom_bank_mask: usize,

    upper_rom_bank_number: usize,
    ram_accessible: bool,
    ram_bank: RamBank<{ Self::MBC2_RAM_SIZE }>, // RAM lives on MBC2 itself
}

impl MemoryBankController2 {
    pub const MBC2_RAM_SIZE: usize = 512;
    const RAMG_REGISTER_MASK: u8 = 0x0F;
    const RAM_VALUE_MASK: u8 = 0xF0; // MBC2 RAM only stores in the right nibble. Top is always open bus value.
    const RAM_ADDRESS_MASK: Address = 0x01FF; // MBC2 RAM addresses wrap in the 10th bit
    const ROMB_REGISTER_MASK: u8 = 0x0F;

    pub fn new(num_rom_banks: usize) -> Self {
        debug_assert!(
            num_rom_banks.is_power_of_two(),
            "bank masking assumes a power-of-two bank count"
        );
        Self {
            rom_bank_mask: num_rom_banks - 1,
            upper_rom_bank_number: 1,
            ram_accessible: false,
            ram_bank: RamBank::<{ Self::MBC2_RAM_SIZE }>::default(),
        }
    }

    fn is_addressing_ramg_not_romb(address: Address) -> bool {
        (address & 0b0000_0001_0000_0000) == 0
    }

    fn write_ram_gate_register(&mut self, value: u8) {
        self.ram_accessible = value & Self::RAMG_REGISTER_MASK == RAMG_OPEN;
    }
    fn write_rom_bank_register(&mut self, mut value: u8) {
        value &= Self::ROMB_REGISTER_MASK;
        if value == 0 {
            value = 1;
        }

        self.upper_rom_bank_number = (value as usize) & self.rom_bank_mask;
    }

    fn get_ram_address(address: Address) -> Address {
        (address - CartridgeDevice::ExternalRam.get_starting_address()) & Self::RAM_ADDRESS_MASK
    }
    fn convert_ram_value(value: u8) -> u8 {
        value | Self::RAM_VALUE_MASK
    }
}

impl BankController for MemoryBankController2 {
    fn write(&mut self, address: Address, value: u8, _: &mut [RamBank<RAM_BANK_SIZE>], device: CartridgeDevice) {
        match device {
            CartridgeDevice::UpperRomBank => return,
            CartridgeDevice::LowerRomBank => match Self::is_addressing_ramg_not_romb(address) {
                true => self.write_ram_gate_register(value),
                false => self.write_rom_bank_register(value),
            },
            CartridgeDevice::ExternalRam => {
                if !self.ram_accessible {
                    return;
                }
                let ram_value = Self::convert_ram_value(value);
                let ram_address = Self::get_ram_address(address);
                self.ram_bank.write(ram_address, ram_value);
            },
        }
    }

    fn read(
        &mut self,
        address: Address,
        rom_banks: &[RomBank],
        _ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        self.peek(address, rom_banks, _ram_banks, device)
    }

    fn peek(
        &self,
        address: Address,
        rom_banks: &[RomBank],
        _: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        let read_value = match device {
            CartridgeDevice::LowerRomBank | CartridgeDevice::UpperRomBank => {
                let bank_number = self.get_bank_number(device).unwrap();
                let in_device_address = address - device.get_starting_address();
                rom_banks.get(bank_number).map(|b| b.read(in_device_address))
            },
            CartridgeDevice::ExternalRam => self
                .ram_accessible
                .then(|| self.ram_bank.peek(Self::get_ram_address(address))),
        };

        read_value.unwrap_or(u8::DEFAULT_BUS_VALUE)
    }

    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize> {
        match device {
            CartridgeDevice::LowerRomBank => Some(0),
            CartridgeDevice::UpperRomBank => Some(self.upper_rom_bank_number),
            CartridgeDevice::ExternalRam => None,
        }
    }

    fn bank_name(&self) -> &'static str {
        "MBC2"
    }

    fn load_save_data(&mut self, data: &mut SaveDataReader) -> Result<(), CartridgeError> {
        self.ram_bank.get_data_mut().copy_from_slice(
            data.read_ram(Self::MBC2_RAM_SIZE)
                .ok_or(CartridgeError::insufficient_save_data())?,
        );
        Ok(())
    }

    fn append_save_data(&self, save_data: &mut SaveData) {
        save_data.append_ram(self.ram_bank.get_data());
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBankController3 {
    rtc: Option<RealTimeClock>,
    ram_timer_gate_register: u8,
    ram_bank_and_rtc_select_register: u8,

    upper_rom_bank_number: usize,
    ram_target: Mbc3RamTarget,

    ram_bank_mask: Option<usize>,
    rom_bank_mask: usize,
}

impl MemoryBankController3 {
    pub fn new(num_rom_banks: usize, num_ram_banks: usize, has_rtc: bool) -> Self {
        let rtc = has_rtc.then(|| RealTimeClock::new());
        debug_assert!(
            num_rom_banks.is_power_of_two(),
            "bank masking assumes a power-of-two bank count"
        );
        debug_assert!(num_ram_banks == 0 || num_ram_banks.is_power_of_two());
        Self {
            rtc,
            ram_timer_gate_register: 0,
            ram_bank_and_rtc_select_register: 0,
            upper_rom_bank_number: 1,
            ram_target: Mbc3RamTarget::Nothing,
            ram_bank_mask: (num_ram_banks > 0).then(|| num_ram_banks - 1),
            rom_bank_mask: num_rom_banks - 1,
        }
    }

    const RAMG_REGISTER_MASK: u8 = 0x0F;
    const ROM_BANK_NUMBER_REGISTER_MASK: u8 = 0b01111111;

    fn write_ram_timer_gate_register(&mut self, value: u8) {
        self.ram_timer_gate_register = value & Self::RAMG_REGISTER_MASK;

        self.recalculate_ram_target();
    }
    fn write_ram_bank_and_rtc_select_register(&mut self, value: u8) {
        self.ram_bank_and_rtc_select_register = value;

        self.recalculate_ram_target();
    }
    fn recalculate_ram_target(&mut self) {
        self.ram_target = Mbc3RamTarget::derive(
            self.ram_timer_gate_register,
            self.ram_bank_and_rtc_select_register,
            self.ram_bank_mask,
        )
    }
    fn write_rom_bank_number_register(&mut self, mut value: u8) {
        value &= Self::ROM_BANK_NUMBER_REGISTER_MASK;
        if value == 0 {
            value = 1;
        }
        self.upper_rom_bank_number = value as usize & self.rom_bank_mask;
    }
}
impl BankController for MemoryBankController3 {
    fn read(
        &mut self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        self.peek(address, rom_banks, ram_banks, device)
    }

    fn write(
        &mut self,
        address: Address,
        value: u8,
        ram_banks: &mut [RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) {
        if device == CartridgeDevice::ExternalRam {
            match self.ram_target {
                Mbc3RamTarget::RamBank(bank_num) => {
                    let ram_address = address - CartridgeDevice::ExternalRam.get_starting_address();
                    ram_banks.get_mut(bank_num).map(|b| b.write(ram_address, value));
                },
                Mbc3RamTarget::Nothing => (),
                Mbc3RamTarget::RtcReg(reg_num) => {
                    if let Some(rtc) = &mut self.rtc {
                        rtc.write(reg_num, value);
                    }
                },
            }
            return;
        }

        match address {
            0x0000..0x2000 => self.write_ram_timer_gate_register(value),
            0x2000..0x4000 => self.write_rom_bank_number_register(value),
            0x4000..0x6000 => self.write_ram_bank_and_rtc_select_register(value),
            0x6000..0x8000 => {
                if let Some(rtc) = &mut self.rtc {
                    rtc.write_to_latch(value);
                }
            },
            _ => unreachable!("RAM addresses were already handled"),
        }
    }

    fn peek(
        &self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        let in_device_address = address - device.get_starting_address();
        let read_val = match device {
            CartridgeDevice::LowerRomBank | CartridgeDevice::UpperRomBank => rom_banks
                .get(self.get_bank_number(device).unwrap())
                .map(|b| b.read(in_device_address)),
            CartridgeDevice::ExternalRam => match self.ram_target {
                Mbc3RamTarget::RamBank(bank_num) => ram_banks.get(bank_num).map(|b| b.read(in_device_address)),
                Mbc3RamTarget::Nothing => None,
                Mbc3RamTarget::RtcReg(reg_num) => self.rtc.map(|c| c.read(reg_num)),
            },
        };

        read_val.unwrap_or(u8::DEFAULT_BUS_VALUE)
    }

    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize> {
        match device {
            CartridgeDevice::LowerRomBank => Some(0),
            CartridgeDevice::UpperRomBank => Some(self.upper_rom_bank_number),
            CartridgeDevice::ExternalRam => match self.ram_target {
                Mbc3RamTarget::RamBank(bank_num) => Some(bank_num),
                Mbc3RamTarget::Nothing | Mbc3RamTarget::RtcReg(_) => None,
            },
        }
    }

    fn bank_name(&self) -> &'static str {
        match self.rtc.is_some() {
            true => "MBC3+RTC",
            false => "MBC3",
        }
    }

    fn load_save_data(&mut self, data: &mut SaveDataReader) -> Result<(), CartridgeError> {
        match (&mut self.rtc, data.read_rtc()) {
            (None, Some(_)) => Err(CartridgeError::too_much_save_data()),
            (Some(_), None) => Err(CartridgeError::insufficient_save_data()),
            (Some(mbc_rtc), Some(save_rtc)) => Ok(*mbc_rtc = RealTimeClock::try_from(save_rtc)?),
            (None, None) => Ok(()),
        }
    }

    fn append_save_data(&self, save_data: &mut SaveData) {
        if let Some(rtc) = self.rtc {
            save_data.append_rtc(&rtc);
        }
    }
}

#[derive(Debug, Clone)]
enum Mbc3RamTarget {
    RamBank(usize),
    Nothing,
    RtcReg(u8),
}
impl Mbc3RamTarget {
    fn derive(gate_reg: u8, select_reg: u8, mask: Option<usize>) -> Self {
        if gate_reg != RAMG_OPEN {
            return Self::Nothing;
        }
        // RAM Banks
        if select_reg < 0x08 {
            match mask {
                Some(mask) => Self::RamBank(select_reg as usize & mask),
                None => Self::Nothing,
            }
        // RTC Regs
        } else if select_reg < 0x0D {
            Self::RtcReg(select_reg)
        // above 0x0C is undocumented so we're just going with Nothing connected
        } else {
            Self::Nothing
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RealTimeClock {
    seconds: u8,
    minutes: u8,
    hours: u8,
    days_low: u8,
    days_high_and_flags: u8,

    latch_trigger: RtcLatchTrigger,

    crystal_anchor_time: SystemTime,
}
impl RealTimeClock {
    const SECS_IN_DAY: u64 = Self::SECS_IN_HOUR * 24;
    const SECS_IN_HOUR: u64 = Self::SECS_IN_MINUTE * 60;
    const SECS_IN_MINUTE: u64 = 60; // revolutionary math here, I know.

    fn new() -> Self {
        Self {
            seconds: 0,
            minutes: 0,
            hours: 0,
            days_low: 0,
            days_high_and_flags: 0,
            latch_trigger: RtcLatchTrigger::new(),
            crystal_anchor_time: SystemTime::now(),
        }
    }

    fn get_reg(&self, reg_num: u8) -> Option<u8> {
        match reg_num {
            0x08 => Some(self.seconds),
            0x09 => Some(self.minutes),
            0x0A => Some(self.hours),
            0x0B => Some(self.days_low),
            0x0C => Some(self.days_high_and_flags),
            _ => None,
        }
    }
    fn get_reg_mut(&mut self, reg_num: u8) -> Option<&mut u8> {
        match reg_num {
            0x08 => Some(&mut self.seconds),
            0x09 => Some(&mut self.minutes),
            0x0A => Some(&mut self.hours),
            0x0B => Some(&mut self.days_low),
            0x0C => Some(&mut self.days_high_and_flags),
            _ => None,
        }
    }
    fn write(&mut self, reg_num: u8, value: u8) {
        self.get_reg_mut(reg_num).map(|r| *r = value);

        if reg_num == 0x0C && self.is_halted() {
            self.latch_clock(); // if we're halting latch the clock
        } else {
            self.write_regs_back_to_clock();
        }
    }

    fn read(&self, reg_num: u8) -> u8 {
        self.peek(reg_num)
    }

    fn peek(&self, reg_num: u8) -> u8 {
        self.get_reg(reg_num).unwrap_or(u8::DEFAULT_BUS_VALUE)
    }

    fn is_halted(&self) -> bool {
        self.days_high_and_flags & 0b0100_0000 != 0
    }

    fn write_to_latch(&mut self, value: u8) {
        if self.latch_trigger.write(value) && !self.is_halted() {
            self.latch_clock();
        }
    }

    fn latch_clock(&mut self) {
        let duration = SystemTime::now()
            .duration_since(self.crystal_anchor_time)
            .unwrap_or_default();
        let regs = self.convert_duration_to_regs(duration);

        self.seconds = regs[0];
        self.minutes = regs[1];
        self.hours = regs[2];
        self.days_low = regs[3];
        self.days_high_and_flags = regs[4];
    }
    fn write_regs_back_to_clock(&mut self) {
        let mut time_in_secs = self.seconds as u64;
        time_in_secs += self.minutes as u64 * Self::SECS_IN_MINUTE;
        time_in_secs += self.hours as u64 * Self::SECS_IN_HOUR;
        time_in_secs += self.recompose_days() * Self::SECS_IN_DAY;

        self.crystal_anchor_time = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(time_in_secs))
            .unwrap_or(SystemTime::UNIX_EPOCH);
    }

    fn recompose_days(&self) -> u64 {
        self.days_low as u64 | ((self.days_high_and_flags as u64 & 0b1) << 8)
    }

    fn get_latched_regs(&self) -> [u8; 5] {
        [
            self.seconds,
            self.minutes,
            self.hours,
            self.days_low,
            self.days_high_and_flags,
        ]
    }
    fn get_live_regs(&self) -> [u8; 5] {
        if self.is_halted() {
            self.get_latched_regs()
        } else {
            let elapsed = SystemTime::now()
                .duration_since(self.crystal_anchor_time)
                .unwrap_or_default();
            self.convert_duration_to_regs(elapsed)
        }
    }

    fn from_rtc_file(data: &[u8; 8]) -> Self {
        let anchor_time = SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(u64::from_le_bytes(*data)));

        let mut this = Self::new();

        this.crystal_anchor_time = anchor_time.unwrap_or(SystemTime::now());
        this.latch_clock();

        this
    }

    /// Converts regs (assuming the order on the RTC) into a Duration
    fn convert_regs_to_duration(regs: [u8; 5]) -> Duration {
        let live_days = regs[3] as u64 | ((regs[4] as u64 & 0b1) << 8);
        let live_secs = regs[0] as u64
            + (regs[1] as u64 * Self::SECS_IN_MINUTE)
            + (regs[2] as u64 * Self::SECS_IN_HOUR)
            + (live_days * Self::SECS_IN_DAY);

        Duration::from_secs(live_secs)
    }

    fn convert_duration_to_regs(&self, elapsed: Duration) -> [u8; 5] {
        let mut secs = elapsed.as_secs();

        let days = secs / Self::SECS_IN_DAY;
        secs %= Self::SECS_IN_DAY;
        let hours = secs / Self::SECS_IN_HOUR;
        secs %= Self::SECS_IN_HOUR;
        let minutes = secs / Self::SECS_IN_MINUTE;
        let seconds = secs % Self::SECS_IN_MINUTE;

        let mut day_high_and_flags = self.days_high_and_flags & 0b1111_1110;
        if days % 512 > 255 {
            day_high_and_flags |= 0b0000_0001; // 9th bit
        }
        if days > 511 {
            day_high_and_flags |= 0b1000_0000; // overflow bit
        }

        [
            seconds as u8,
            minutes as u8,
            hours as u8,
            days as u8,
            day_high_and_flags,
        ]
    }

    fn anchor_from_now(elapsed: Duration) -> SystemTime {
        SystemTime::now().checked_sub(elapsed).unwrap_or(SystemTime::UNIX_EPOCH)
    }
    /// `.sav` layout is 5 live registers as u32::le, 5 latched registers as u32::le, and an 8 byte Unix TimeStamp
    fn from_sav_file(data: &[u8; 48]) -> Self {
        let mut this = Self::new();

        // convert live and latched registers back to u8s
        let registers: Vec<u8> = data[0..40]
            .chunks(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()) as u8)
            .collect();

        // latched regs are the second group
        this.seconds = registers[5];
        this.minutes = registers[6];
        this.hours = registers[7];
        this.days_low = registers[8];
        this.days_high_and_flags = registers[9];

        let live = Self::convert_regs_to_duration(registers[0..5].try_into().unwrap());
        let timestamp = u64::from_le_bytes(data[40..48].try_into().unwrap());
        this.crystal_anchor_time = if this.is_halted() {
            Self::anchor_from_now(live)
        } else {
            Duration::from_secs(timestamp)
                .checked_sub(live)
                .and_then(|d| SystemTime::UNIX_EPOCH.checked_add(d))
                .unwrap_or_else(|| Self::anchor_from_now(live))
        };
        this
    }

    pub fn as_rtc_file_timestamp(&self) -> [u8; 8] {
        self.crystal_anchor_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_le_bytes()
            .into()
    }

    pub fn as_sav_file_data(&self) -> [u8; 48] {
        let mut data = [0; 48];

        let live = self.get_live_regs();
        let latched = self.get_latched_regs();
        for (i, reg) in live.iter().chain(latched.iter()).enumerate() {
            data[i * 4] = *reg; // "u32 le" by just putting the u8 in the leftmost byte
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        data[40..48].copy_from_slice(&timestamp.to_le_bytes());

        data
    }
}

impl TryFrom<&[u8]> for RealTimeClock {
    type Error = CartridgeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.len() {
            8 => Ok(Self::from_rtc_file(value.try_into().unwrap())),
            48 => Ok(Self::from_sav_file(value.try_into().unwrap())),
            _ => Err(CartridgeError::MisMatchedRamSaveSize(
                "RTC supports from 8 bytes (just a timestamp) or from 48 (regs, live regs, timestamp)".into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RtcLatchTrigger {
    in_mid_state: bool,
}
impl RtcLatchTrigger {
    fn new() -> Self {
        Self { in_mid_state: false }
    }
    /// alters latch trigger based on value.
    /// Returns whether or not the sequence is completed and we should latch.
    fn write(&mut self, value: u8) -> bool {
        if value == 0 {
            self.in_mid_state = true;
            false
        } else if value == 1 && self.in_mid_state {
            self.in_mid_state = false;
            true
        } else {
            self.in_mid_state = false;
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBankController5 {
    ram_gate_register: u8,
    combined_romb: u16, // romb0 and romb1 concatenated
    ramb: u8,

    ram_bank_num: Option<usize>,
    upper_rom_bank_num: usize,

    rom_bank_mask: usize,
    ram_bank_mask: Option<usize>,
}
impl MemoryBankController5 {
    pub fn new(num_rom_banks: usize, num_ram_banks: usize) -> Self {
        debug_assert!(
            num_rom_banks.is_power_of_two(),
            "bank masking assumes a power-of-two bank count"
        );
        debug_assert!(num_ram_banks == 0 || num_ram_banks.is_power_of_two());
        Self {
            ram_gate_register: 0,
            combined_romb: 0,
            ramb: 0,

            ram_bank_num: None,
            upper_rom_bank_num: 1,

            rom_bank_mask: num_rom_banks - 1,
            ram_bank_mask: (num_ram_banks > 0).then(|| num_ram_banks - 1),
        }
    }

    fn write_ram_gate_register(&mut self, value: u8) {
        self.ram_gate_register = value;
        self.recalculate_ram_bank_number();
    }
    fn write_romb0(&mut self, value: u8) {
        self.combined_romb &= 0x0100; // zero out lower 8 bits
        self.combined_romb |= value as u16;
        self.recalculate_rom_bank_number();
    }
    fn write_romb1(&mut self, value: u8) {
        self.combined_romb &= 0x00FF; // zero out top bit
        self.combined_romb |= ((value as u16) & 0b1) << 8;
        self.recalculate_rom_bank_number();
    }
    fn write_ramb(&mut self, value: u8) {
        self.ramb = value & 0x0F;
        self.recalculate_ram_bank_number();
    }

    fn recalculate_ram_bank_number(&mut self) {
        self.ram_bank_num = match self.ram_bank_mask {
            Some(mask) if self.ram_gate_register == RAMG_OPEN => Some(self.ramb as usize & mask),
            _ => None,
        };
    }
    fn recalculate_rom_bank_number(&mut self) {
        self.upper_rom_bank_num = self.combined_romb as usize & self.rom_bank_mask;
    }
}
impl BankController for MemoryBankController5 {
    fn read(
        &mut self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        self.peek(address, rom_banks, ram_banks, device)
    }

    fn write(
        &mut self,
        address: Address,
        value: u8,
        ram_banks: &mut [RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) {
        if device == CartridgeDevice::ExternalRam {
            let ram_address = address - CartridgeDevice::ExternalRam.get_starting_address();
            let Some(ram_bank) = self.ram_bank_num.and_then(|i| ram_banks.get_mut(i)) else {
                return;
            };
            ram_bank.write(ram_address, value);
            return;
        }

        match address {
            0x0000..0x2000 => self.write_ram_gate_register(value),
            0x2000..0x3000 => self.write_romb0(value),
            0x3000..0x4000 => self.write_romb1(value),
            0x4000..0x6000 => self.write_ramb(value),
            0x6000..0x8000 => (),
            _ => unreachable!("RAM addresses were already handled"),
        }
    }

    fn peek(
        &self,
        address: Address,
        rom_banks: &[RomBank],
        ram_banks: &[RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) -> u8 {
        let in_device_address = address - device.get_starting_address();
        let bank_number = self.get_bank_number(device);

        let read_val = match device {
            CartridgeDevice::LowerRomBank | CartridgeDevice::UpperRomBank => {
                rom_banks.get(bank_number.unwrap()).map(|b| b.peek(in_device_address))
            },
            CartridgeDevice::ExternalRam => bank_number
                .and_then(|bank_num| ram_banks.get(bank_num))
                .map(|b| b.peek(in_device_address)),
        };

        read_val.unwrap_or(u8::DEFAULT_BUS_VALUE)
    }

    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize> {
        match device {
            CartridgeDevice::LowerRomBank => Some(0),
            CartridgeDevice::UpperRomBank => Some(self.upper_rom_bank_num),
            CartridgeDevice::ExternalRam => self.ram_bank_num,
        }
    }

    fn bank_name(&self) -> &'static str {
        "MBC5"
    }
}
