use axum::{
    extract::{Path, State},
    routing::get,
    Router, Json,
};
use std::{str::FromStr, sync::Arc};
use tracing::{instrument, info};
use serde_json::json;
use solana_account_decoder::UiAccountEncoding;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use jito_restaking_core::{operator::Operator, config::Config};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use solana_sdk::pubkey::{Pubkey};
use std::time::Instant;

use crate::common::{AppState, ApiError};

#[instrument(skip(state))]
async fn get_operator(
    Path(pubkey): Path<String>,
    state: State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start = Instant::now();
    info!("Getting operator for pubkey: {}", pubkey);
    let pubkey = Pubkey::from_str(&pubkey)?;
    let account = state.rpc_client.get_account(&pubkey).await?;
    let operator = Operator::try_from_slice_unchecked(&account.data)
        .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
    let response = Json(json!({
        "pubkey": pubkey.to_string(),
        "operator": operator,
    }));
    info!("get_operator completed in {:?}", start.elapsed());
    Ok(response)
}

#[instrument(skip(state))]
async fn list_operators(state: State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ApiError> {
    let start = Instant::now();
    info!("Listing operators");
    let operator_accounts = state
        .rpc_client
        .get_program_accounts_with_config(
            &state.restaking_program_id,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                    0,
                    MemcmpEncodedBytes::Bytes(vec![Operator::DISCRIMINATOR]),
                ))]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..RpcAccountInfoConfig::default()
                },
                with_context: None,
            },
        )
        .await?;

    let operators: Result<Vec<_>, ApiError> = operator_accounts
        .into_iter()
        .map(|(pubkey, account)| {
            let operator = Operator::try_from_slice_unchecked(&account.data)
                .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
            Ok(json!({
                "pubkey": pubkey.to_string(),
                "operator": operator,
            }))
        })
        .collect();

    let operators = operators?;
    let response = Json(json!({ "operators": operators }));

    info!("list_operators completed in {:?}", start.elapsed());
    Ok(response)
}

#[instrument(skip(state))]
async fn get_restaking_config(state: State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ApiError> {
    let start = Instant::now();
    info!("Getting restaking config");
    let (config_pubkey, _, _) = Config::find_program_address(&state.restaking_program_id);
    let account = state.rpc_client.get_account(&config_pubkey).await?;
    let config = Config::try_from_slice_unchecked(&account.data)
        .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
    let response = Json(json!({
        "pubkey": config_pubkey.to_string(),
        "config": config,
    }));
    info!("get_restaking_config completed in {:?}", start.elapsed());
    Ok(response)
}

pub fn restaking_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/operator/:pubkey", get(get_operator))
        .route("/operators", get(list_operators))
        .route("/config", get(get_restaking_config))
}