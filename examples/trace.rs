//! Emits an execution trace for diffing against a reference core (see scripts/trace_diff.py).
//!
//! Run with:
//!   cargo run --release --features headless --example trace -- <rom> <max_m_cycles> <out.trace> [skip]
//!
//! One line per M-cycle:
//!   <m_cycle> <pc> <bc> <de> <hl> <ly> <stat> <div> <tima> <if> <ie>
//!
//! The reference tracer samples once per instruction, so its cycle numbers are a subset of the
//! ones here; the differ joins the two on that column. That join is the point of the whole
//! exercise -- an instruction that takes the wrong number of cycles shows up as state landing on
//! the wrong absolute cycle, which is exactly what the mooneye timing tests measure.

fn main() {
    #[cfg(not(feature = "headless"))]
    eprintln!("build with --features headless");

    #[cfg(feature = "headless")]
    headless::run();
}

#[cfg(feature = "headless")]
mod headless {
    use gb_emu::{cartridge::cartridge::Cartridge, game_boy::GameBoy};
    use std::{
        fs,
        io::{BufWriter, Write},
    };

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        let (rom_path, max_cycles, out_path, skip) = match args.as_slice() {
            [_, rom, max, out] => (rom, max, out, 0),
            [_, rom, max, out, skip] => (rom, max, out, skip.parse().expect("skip must be a number")),
            _ => {
                eprintln!("usage: trace <rom> <max_m_cycles> <out.trace> [skip_m_cycles]");
                std::process::exit(2);
            },
        };
        let max_cycles: u64 = max_cycles.parse().expect("max_m_cycles must be a number");
        // Matches the reference tracer's skip: the boot ROM is ~1M M-cycles of scrolling logo
        // that would otherwise dominate the trace file.
        let skip: u64 = skip;

        let rom = fs::read(rom_path).expect("could not read ROM");
        let cartridge = Cartridge::new(&rom).expect("could not load cartridge");

        let mut game_boy = GameBoy::new();
        game_boy.load_cartridge(cartridge);

        let file = fs::File::create(out_path).expect("could not open trace for writing");
        let mut out = BufWriter::with_capacity(1 << 20, file);

        for cycle in 0..max_cycles {
            if cycle < skip {
                game_boy.tick();
                continue;
            }
            let [b, c, d, e, h, l] = game_boy.debug_registers();
            writeln!(
                out,
                "{cycle} {:04X} {:02X}{:02X} {:02X}{:02X} {:02X}{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                game_boy.get_pc(),
                b,
                c,
                d,
                e,
                h,
                l,
                game_boy.peek_mem(0xFF44), // LY
                game_boy.peek_mem(0xFF41), // STAT
                game_boy.peek_mem(0xFF04), // DIV
                game_boy.peek_mem(0xFF05), // TIMA
                game_boy.peek_mem(0xFF0F), // IF
                game_boy.peek_mem(0xFFFF), // IE
            )
            .expect("trace write failed");
            game_boy.tick();
        }
    }
}
