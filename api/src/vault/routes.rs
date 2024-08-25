use axum::{
    extract::{Path, State}, routing::get, Json, Router
};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_vault_core::{config::Config, vault::Vault};
use serde_json::json;
use solana_account_decoder::UiAccountEncoding;
use solana_rpc_client_api::{config::{RpcAccountInfoConfig, RpcProgramAccountsConfig}, filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType}};
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::Arc, time::Instant};
use tracing::{info, instrument};

use crate::common::{AppState, ApiError};

#[instrument(skip(state))]
async fn get_vault(
    Path(pubkey): Path<String>,
    state: State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start = Instant::now();
    info!("Getting vault for pubkey: {}", pubkey);
    let pubkey = Pubkey::from_str(&pubkey)?;
    let account = state.rpc_client.get_account(&pubkey).await?;
    let vault = Vault::try_from_slice_unchecked(&account.data)
        .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
    let response = Json(json!({
        "pubkey": pubkey.to_string(),
        "vault": vault,
    }));
    info!("get_vault completed in {:?}", start.elapsed());
    Ok(response)
}

#[instrument(skip(state))]
async fn list_vaults(state: State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ApiError> {
    let start = Instant::now();
    info!("Listing vaults");

    let vault_accounts = state
        .rpc_client
        .get_program_accounts_with_config(
            &state.vault_program_id,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                    0,
                    MemcmpEncodedBytes::Bytes(vec![Vault::DISCRIMINATOR]),
                ))]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..RpcAccountInfoConfig::default()
                },
                with_context: None,
            },
        )
        .await?;

    let vaults: Result<Vec<_>, ApiError> = vault_accounts
        .into_iter()
        .map(|(pubkey, account)| {
            let vault = Vault::try_from_slice_unchecked(&account.data)
                .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
            Ok(json!({
                "pubkey": pubkey.to_string(),
                "vault": vault,
            }))
        })
        .collect();

    let vaults = vaults?;
    let response = Json(json!({ "vaults": vaults }));

    info!("list_vaults completed in {:?}", start.elapsed());
    Ok(response)
}

#[instrument(skip(state))]
async fn get_vault_config(state: State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ApiError> {
    let start = Instant::now();
    info!("Getting vault config");
    let (config_pubkey, _, _) = Config::find_program_address(&state.vault_program_id);
    let account = state.rpc_client.get_account(&config_pubkey).await?;
    let config = Config::try_from_slice_unchecked(&account.data)
        .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
    let response = Json(json!({
        "pubkey": config_pubkey.to_string(),
        "config": config,
    }));
    info!("get_vault_config completed in {:?}", start.elapsed());
    Ok(response)
}

pub fn vault_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/vault/:pubkey", get(get_vault))
        .route("/vaults", get(list_vaults))
        .route("/config/vault", get(get_vault_config))
}