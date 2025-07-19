use solana_sdk::commitment_config::CommitmentConfig;

use crate::cli_signer::CliSigner;

pub struct CliConfig {
    /// The RPC endpoint URL
    pub rpc_url: String,

    /// The Commitment level
    pub commitment: CommitmentConfig,

    /// Optional signer
    pub signer: Option<CliSigner>,
}
