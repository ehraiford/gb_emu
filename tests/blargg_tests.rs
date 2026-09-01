#![cfg(feature = "headless")]

///! This is designed to use the blargg test suite.
/// If you clone it (https://github.com/retrio/gb-test-roms.git) in /test_roms, the all of the paths should line up.
/// It's not been updated in 11 years so we should be fine to rely on their directory structures.
use gb_emu::{cartridge::cartridge::Cartridge, game_boy::GameBoy, io_devices::serial::turn_output_to_string};
use std::{
    fs::{self, ReadDir},
    path::PathBuf,
};

const MAX_RUN_CYCLES: u64 = 0x2_000_000;

fn get_test_dir_pathbuf() -> PathBuf {
    let mut path = PathBuf::from(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    path.push(PathBuf::from("test_roms"));
    path.push(PathBuf::from("gb-test-roms"));
    path
}

/// Most of the tests are comprised of little tests.
/// This gets the little tests.
/// Some dirs are named "individual" while others are "rom_singles" so we need to check for both
fn get_individual_test_dir(path_name: &'static str) -> Option<ReadDir> {
    let mut individual_test_dir = get_test_dir_pathbuf();
    individual_test_dir.push(path_name); // dir
    individual_test_dir.push("individual");
    if let Ok(entries) = fs::read_dir(individual_test_dir) {
        return Some(entries);
    } else {
        let mut individual_test_dir = get_test_dir_pathbuf();
        individual_test_dir.push(path_name); // dir
        individual_test_dir.push("rom_singles");
        fs::read_dir(individual_test_dir).ok()
    }
}

fn run_blargg_test_group(path_name: &'static str) {
    // run group test first
    let mut path = get_test_dir_pathbuf();
    path.push(path_name); // dir
    path.push(path_name); // filename
    path.set_extension("gb");
    let cartridge = Cartridge::new(&fs::read(path).unwrap()).unwrap();
    if run_blargg_test(cartridge).is_ok() {
        return;
    }

    // if that fails, run each test individually and report which ones failed
    let Some(entries) = get_individual_test_dir(path_name) else {
        panic!("Test failed. There are no individual tests to figure out where.")
    };
    for entry in entries {
        let individual_test_path = entry.unwrap().path();
        let cartridge = Cartridge::new(&fs::read(&individual_test_path).unwrap()).unwrap();
        if let Err(failure) = run_blargg_test(cartridge) {
            println!(
                "{path_name}::{} failed: {failure}",
                individual_test_path.file_name().unwrap().to_string_lossy()
            );
        }
    }
    panic!();
}

fn run_blargg_test(cartridge: Cartridge) -> Result<(), String> {
    let mut game_boy = GameBoy::new();
    game_boy.load_cartridge(cartridge);

    // Rebuilding the serial string allocates; only do it on ticks that actually emitted a new bit
    // (one bit per 128 M-cycles) rather than on every tick.
    let mut last_bit_count = game_boy.serial_output_bit_count();

    for _ in 0..MAX_RUN_CYCLES {
        game_boy.tick();

        let bit_count = game_boy.serial_output_bit_count();
        if bit_count == last_bit_count {
            continue;
        }
        last_bit_count = bit_count;

        let output = turn_output_to_string(game_boy.get_serial_output());
        if output.ends_with("Passed") {
            return Ok(());
        } else if output.ends_with("Failed") {
            return Err("Failed Test".to_string());
        }
    }
    Err("Did not pass test within time limit".to_string())
}


// Blargg tests' readme's "Output to memory" section expects the emulator to consume the text as it appears.
const TEXT_OUT_BASE: u16 = 0xA004;
const TEXT_OUT_PTR: u16 = 0xD883; // text_out_addr in bss; fixed by this suite's linkfile

fn drain_text_out(game_boy: &mut GameBoy) {
    let end = game_boy.peek_mem(TEXT_OUT_PTR) as u16 | ((game_boy.peek_mem(TEXT_OUT_PTR + 1) as u16) << 8);
    if !(TEXT_OUT_BASE..0xC000).contains(&end) {
        return; // not a text_out pointer - ROM hasn't initialized it, or the layout moved
    }
    game_boy.write_mem_debug(TEXT_OUT_PTR, TEXT_OUT_BASE as u8);
    game_boy.write_mem_debug(TEXT_OUT_PTR + 1, (TEXT_OUT_BASE >> 8) as u8);
    game_boy.write_mem_debug(TEXT_OUT_BASE, 0);
}

// Some tests write pass/fail to external RAM (0xA000), not serial.
// 0x80 = still running, 0x00 = passed, other = fail code.
// Waits for the 0x80 sentinel before checking completion so the initial
// zero-initialized RAM doesn't cause a false pass.
fn run_ram_result_test(cartridge: Cartridge) -> Result<(), String> {
    const MAX_CYCLES: u64 = 0x400_0000;
    let mut game_boy = GameBoy::new();
    game_boy.load_cartridge(cartridge);

    let mut started = false;
    for _ in 0..MAX_CYCLES {
        game_boy.tick();
        let status = game_boy.peek_mem(0xA000);
        if !started {
            started = status == 0x80;
            continue;
        }
        drain_text_out(&mut game_boy);
        match status {
            0x80 => continue,
            0x00 => return Ok(()),
            code => return Err(format!("Test failed with code {:#04x}", code)),
        }
    }
    Err("Did not pass test within time limit".to_string())
}

fn run_oam_bug_test(cartridge: Cartridge) {
    if let Err(failure) = run_ram_result_test(cartridge) {
        panic!("{failure}");
    }
}

#[test]
fn test_cpu_instrs() {
    // The combined cpu_instrs.gb runs all 11 sub-tests sequentially and exceeds MAX_RUN_CYCLES.
    // Run the individual tests directly instead.
    let Some(entries) = get_individual_test_dir("cpu_instrs") else {
        panic!("No individual cpu_instrs tests found");
    };
    let mut any_failed = false;
    for entry in entries {
        let path = entry.unwrap().path();
        let cartridge = Cartridge::new(&fs::read(&path).unwrap()).unwrap();
        if let Err(failure) = run_blargg_test(cartridge) {
            any_failed = true;
            println!(
                "cpu_instrs::{} failed: {failure}",
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }
    if any_failed {
        panic!();
    }
}

#[test]
fn test_cgb_sound() {
    run_blargg_test_group("dmg_sound");
}

#[test]
fn test_instr_timing() {
    run_blargg_test_group("instr_timing");
}

#[test]
#[ignore = "CGB test relying on double speed mode"]
fn test_interrupt_timing() {
    run_blargg_test_group("interrupt_time");
}

#[test]
fn test_mem_timing() {
    run_blargg_test_group("mem_timing");
}

#[test]
fn test_mem_timing2() {
    // mem_timing-2 names its combined ROM "mem_timing.gb" not "mem_timing-2.gb",
    // and outputs results to $A000 instead of serial.
    let mut path = get_test_dir_pathbuf();
    path.push("mem_timing-2");
    path.push("mem_timing");
    path.set_extension("gb");

    let cartridge = Cartridge::new(&fs::read(path).unwrap()).unwrap();
    if run_ram_result_test(cartridge).is_ok() {
        return;
    }

    let Some(entries) = get_individual_test_dir("mem_timing-2") else {
        panic!("Test failed. There are no individual tests to figure out where.")
    };
    let mut any_failed = false;
    for entry in entries {
        let individual_test_path = entry.unwrap().path();
        let cartridge = Cartridge::new(&fs::read(&individual_test_path).unwrap()).unwrap();
        if let Err(failure) = run_ram_result_test(cartridge) {
            any_failed = true;
            println!(
                "mem_timing-2::{} failed: {failure}",
                individual_test_path.file_name().unwrap().to_string_lossy()
            );
        } else {
            println!(
                "mem_timing-2::{} passed",
                individual_test_path.file_name().unwrap().to_string_lossy()
            );
        }
    }
    if any_failed {
        panic!();
    }
}

#[test]
#[ignore = "Multi-ROM never reports a result; test_oam_incremental covers the same ROMs"]
fn test_oam_bug() {
    // Every sub-test's main ends in `exit`, which beeps the result out through play_byte, so the
    // combined ROM spends most of its time in delay loops. It gets through all 186 sub-tests with
    // each reporting "ok", then restarts its counter instead of reaching post_exit, which is the
    // only thing that writes the result to $A000. Nothing here is OAM-specific.
    let mut path = get_test_dir_pathbuf();
    path.push("oam_bug/oam_bug.gb");
    let cartridge = Cartridge::new(&fs::read(path).unwrap()).unwrap();
    run_oam_bug_test(cartridge);
}

#[test]
fn test_oam_incremental() {
    let tests = [
        "1-lcd_sync.gb",
        "2-causes.gb",
        "3-non_causes.gb",
        "4-scanline_timing.gb",
        "5-timing_bug.gb",
        "6-timing_no_bug.gb",
        "7-timing_effect.gb",
        "8-instr_effect.gb",
    ];

    let mut failed = false;
    for test_name in &tests {
        let mut path = get_test_dir_pathbuf();
        path.push("oam_bug/rom_singles");
        path.push(test_name);

        let rom = fs::read(&path).expect(&format!("Failed to read {}", test_name));
        let cartridge = Cartridge::new(&rom).expect(&format!("Failed to load cartridge {}", test_name));
        if let Err(failure) = run_ram_result_test(cartridge) {
            println!("{test_name} failed: {failure}");
            failed = true;
        }
    }
    assert!(!failed)
}
