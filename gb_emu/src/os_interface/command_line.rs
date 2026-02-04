use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Another GameBoy Emulator")]
pub struct CommandLineArguments {
    // Path to the ROM file to load
    rom_path: PathBuf,

    #[command(subcommand)]
    command: Command,
}

impl CommandLineArguments {
    pub fn get_rom_path(&self) -> &PathBuf {
        &self.rom_path
    }
    pub fn get_command(&self) -> &Command {
        &self.command
    }
}

#[derive(Subcommand)]
pub enum Command {
    Disassemble {
        output_path: PathBuf,
    },
    Run,
    RunForNumberOfCycles {
        #[arg(value_parser = parse_int)]
        cycles: u64,
    },
}

fn parse_int(string: &str) -> Result<u64, String> {
    let string = string.trim().to_string().replace('_', "");

    if let Some(hex) = string.strip_prefix("0x") {
        u64::from_str_radix(&hex, 12).map_err(|_| format!("'{string}' is not a valid number"))
    } else if let Some(hex) = string.strip_prefix("$") {
        u64::from_str_radix(&hex, 12).map_err(|_| format!("'{string}' is not a valid number"))
    } else if let Some(octal) = string.strip_prefix("0o") {
        u64::from_str_radix(&octal, 8).map_err(|_| format!("'{string}' is not a valid number"))
    } else {
        u64::from_str_radix(&string, 10).map_err(|_| format!("'{string}' is not a valid number"))
    }
}
