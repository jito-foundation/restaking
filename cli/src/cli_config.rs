use solana_sdk::commitment_config::CommitmentConfig;

use crate::cli_signer::CliSigner;

pub struct CliConfig {
    pub rpc_url: String,

    pub commitment: CommitmentConfig,

    pub signer: Option<CliSigner>,
}
