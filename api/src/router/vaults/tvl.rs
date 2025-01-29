use std::{collections::HashMap, sync::Arc};

use anchor_lang::AnchorDeserialize;
use axum::{extract::State, response::IntoResponse, Json};
use jito_bytemuck::Discriminator;
use jito_vault_client::{accounts::Vault, programs::JITO_VAULT_ID};
use reqwest;
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
    let mut price_tables = HashMap::new();
    for (vault_pubkey, vault) in accounts {
        let vault = Vault::deserialize(&mut vault.data.as_slice()).unwrap();

        let price_usd = match price_tables.get(&vault.supported_mint.to_string()) {
            Some(p) => *p,
            None => {
                let url  = format!("https://api.coingecko.com/api/v3/simple/token_price/solana?contract_addresses={}&vs_currencies=usd", vault.supported_mint);
                let price_data: HashMap<String, HashMap<String, f64>> =
                    reqwest::get(url).await?.json().await?;

                let mut p = 0f64;
                if let Some(inner_map) = price_data.get(&vault.supported_mint.to_string()) {
                    if let Some(price) = inner_map.get("usd") {
                        p = *price;
                    }
                }

                price_tables.insert(vault.supported_mint.to_string(), p);
                p
            }
        };

        tvls.push(Tvl {
            vault_pubkey: vault_pubkey.to_string(),
            supported_mint: vault.supported_mint.to_string(),
            native: vault.tokens_deposited,
            usd: vault.tokens_deposited as f64 * price_usd,
        });
    }

    Ok(Json(tvls))
}
