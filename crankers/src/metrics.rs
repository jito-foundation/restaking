use std::collections::HashMap;

use jito_jsm_core::get_epoch;
use jito_vault_core::config::Config;
use log::error;
use solana_metrics::datapoint_info;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account as TokenAccount, Mint};

use crate::vault_handler::VaultHandler;

pub async fn emit_vault_metrics(
    rpc_client: &RpcClient,
    config_epoch_length: u64,
    cluster_name: &str,
) -> anyhow::Result<()> {
    let slot = rpc_client.get_slot().await?;
    let epoch = slot / config_epoch_length;
    let slot_index = slot % config_epoch_length;

    let config_address =
        Config::find_program_address(&jito_vault_client::programs::JITO_VAULT_ID).0;
    let vault_handler = VaultHandler::new(
        rpc_client.url().as_str(),
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
            get_epoch(delegation.last_update_slot(), config_epoch_length).unwrap() == epoch
        })
        .count() as i64;

    let vrt_mint_pubkeys: Vec<Pubkey> = vaults.iter().map(|(_, vault)| vault.vrt_mint).collect();
    let vrt_mint_accounts = rpc_client.get_multiple_accounts(&vrt_mint_pubkeys).await?;
    let vrt_mint_map: HashMap<Pubkey, Mint> = vrt_mint_pubkeys
        .into_iter()
        .zip(vrt_mint_accounts.into_iter())
        .filter_map(|(pubkey, account)| account.map(|acc| (pubkey, acc)))
        .map(|(pubkey, account)| {
            let mint = Mint::unpack(&account.data).expect("Failed to unpack Mint");
            (pubkey, mint)
        })
        .collect();

    let st_ata_pubkeys: Vec<Pubkey> = vaults
        .iter()
        .map(|(vault_address, vault)| {
            get_associated_token_address(vault_address, &vault.supported_mint)
        })
        .collect();

    let st_ata_accounts = rpc_client.get_multiple_accounts(&st_ata_pubkeys).await?;
    let st_ata_map: HashMap<Pubkey, TokenAccount> = st_ata_pubkeys
        .into_iter()
        .zip(st_ata_accounts.into_iter())
        .filter_map(|(pubkey, account)| {
            account.map(|acc| {
                (
                    pubkey,
                    TokenAccount::unpack(&acc.data).expect("Failed to unpack TokenAccount"),
                )
            })
        })
        .collect();

    for (address, vault) in vaults.iter() {
        let vrt_mint = vrt_mint_map
            .get(&vault.vrt_mint)
            .ok_or_else(|| anyhow::anyhow!("Mint not found in map"))?;

        let try_st_deposit_account = st_ata_map
            .get(&get_associated_token_address(
                address,
                &vault.supported_mint,
            ))
            .ok_or_else(|| anyhow::anyhow!("ST deposit account not found in map"));

        if try_st_deposit_account.is_err() {
            error!(
                "Failed to get ST deposit account for vault {}: {}",
                address,
                try_st_deposit_account.unwrap_err()
            );
            continue;
        }

        let st_deposit_account = try_st_deposit_account.unwrap();

        datapoint_info!(
            "restaking-vault-supply",
            "vault" => address.to_string(),
            "vrt_mint" => vault.vrt_mint.to_string(),
            ("slot", slot as i64, i64),
            ("slot_index", slot_index as i64, i64),
            ("vrt_supply_internal", vault.vrt_supply() as i64, i64),
            ("vrt_supply_external", vrt_mint.supply as i64, i64),
            ("st_supply_internal", vault.tokens_deposited() as i64, i64),
            ("st_supply_external", st_deposit_account.amount as i64, i64),
            "cluster" => cluster_name,
        );
    }

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
        "cluster" => cluster_name,
    );

    Ok(())
}
