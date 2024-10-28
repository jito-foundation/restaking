use std::path::PathBuf;

use clap::{Error, Parser, Subcommand};
use include_idl::{
    compress_idl,
    parse::{parse_idl_from_program_binary, IdlType},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Parse {
        /// Read IDL from a solana program binary
        path: PathBuf,
        idl_type: IdlType,
    },
    Compress {
        /// Path to the input IDL JSON file
        idl_path: PathBuf,
        /// Path where the compressed IDL should be saved
        dest_path: PathBuf,
    },
}

pub fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Parse { path, idl_type }) => {
            let buffer = std::fs::read(path).expect("Could not read file.");
            if let Ok(idl) = parse_idl_from_program_binary(&buffer, idl_type.clone()) {
                println!("        Program IDL");
                println!("============================");
                println!("{}", idl);
            } else {
                println!("Could not find {:?} IDL in program binary", idl_type);
            }
        }
        Some(Commands::Compress {
            idl_path,
            dest_path,
        }) => {
            compress_idl(idl_path, dest_path);
            println!(
                "Successfully compressed IDL from {:?} to {:?}",
                idl_path, dest_path
            );
        }
        None => {}
    }
    Ok(())
}
