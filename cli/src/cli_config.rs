use solana_commitment_config::CommitmentConfig;

use crate::cli_signer::CliSigner;

pub struct CliConfig {
    /// The RPC endpoint URL
    pub rpc_url: String,

    /// The commitment level
    pub commitment: CommitmentConfig,

    /// Optional signer
    pub signer: Option<CliSigner>,
}
