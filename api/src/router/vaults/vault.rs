use std::{str::FromStr, sync::Arc};

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

pub async fn list_vaults(
    State(state): State<Arc<RouterState>>,
) -> crate::Result<impl IntoResponse> {
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

    let mut vaults = Vec::new();
    for (_vault_pubkey, vault) in accounts {
        let vault = Vault::deserialize(&mut vault.data.as_slice()).map_err(|e| {
            tracing::warn!("error deserializing Vault: {:?}", e);
            JitoRestakingApiError::AnchorError(e.into())
        })?;
        vaults.push(vault);
    }

    Ok(Json(vaults))
}

pub async fn get_vault(
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
