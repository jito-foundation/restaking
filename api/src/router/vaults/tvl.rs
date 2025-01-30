use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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

    /// The amount of tokens deposited in Vault in USD
    usd: f64,
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

    let st_pubkeys: HashSet<String> = accounts
        .iter()
        .map(|(_, vault)| {
            let vault = Vault::deserialize(&mut vault.data.as_slice()).unwrap();
            vault.supported_mint.to_string()
        })
        .collect();
    let st_pubkeys: Vec<String> = st_pubkeys.into_iter().collect();

    let base_url = String::from("https://coins.llama.fi/prices/current/solana:");
    let url = format!("{base_url}{}", st_pubkeys.join(",solana:").to_string());

    let response: CoinResponse = reqwest::get(url).await.unwrap().json().await.unwrap();

    let mut tvls = Vec::new();
    for (vault_pubkey, vault) in accounts {
        let vault = Vault::deserialize(&mut vault.data.as_slice()).unwrap();

        let key = format!("solana:{}", vault.supported_mint.to_string());
        let price_usd = match response.coins.get(&key) {
            Some(coin_data) => coin_data.price,
            None => 0_f64,
        };

        tvls.push(Tvl {
            vault_pubkey: vault_pubkey.to_string(),
            supported_mint: vault.supported_mint.to_string(),
            native: vault.tokens_deposited,
            usd: vault.tokens_deposited as f64 * price_usd,
        });
    }

    tvls.sort_by(|a, b| b.usd.total_cmp(&a.usd));

    Ok(Json(tvls))
}
