use jito_vault_sdk::error::VaultError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JitoVaultCrankerError {
    #[error("Failed to read keypair file: {0}")]
    KeypairRead(String),

    #[error("Solana RPC Client error: {0}")]
    RpcClient(#[from] solana_rpc_client_api::client_error::Error),

    #[error("Math error: {0}")]
    MathError(String),

    #[error("Math overflow: {0}")]
    MathOverflow(String),

    #[error("Transaction failed after {retries} retries: {last_error}")]
    TransactionRetryExhausted { retries: u8, last_error: String },

    #[error("Failed to deserialize {account_type} at {pubkey}: {src}")]
    Deserialization {
        account_type: String,
        pubkey: String,
        src: String,
    },

    #[error("Vault Error: {0}")]
    Vault(#[from] VaultError),

    #[error("Cranking incomplete for vault {vault}, tracker {tracker}: not all operators updated")]
    IncompleteCranking { vault: String, tracker: String },
}
