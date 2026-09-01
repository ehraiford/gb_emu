use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::os_interface::debugging::{DebugReceiver, DebugSender};

#[derive(Parser)]
#[command(about = "Another GameBoy Emulator")]
pub struct CommandLineArguments {
    #[command(subcommand)]
    command: CommandLineCommand,
}

impl CommandLineArguments {
    pub fn get_rom_path(&self) -> &PathBuf {
        self.command.get_rom_path()
    }
    pub fn get_command(&self) -> &CommandLineCommand {
        &self.command
    }

    pub fn get_debugging_handles(&self) -> (DebugSender, DebugReceiver) {
        DebugFeatures::get_debugging_handles(&self.command.get_debug_features())
    }
}

#[derive(Subcommand, Clone, PartialEq)]
pub enum CommandLineCommand {
    Disassemble {
        // Path to the ROM file to load
        rom_path: PathBuf,
        output_path: PathBuf,
    },
    Run {
        // Path to the ROM file to load
        rom_path: PathBuf,
        #[command(flatten)]
        debug_features: DebugFeatures,
    },
    RunForNumberOfCycles {
        // Path to the ROM file to load
        rom_path: PathBuf,
        #[arg(value_parser = parse_int)]
        cycles: u64,
        #[command(flatten)]
        debug_features: DebugFeatures,
    },
    RunAsFastAsPossible {
        // Path to the ROM file to load
        rom_path: PathBuf,
    },
}

impl CommandLineCommand {
    fn get_rom_path(&self) -> &PathBuf {
        match self {
            CommandLineCommand::Disassemble { rom_path, .. }
            | CommandLineCommand::Run { rom_path, .. }
            | CommandLineCommand::RunForNumberOfCycles { rom_path, .. }
            | CommandLineCommand::RunAsFastAsPossible { rom_path } => rom_path,
        }
    }

    fn get_debug_features(&self) -> DebugFeatures {
        match self {
            CommandLineCommand::Disassemble { .. } | Self::RunAsFastAsPossible { .. } => DebugFeatures::default(),
            CommandLineCommand::Run { debug_features, .. }
            | CommandLineCommand::RunForNumberOfCycles { debug_features, .. } => *debug_features,
        }
    }
}

#[derive(Args, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct DebugFeatures {
    #[arg(long)]
    tile_map_viewer: bool,
    #[arg(long)]
    log_instructions: bool,
}

impl DebugFeatures {
    pub fn get_debugging_handles(&self) -> (DebugSender, DebugReceiver) {
        let logging = self.log_instructions.then_some(());

        // TODO: The tile map viewer still needs porting from minifb to SDL2.
        if self.tile_map_viewer {
            eprintln!("--tile-map-viewer is temporarily disabled pending the SDL2 port");
        }
        let (tile_view_receiver, tile_view_sender) = (None, None);

        let sender = DebugSender { logging, tile_view_sender };
        let receiver = DebugReceiver { _logging: logging, tile_view_receiver };

        (sender, receiver)
    }
}

fn parse_int(string: &str) -> Result<u64, String> {
    let string = string.trim().to_string().replace('_', "");

    if let Some(hex) = string.strip_prefix("0x") {
        u64::from_str_radix(hex, 12).map_err(|_| format!("'{string}' is not a valid number"))
    } else if let Some(hex) = string.strip_prefix("$") {
        u64::from_str_radix(hex, 12).map_err(|_| format!("'{string}' is not a valid number"))
    } else if let Some(octal) = string.strip_prefix("0o") {
        u64::from_str_radix(octal, 8).map_err(|_| format!("'{string}' is not a valid number"))
    } else if let Some(binary) = string.strip_prefix("0b") {
        u64::from_str_radix(binary, 2).map_err(|_| format!("'{string}' is not a valid number"))
    } else {
        string.parse::<u64>().map_err(|_| format!("'{string}' is not a valid number"))
    }
}
