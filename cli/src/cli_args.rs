use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{restaking::RestakingCommands, vault::VaultCommands};

#[derive(Parser)]
#[command(author, version, about = "A CLI for managing restaking and vault operations", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<ProgramCommand>,

    #[arg(long, global = true, help = "Path to the configuration file")]
    pub config_file: Option<PathBuf>,

    #[arg(long, global = true, help = "RPC URL to use")]
    pub rpc_url: Option<String>,

    #[arg(long, global = true, help = "Commitment level")]
    pub commitment: Option<String>,

    #[arg(long, global = true, help = "Restaking program ID")]
    pub restaking_program_id: Option<String>,

    #[arg(long, global = true, help = "Vault program ID")]
    pub vault_program_id: Option<String>,

    #[arg(long, global = true, help = "Keypair")]
    pub signer: Option<String>,

    #[arg(long, global = true, help = "Verbose mode")]
    pub verbose: bool,

    #[arg(
        long,
        global = true,
        default_value = "false",
        help = "This will print out the raw TX instead of running it"
    )]
    pub print_tx: bool,

    #[arg(long, global = true, hide = true)]
    pub markdown_help: bool,
}

#[derive(Subcommand)]
pub enum ProgramCommand {
    /// Restaking program commands
    Restaking {
        #[command(subcommand)]
        action: RestakingCommands,
    },
    /// Vault program commands
    Vault {
        #[command(subcommand)]
        action: VaultCommands,
    },
}
