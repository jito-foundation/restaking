use crate::restaking_handler::RestakingCommands;
use crate::vault_handler::VaultCommands;
use clap::Subcommand;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Keypair;

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
