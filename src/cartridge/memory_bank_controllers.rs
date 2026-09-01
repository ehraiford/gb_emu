use std::time::SystemTime;

use crate::cartridge::cartridge::{CartridgeDevice, RAM_BANK_SIZE};
use crate::{
    bus::{Address, BusDefault},
    onboard_memory::rom_and_ram::{RamBank, RomBank},
};

#[derive(Debug, Clone)]
pub enum MemoryBankController {
    MBC1(MemoryBankController1),
    MBC2(MemoryBankController2),
    MBC3(MemoryBankController3),
    MBC5,
    MBC6,
    MBC7,
}

impl MemoryBankController {
    /// Which controllers actually have their behaviour wired up. The rest are still parsed out of
    /// the header so a caller can report them by name, but every accessor on them is `todo!()`, so
    /// loading one has to be refused up front rather than panicking on the first ROM read.
    pub fn is_implemented(&self) -> bool {
        matches!(self, Self::MBC1(_) | Self::MBC2(_) | Self::MBC3(_))
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::MBC1(_) => "MBC1",
            Self::MBC2(_) => "MBC2",
            Self::MBC3(_) => "MBC3",
            Self::MBC5 => "MBC5",
            Self::MBC6 => "MBC6",
            Self::MBC7 => "MBC7",
        }
    }

    pub fn write(
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
            MemoryBankController::MBC5 => todo!(),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    pub fn read(
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
            MemoryBankController::MBC5 => todo!(),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }

    pub fn peek(
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
            MemoryBankController::MBC5 => todo!(),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
        }
    }
}

pub(crate) trait BankController {
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
    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize>;
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
    const RAMG_OPEN: u8 = 0b0000_1010;
    const RAMG_REGISTER_MASK: u8 = 0x0F;
    const BANK_REGISTER_1_MASK: u8 = 0b0001_1111;
    const BANK_REGISTER_2_MASK: u8 = 0b0000_0011;
    const BANK_2_MODE_REGISTER_MASK: u8 = 0b0000_0001;

