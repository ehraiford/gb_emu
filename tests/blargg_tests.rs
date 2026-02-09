#![cfg(feature = "headless")]

use std::{fs, path::PathBuf};

use gb_emu::{cartridge::cartridge::Cartridge, game_boy::GameBoy, io_devices::serial::turn_output_to_string};

enum BlarggTest {
    InstrTiming,
    MemTiming,
    InterruptTime,
    DmgSound,
    HaltBug,
    OamBug,
    CgbSound,
    Special,
    Interrupts,
    OpSpHl,
    OpRImm,
    OpRp,
    LdRR,
    JrJpCallRetRst,
    MiscInstrs,
    OpRR,
    BitOps,
    OpAHl,
}

impl BlarggTest {
    const MAX_RUN_CYCLES: u64 = 0x1_000_000;

    fn get_rom_path(&self) -> PathBuf {
        let mut path = PathBuf::from(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        path.push(PathBuf::from("test_roms"));
        path.push(PathBuf::from("blargg_test_roms"));

        let end_path = &match self {
            Self::CgbSound => "cgb_sound.gb",
            Self::DmgSound => "dmg_sound.gb",
            Self::HaltBug => "halt_bug.gb",
            Self::InstrTiming => "instr_timing.gb",
            Self::InterruptTime => "interrupt_time.gb",
            Self::MemTiming => "mem_timing.gb",
            Self::OamBug => "oam_bug.gb",
            Self::Special => "01-special.gb",
            Self::Interrupts => "02-interrupts.gb",
            Self::OpSpHl => "03-op sp,hl.gb",
            Self::OpRImm => "04-op r,imm.gb",
            Self::OpRp => "05-op rp.gb",
            Self::LdRR => "06-ld r,r.gb",
            Self::JrJpCallRetRst => "07-jr,jp,call,ret,rst.gb",
            Self::MiscInstrs => "08-misc instrs.gb",
            Self::OpRR => "09-op r,r.gb",
            Self::BitOps => "10-bit ops.gb",
            Self::OpAHl => "11-op a,(hl).gb",
        };
        path.push(PathBuf::from(end_path));

        path
    }

    fn get_rom(&self) -> Vec<u8> {
        fs::read(self.get_rom_path()).unwrap()
    }
}

#[test]
fn test_special() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::Special.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_interrupts() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::Interrupts.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_op_sp_hl() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::OpSpHl.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_op_r_imm() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::OpRImm.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);

    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_op_rp() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::OpRp.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);

    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_ld_rr() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::LdRR.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);

    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_jr_jp_call_ret_rst() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::JrJpCallRetRst.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);

    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_misc_instrs() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::MiscInstrs.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);

    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_op_rr() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::OpRR.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_bit_ops() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::BitOps.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_op_a_hl() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::OpAHl.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}

#[test]
fn test_instr_timing() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::InstrTiming.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_mem_timing() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::MemTiming.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_interrupt_time() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::InterruptTime.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_dmg_sound() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::DmgSound.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_halt_bug() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::HaltBug.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_oam_bug() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::OamBug.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
#[test]
fn test_cgb_sound() {
    let mut game_boy = GameBoy::new();
    let cartridge = Cartridge::new(&BlarggTest::CgbSound.get_rom()).unwrap();
    game_boy.load_cartridge(cartridge);
    let mut prev_output = turn_output_to_string(game_boy.get_serial_output());

    for _ in 0..BlarggTest::MAX_RUN_CYCLES {
        game_boy.tick();

        let new_output = turn_output_to_string(game_boy.get_serial_output());
        if new_output != prev_output {
            if new_output.contains("Passed") {
                return;
            } else if new_output.contains("Failed") {
                panic!("Failed Test")
            }
            prev_output = new_output;
        }
    }
    panic!("Did not pass test within time limit")
}
