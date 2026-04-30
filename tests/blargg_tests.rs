#![cfg(feature = "headless")]

///! This is designed to use the blargg test suite.
/// If you clone it (https://github.com/retrio/gb-test-roms.git) in /test_roms, the all of the paths should line up.
/// It's not been updated in 11 years so we should be fine to rely on their directory structures.
use gb_emu::{cartridge::cartridge::Cartridge, game_boy::GameBoy, io_devices::serial::turn_output_to_string};
use std::{
    fs::{self, ReadDir},
    path::PathBuf,
};

const MAX_RUN_CYCLES: u64 = 0x1_000_000;

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
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.ends_with("Passed") {
                return Ok(());
            } else if new_output.ends_with("Failed") {
                return Err("Failed Test".to_string());
            }
            prev_output = new_output;
        }
    }
    Err("Did not pass test within time limit".to_string())
}

#[test]
fn test_cpu_instrs() {
    run_blargg_test_group("cpu_instrs");
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
fn test_interrupt_timing() {
    run_blargg_test_group("interrupt_timing");
}

#[test]
fn test_mem_timing() {
    run_blargg_test_group("mem_timing");
}

#[test]
#[ignore]
fn test_mem_timing2() {
    // this one doesn't follow the naming scheme of the rest so we'll need to manually handle it
    run_blargg_test_group("mem_timing-2");
}

#[test]
fn test_oam_bug() {
    run_blargg_test_group("oam_bug");
}
