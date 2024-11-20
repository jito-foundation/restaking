use jito_vault_core::config::Config;
use solana_metrics::datapoint_info;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;

use crate::vault_handler::VaultHandler;

pub async fn emit_vault_metrics(
    rpc_client: &RpcClient,
    config_epoch_length: u64,
) -> anyhow::Result<()> {
    let slot = rpc_client.get_slot().await?;
    let epoch = slot / config_epoch_length;
    let slot_index = slot % config_epoch_length;

    let dummy_keypair = Keypair::new(); // Dummy keypair since we're only reading
    let config_address =
        Config::find_program_address(&jito_vault_client::programs::JITO_VAULT_ID).0;
    let vault_handler = VaultHandler::new(
        rpc_client.url().as_str(),
        &dummy_keypair,
        jito_vault_client::programs::JITO_VAULT_ID,
        config_address,
        0,
    );

    // Get all vaults
    let vaults = vault_handler.get_vaults().await?;

    // Get all vault operator delegations
    let delegations = vault_handler.get_vault_operator_delegations().await?;

    // Calculate metrics
    let num_vaults = vaults.len() as i64;
    let num_vaults_updated = vaults
        .iter()
        .filter(|(_, vault)| {
            !vault
                .is_update_needed(slot, config_epoch_length)
                .expect("Config epoch length is 0")
        })
        .count() as i64;

    let num_vault_operator_delegations = delegations.len() as i64;
    let num_vault_operator_delegations_updated = delegations
        .iter()
        .filter(|(_pubkey, delegation)| {
            delegation
                .last_update_slot()
                .checked_div(config_epoch_length)
                .unwrap()
                == epoch
        })
        .count() as i64;

    datapoint_info!(
        "restaking-vault-stats",
        ("slot", slot as i64, i64),
        ("slot_index", slot_index as i64, i64),
        ("num_vaults", num_vaults, i64),
        ("num_vaults_updated", num_vaults_updated, i64),
        (
            "num_vault_operator_delegations",
            num_vault_operator_delegations,
            i64
        ),
        (
            "num_vault_operator_delegations_updated",
            num_vault_operator_delegations_updated,
            i64
        ),
    );

    Ok(())
}
