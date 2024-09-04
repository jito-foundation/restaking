use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair};

pub mod cli_args;
pub mod log;
pub mod restaking;
pub mod restaking_handler;
pub mod vault;
pub mod vault_handler;

pub struct CliConfig {
    pub rpc_url: String,

    pub commitment: CommitmentConfig,

    pub keypair: Option<Keypair>,
}