    pub fn new(num_rom_banks: usize, num_ram_banks: usize) -> Self {
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
        self.ram_bank_number = if self.ram_gate_register != Self::RAMG_OPEN || self.ram_bank_mask.is_none() {
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
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryBankController2 {
    // MBC2 is simple enough that we're not gonna bother storing the actual registers
    rom_bank_mask: usize,

    upper_rom_bank_number: usize,
    ram_bank_number: Option<usize>,
}

impl MemoryBankController2 {
    const RAMG_OPEN: u8 = 0b0000_1010;
    const RAMG_REGISTER_MASK: u8 = 0x0F;
    const RAM_VALUE_MASK: u8 = 0x0F; // MBC2 RAM only stores in the right nibble
    const RAM_ADDRESS_MASK: Address = 0x01FF; // MBC2 RAM addresses wrap in the 10th bit
    const ROMB_REGISTER_MASK: u8 = 0x0F;

    pub fn new(num_rom_banks: usize) -> Self {
        Self {
            rom_bank_mask: num_rom_banks - 1,
            upper_rom_bank_number: 1,
            ram_bank_number: None,
        }
    }

    fn is_addressing_ramg_not_romb(address: Address) -> bool {
        (address & 0b0000_0001_0000_0000) == 0
    }

    fn write_ram_gate_register(&mut self, value: u8) {
        match value & Self::RAMG_REGISTER_MASK == Self::RAMG_OPEN {
            true => self.ram_bank_number = Some(0),
            false => self.ram_bank_number = None,
        }
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
        value & Self::RAM_VALUE_MASK
    }
}

impl BankController for MemoryBankController2 {
    fn write(
        &mut self,
        address: Address,
        value: u8,
        ram_banks: &mut [RamBank<RAM_BANK_SIZE>],
        device: CartridgeDevice,
    ) {
        match device {
            CartridgeDevice::UpperRomBank => return,
            CartridgeDevice::LowerRomBank => match Self::is_addressing_ramg_not_romb(address) {
                true => self.write_ram_gate_register(value),
                false => self.write_rom_bank_register(value),
            },
            CartridgeDevice::ExternalRam => {
                let Some(ram_bank) = self.ram_bank_number.and_then(|i| ram_banks.get_mut(i)) else {
                    return;
                };
                let ram_value = Self::convert_ram_value(value);
                let ram_address = Self::get_ram_address(address);
                ram_bank.write(ram_address, ram_value);
            },
        }
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
        let bank_number = self.get_bank_number(device);

        let read_val = match device {
            CartridgeDevice::LowerRomBank | CartridgeDevice::UpperRomBank => {
                let in_device_address = address - device.get_starting_address();
                rom_banks.get(bank_number.unwrap()).map(|b| b.read(in_device_address))
            },
            CartridgeDevice::ExternalRam => {
                let ram_address = Self::get_ram_address(address);
                bank_number
                    .and_then(|i| ram_banks.get(i))
                    .map(|b| b.read(ram_address) | !Self::RAM_VALUE_MASK)
            },
        };

        read_val.unwrap_or(u8::DEFAULT_BUS_VALUE)
    }

    fn get_bank_number(&self, device: CartridgeDevice) -> Option<usize> {
        match device {
            CartridgeDevice::LowerRomBank => Some(0),
            CartridgeDevice::UpperRomBank => Some(self.upper_rom_bank_number),
            CartridgeDevice::ExternalRam => self.ram_bank_number,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBankController3 {
    rtc: Option<RealTimeClock>,
    ram_timer_gate_register: u8,
    ram_bank_and_rtc_select_register: u8,

    upper_rom_bank_number: usize,
    ram_target: Mbc3RamTarget,

    ram_bank_mask: usize,
    rom_bank_mask: usize,
}

impl MemoryBankController3 {
    pub fn new(num_rom_banks: usize, num_ram_banks: usize, has_rtc: bool) -> Self {
        let rtc = has_rtc.then(|| RealTimeClock::new());
        Self {
            rtc,
            ram_timer_gate_register: 0,
            ram_bank_and_rtc_select_register: 0,
            upper_rom_bank_number: 1,
            ram_target: Mbc3RamTarget::Nothing,
            ram_bank_mask: num_ram_banks.saturating_sub(1), // this being zero is already handled in read/write/peek
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
}

#[derive(Debug, Clone)]
enum Mbc3RamTarget {
    RamBank(usize),
    Nothing,
    RtcReg(u8),
}
impl Mbc3RamTarget {
    const RAMG_OPEN: u8 = 0b0000_1010;
    fn derive(gate_reg: u8, select_reg: u8, mask: usize) -> Self {
        if gate_reg != Self::RAMG_OPEN {
            return Self::Nothing;
        }
        // RAM Banks
        if select_reg < 0x08 {
            Self::RamBank(select_reg as usize & mask)
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
struct RealTimeClock {
    seconds: u8,
    minutes: u8,
    hours: u8,
    days_low: u8,
    day_high_and_flags: u8,

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
            day_high_and_flags: 0,
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
            0x0C => Some(self.day_high_and_flags),
            _ => None,
        }
    }
    fn get_reg_mut(&mut self, reg_num: u8) -> Option<&mut u8> {
        match reg_num {
            0x08 => Some(&mut self.seconds),
            0x09 => Some(&mut self.minutes),
            0x0A => Some(&mut self.hours),
            0x0B => Some(&mut self.days_low),
            0x0C => Some(&mut self.day_high_and_flags),
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
        self.day_high_and_flags & 0b0100_0000 != 0
    }

    fn write_to_latch(&mut self, value: u8) {
        if self.latch_trigger.write(value) && !self.is_halted() {
            self.latch_clock();
        }
    }

    fn latch_clock(&mut self) {
        let mut secs_since_anchor = SystemTime::now()
            .duration_since(self.crystal_anchor_time)
            .unwrap_or_default()
            .as_secs();

        let days = secs_since_anchor / Self::SECS_IN_DAY;
        secs_since_anchor %= Self::SECS_IN_DAY;

        let hours = secs_since_anchor / Self::SECS_IN_HOUR;
        secs_since_anchor %= Self::SECS_IN_HOUR;

        let minutes = secs_since_anchor / Self::SECS_IN_MINUTE;
        let secs = secs_since_anchor % Self::SECS_IN_MINUTE;

        self.seconds = secs as u8;
        self.minutes = minutes as u8;
        self.hours = hours as u8;
        self.days_low = days as u8; // just gives us the LSB anyway

        self.day_high_and_flags &= 0b1111_1110; // mask out 9th bit of Day
        if days % 512 > 255 {
            self.day_high_and_flags |= 0b0000_0001; // 9th bit
        }
        if days > 511 {
            self.day_high_and_flags |= 0b1000_0000; // overflow bit
        }
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
        self.days_low as u64 | ((self.day_high_and_flags as u64 & 0b1) << 8)
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
