use gb_emu::cartridge::cartridge::{Cartridge, CartridgeDevice};

/// 1MB MBC3+RAM+BATTERY image; every bank is filled with its own bank number.
fn synth_rom(cart_type: u8, rom_size_code: u8, ram_size_code: u8) -> Vec<u8> {
    let num_banks = 2usize << rom_size_code;
    let mut rom = vec![0u8; num_banks * 16 * 1024];
    for bank in 0..num_banks {
        let base = bank * 16 * 1024;
        rom[base..base + 16 * 1024].fill(bank as u8);
    }
    rom[0x134..0x13F].copy_from_slice(b"MBC3SMOKE\0\0");
    rom[0x147] = cart_type;
    rom[0x148] = rom_size_code;
    rom[0x149] = ram_size_code;
    rom
}

#[test]
fn mbc3_loads_and_banks() {
    // 0x13 = MBC3+RAM+BAT, 1MB (64 banks), 32KB RAM (4 banks) -- Pokemon Red/Blue's shape.
    let rom = synth_rom(0x13, 0x05, 0x03);
    let mut cart = Cartridge::new(&rom).expect("MBC3 cart should load");

    assert_eq!(cart.read(0x0000, CartridgeDevice::LowerRomBank), 0, "lower is bank 0");
    assert_eq!(cart.read(0x4000, CartridgeDevice::UpperRomBank), 1, "upper defaults to bank 1");

    for bank in [2u8, 31, 63] {
        cart.write(0x2000, bank, CartridgeDevice::LowerRomBank);
        assert_eq!(cart.read(0x4000, CartridgeDevice::UpperRomBank), bank, "bank {bank}");
    }

    // Bank 0 in the upper region translates to 1, same as MBC1.
    cart.write(0x2000, 0, CartridgeDevice::LowerRomBank);
    assert_eq!(cart.read(0x4000, CartridgeDevice::UpperRomBank), 1, "0 -> 1");

    // RAM is gated shut until 0x0A is written.
    assert_eq!(cart.read(0xA000, CartridgeDevice::ExternalRam), 0xFF, "gate closed");
    cart.write(0x0000, 0x0A, CartridgeDevice::LowerRomBank);
    for bank in 0..4u8 {
        cart.write(0x4000, bank, CartridgeDevice::LowerRomBank);
        cart.write(0xA000, 0x40 + bank, CartridgeDevice::ExternalRam);
    }
    for bank in 0..4u8 {
        cart.write(0x4000, bank, CartridgeDevice::LowerRomBank);
        assert_eq!(cart.read(0xA000, CartridgeDevice::ExternalRam), 0x40 + bank, "ram bank {bank}");
    }
}

#[test]
fn mbc3_rtc_latches() {
    // 0x10 = MBC3+TIMER+RAM+BAT -- Gold/Silver's shape, 2MB.
    let rom = synth_rom(0x10, 0x06, 0x03);
    let mut cart = Cartridge::new(&rom).expect("MBC3+TIMER cart should load");

    cart.write(0x0000, 0x0A, CartridgeDevice::LowerRomBank); // open the RAM/timer gate
    cart.write(0x4000, 0x08, CartridgeDevice::LowerRomBank); // select RTC seconds

    cart.write(0xA000, 30, CartridgeDevice::ExternalRam); // set the clock
    cart.write(0x6000, 0x00, CartridgeDevice::LowerRomBank);
    cart.write(0x6000, 0x01, CartridgeDevice::LowerRomBank); // latch
    assert_eq!(cart.read(0xA000, CartridgeDevice::ExternalRam), 30, "seconds round-trip");

    // 0x0D is outside the RTC range and selects nothing.
    cart.write(0x4000, 0x0D, CartridgeDevice::LowerRomBank);
    assert_eq!(cart.read(0xA000, CartridgeDevice::ExternalRam), 0xFF, "0x0D -> open bus");
}

#[test]
fn mbc3_timer_without_ram_loads() {
    // 0x0F = MBC3+TIMER+BATTERY: a timer but no RAM at all.
    let rom = synth_rom(0x0F, 0x05, 0x00);
    let mut cart = Cartridge::new(&rom).expect("no-RAM MBC3 should load");
    cart.write(0x0000, 0x0A, CartridgeDevice::LowerRomBank);
    cart.write(0x4000, 0x00, CartridgeDevice::LowerRomBank);
    assert_eq!(cart.read(0xA000, CartridgeDevice::ExternalRam), 0xFF, "no RAM -> open bus");
}
