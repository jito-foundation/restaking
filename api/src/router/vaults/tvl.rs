use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose, Engine};
use jito_bytemuck::Discriminator;
use jito_vault_client::{accounts::Vault, programs::JITO_VAULT_ID};
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};

use crate::{error::JitoRestakingApiError, router::RouterState};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tvl {
    /// Vault Pubkey
    vault_pubkey: String,

    /// Supported Token (JitoSOL, JTO...)
    supported_mint: String,

    /// The amount of tokens deposited in Vault
    native_unit_tvl: f64,

    /// Supported mint token symbol
    native_unit_symbol: String,

    /// The amount of tokens deposited in Vault in USD
    usd_tvl: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CoinData {
    decimals: u8,
    price: f64,
    symbol: String,
    timestamp: f64,
}

#[derive(Debug, serde::Deserialize)]
struct CoinResponse {
    coins: HashMap<String, CoinData>,
}

pub async fn get_tvls(State(state): State<Arc<RouterState>>) -> crate::Result<impl IntoResponse> {
    let data_size = std::mem::size_of::<jito_vault_core::vault::Vault>()
        .checked_add(8)
        .ok_or_else(|| JitoRestakingApiError::InternalError)?;
    let encoded_discriminator = general_purpose::STANDARD.encode(vec![
        jito_vault_core::vault::Vault::DISCRIMINATOR,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]);
    let memcmp = RpcFilterType::Memcmp(Memcmp::new(
        0,
        MemcmpEncodedBytes::Base64(encoded_discriminator),
    ));
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::DataSize(data_size as u64), memcmp]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: Some(UiDataSliceConfig {
                offset: 0,
                length: data_size,
            }),
            commitment: None,
            min_context_slot: None,
        },
        with_context: Some(false),
        sort_results: Some(false),
    };

    let accounts = state
        .rpc_client
        .get_program_accounts_with_config(&JITO_VAULT_ID, config)
        .await?;

    let st_pubkeys: HashSet<String> = accounts
        .iter()
        .filter_map(
            |(_, vault)| match Vault::deserialize(&mut vault.data.as_slice()) {
                Ok(vault) => Some(vault.supported_mint.to_string()),
                Err(e) => {
                    tracing::warn!("error deserializing Vault: {:?}", e);
                    None
                }
            },
        )
        .collect();
    let st_pubkeys: Vec<String> = st_pubkeys.into_iter().collect();

    let base_url = String::from("https://coins.llama.fi/prices/current/solana:");
    let url = format!("{base_url}{}", st_pubkeys.join(",solana:"));

    let response: CoinResponse = reqwest::get(url).await.unwrap().json().await.unwrap();

    let mut tvls = Vec::new();
    for (vault_pubkey, vault) in accounts {
        let vault = Vault::deserialize(&mut vault.data.as_slice()).unwrap();

        let key = format!("solana:{}", vault.supported_mint);
        let (native_unit_symbol, price_usd, decimals) =
            response
                .coins
                .get(&key)
                .map_or(("", 0_f64, 0_u8), |coin_data| {
                    (
                        coin_data.symbol.as_str(),
                        coin_data.price,
                        coin_data.decimals,
                    )
                });

        let decimal_factor = 10u64.pow(decimals as u32) as f64;
        let native_unit_tvl = vault.tokens_deposited as f64 / decimal_factor;
        tvls.push(Tvl {
            vault_pubkey: vault_pubkey.to_string(),
            supported_mint: vault.supported_mint.to_string(),
            native_unit_tvl,
            native_unit_symbol: native_unit_symbol.to_string(),
            usd_tvl: native_unit_tvl * price_usd,
        });
    }

    tvls.sort_by(|a, b| b.usd_tvl.total_cmp(&a.usd_tvl));

    Ok(Json(tvls))
}

pub async fn get_tvl(
    State(state): State<Arc<RouterState>>,
    Path(vault_pubkey): Path<String>,
) -> crate::Result<impl IntoResponse> {
    let vault_pubkey = Pubkey::from_str(&vault_pubkey)?;

    let account = state.rpc_client.get_account(&vault_pubkey).await?;
    let vault = Vault::deserialize(&mut account.data.as_slice()).map_err(|e| {
        tracing::warn!("error deserializing Vault: {:?}", e);
        JitoRestakingApiError::AnchorError(e.into())
    })?;

    let url = format!(
        "https://coins.llama.fi/prices/current/solana:{}",
        vault.supported_mint,
    );
    let response: CoinResponse = reqwest::get(url).await.unwrap().json().await.unwrap();

    let key = format!("solana:{}", vault.supported_mint);
    let (native_unit_symbol, price_usd, decimals) =
        response
            .coins
            .get(&key)
            .map_or(("", 0_f64, 0_u8), |coin_data| {
                (
                    coin_data.symbol.as_str(),
                    coin_data.price,
                    coin_data.decimals,
                )
            });

    let decimal_factor = 10u64.pow(decimals as u32) as f64;
    let native_unit_tvl = vault.tokens_deposited as f64 / decimal_factor;
    let tvl = Tvl {
        vault_pubkey: vault_pubkey.to_string(),
        supported_mint: vault.supported_mint.to_string(),
        native_unit_tvl,
        native_unit_symbol: native_unit_symbol.to_string(),
        usd_tvl: native_unit_tvl * price_usd,
    };

    Ok(Json(tvl))
}
