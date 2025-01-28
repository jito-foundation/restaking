use std::{str::FromStr, sync::Arc};

use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use jito_vault_client::accounts::Vault;

use crate::{error::JitoRestakingApiError, router::RouterState};

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
