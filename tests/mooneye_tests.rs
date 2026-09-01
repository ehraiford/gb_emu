#![cfg(feature = "headless")]

//! Gekkio's Mooneye Test Suite.
//!
//! Clone/extract the prebuilt ROMs into /test_roms/mooneye-test-suite so the paths line up:
//!   https://gekkio.fi/files/mooneye-test-suite/
//!
//! A test signals its result by loading B/C/D/E/H/L with the Fibonacci numbers 3/5/8/13/21/34 on
//! success, or 0x42 in every register on failure, then executing `LD B,B` (opcode 0x40) as a magic
//! breakpoint followed by an infinite loop. The suite also mirrors the same bytes over the serial
//! port, but the registers are the primary signal and the one this harness reads.

use gb_emu::{cartridge::cartridge::Cartridge, game_boy::GameBoy};
use std::{fs, path::{Path, PathBuf}};

const PASSED: [u8; 6] = [3, 5, 8, 13, 21, 34];
const FAILED: [u8; 6] = [0x42; 6];
/// `LD B,B ; JR -2` — the magic breakpoint plus the infinite loop that follows it.
const TERMINATOR: [u8; 3] = [0x40, 0x18, 0xFE];
const MAX_CYCLES: u64 = 0x200_0000;

/// Model suffixes the DMG core is not expected to satisfy. A ROM with no suffix applies to every
/// model; the letters after the final dash list the models the test is written for (G = DMG,
/// S = SGB, C = CGB, A = AGB), so anything mentioning DMG is in scope.
fn is_for_dmg(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    match stem.rsplit_once('-') {
        None => true,
        Some((_, suffix)) => suffix.contains('G') || suffix.contains("dmg"),
    }
}

/// True once execution has reached the end-of-test sequence. `get_pc` is the CPU's fetch pointer
/// rather than the address of the instruction being executed, so it can sit on any byte of the
/// three; matching the whole window avoids both a spurious hit on a lone 0x40 operand byte and a
/// miss once the CPU has settled into the JR loop.
fn at_terminator(game_boy: &GameBoy) -> bool {
    let pc = game_boy.get_pc();
    (0..TERMINATOR.len() as u16).any(|offset| {
        let base = pc.wrapping_sub(offset);
        TERMINATOR
            .iter()
            .enumerate()
            .all(|(i, byte)| game_boy.peek_mem(base.wrapping_add(i as u16)) == *byte)
    })
}

fn run_mooneye_test(cartridge: Cartridge) -> Result<(), String> {
    let mut game_boy = GameBoy::new();
    game_boy.load_cartridge(cartridge);

    for _ in 0..MAX_CYCLES {
        game_boy.tick();
        if !at_terminator(&game_boy) {
            continue;
        }
        let registers = game_boy.debug_registers();
        return match registers {
            PASSED => Ok(()),
            FAILED => Err("Test reported failure (0x42 in all registers)".to_string()),
            other => Err(format!("Stopped at LD B,B with unexpected registers: {other:02X?}")),
        };
    }
    Err("Did not reach the LD B,B breakpoint within the cycle limit".to_string())
}

fn mooneye_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_roms");
    path.push("mooneye-test-suite");
    path
}

/// `acceptance/` nests ROMs a few directories deep, so gather them recursively.
fn collect_roms(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_roms(&path, found);
        } else if path.extension().is_some_and(|e| e == "gb") && is_for_dmg(&path) {
            found.push(path);
        }
    }
}

fn run_group(group: &str) {
    let root = mooneye_dir();
    let mut roms = Vec::new();
    collect_roms(&root.join(group), &mut roms);
    assert!(!roms.is_empty(), "No ROMs found under {}", root.join(group).display());
    roms.sort();

    let mut failures = Vec::new();
    for path in &roms {
        let name = path.strip_prefix(&root).unwrap_or(path).display().to_string();
        let cartridge = match Cartridge::new(&fs::read(path).unwrap()) {
            Ok(cartridge) => cartridge,
            Err(_) => {
                failures.push(format!("{name}: unsupported cartridge"));
                continue;
            },
        };
        if let Err(failure) = run_mooneye_test(cartridge) {
            failures.push(format!("{name}: {failure}"));
        }
    }

    println!("{group}: {}/{} passed", roms.len() - failures.len(), roms.len());
    for failure in &failures {
        println!("  {failure}");
    }
    assert!(failures.is_empty(), "{} of {} mooneye {group} tests failed", failures.len(), roms.len());
}

// Mooneye is far stricter than blargg and most of it is still red, so these are opt-in for now:
// run them with `cargo test --features headless --test mooneye_tests -- --ignored --nocapture`.
// Drop the attribute once a group goes green so it can hold the line.
#[test]
#[ignore = "14/69 passing - opt-in scoreboard, not a regression gate yet"]
fn test_mooneye_acceptance() {
    run_group("acceptance");
}

#[test]
#[ignore = "12/28 passing - all 15 remaining are MBC2/MBC5 ROMs, plus MBC1M multicart"]
fn test_mooneye_emulator_only() {
    run_group("emulator-only");
}
