use std::sync::Arc;

use anchor_lang::AnchorDeserialize;
use axum::{extract::State, response::IntoResponse, Json};
use jito_bytemuck::Discriminator;
use jito_vault_client::{accounts::Vault, programs::JITO_VAULT_ID};
use solana_account_decoder::UiAccountEncoding;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};

use crate::router::RouterState;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Tvl {
    /// Vault Pubkey
    vault_pubkey: String,

    /// Supported Token (JitoSOL, JTO...)
    supported_mint: String,

    /// The amount of tokens deposited in Vault
    native: u64,
    usd: u64,
}

/// Retrieves the history of a specific validator, based on the provided vote account and optional epoch filter.
///
/// # Returns
/// - `Ok(Json(history))`: A JSON response containing the validator history information. If the epoch filter is provided, it only returns the history for the specified epoch.
///
/// # Example
/// This endpoint can be used to fetch the history of a validator's performance over time, either for a specific epoch or for all recorded epochs:
/// ```
/// GET /validator_history/{vote_account}?epoch=200
/// ```
/// This request retrieves the history for the specified vote account, filtered by epoch 200.
pub(crate) async fn get_tvls(
    State(state): State<Arc<RouterState>>,
) -> crate::Result<impl IntoResponse> {
    let accounts = state
        .rpc_client
        .get_program_accounts_with_config(
            &JITO_VAULT_ID,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                    0,
                    MemcmpEncodedBytes::Bytes(vec![jito_vault_core::vault::Vault::DISCRIMINATOR]),
                ))]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
                with_context: None,
            },
        )
        .await?;

    let mut tvls = Vec::new();
    for (vault_pubkey, vault) in accounts {
        let vault = Vault::deserialize(&mut vault.data.as_slice()).unwrap();

        tvls.push(Tvl {
            vault_pubkey: vault_pubkey.to_string(),
            supported_mint: vault.supported_mint.to_string(),
            native: vault.tokens_deposited,
            usd: 0,
        });
    }

    Ok(Json(tvls))
}
