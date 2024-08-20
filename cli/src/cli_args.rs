use clap::Subcommand;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair};

use crate::{restaking_handler::RestakingCommands, vault_handler::VaultCommands};

pub struct CliConfig {
    pub rpc_url: String,

    pub commitment: CommitmentConfig,

    pub keypair: Option<Keypair>,
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
