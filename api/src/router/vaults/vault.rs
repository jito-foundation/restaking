use std::{str::FromStr, sync::Arc};

use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use jito_bytemuck::Discriminator;
use jito_vault_client::{accounts::Vault, programs::JITO_VAULT_ID};
use solana_account_decoder::UiAccountEncoding;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};

use crate::{error::JitoRestakingApiError, router::RouterState};

pub(crate) async fn list_vaults(
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

    let mut vaults = Vec::new();
    for (_vault_pubkey, vault) in accounts {
        let vault = Vault::deserialize(&mut vault.data.as_slice()).unwrap();
        vaults.push(vault);
    }

    Ok(Json(vaults))
}

pub(crate) async fn get_vault(
    State(state): State<Arc<RouterState>>,
    Path(vault_pubkey): Path<String>,
) -> crate::Result<impl IntoResponse> {
    let vault_pubkey = Pubkey::from_str(&vault_pubkey)?;
    let account = state.rpc_client.get_account(&vault_pubkey).await?;
    let vault = Vault::deserialize(&mut account.data.as_slice()).map_err(|e| {
        tracing::warn!("error deserializing Vault: {:?}", e);
        JitoRestakingApiError::AnchorError(e.into())
    })?;

    Ok(Json(vault))
}
