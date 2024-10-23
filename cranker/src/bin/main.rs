use std::{collections::HashMap, path::PathBuf, time::Duration};

use anyhow::Context;
use clap::Parser;
use jito_bytemuck::AccountDeserialize;
use jito_restaking_cranker::{
    restaking_handler::RestakingHandler, vault_handler::VaultHandler,
    vault_update_state_tracker_handler::VaultUpdateStateTrackerHandler,
};
use jito_vault_core::{vault::Vault, vault_operator_delegation::VaultOperatorDelegation};
use log::info;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file};

#[derive(Parser)]
struct Args {
    /// RPC URL for the cluster
    #[arg(short, long, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,

    /// Path to keypair used to pay
    #[arg(short, long)]
    keypair: PathBuf,

    /// Vault program ID (Pubkey as base58 string)
    #[arg(long, default_value = "34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx")]
    vault_program_id: Pubkey,

    /// Restaking program ID (Pubkey as base58 string)
    #[arg(long, default_value = "78J8YzXGGNynLRpn85MH77PVLBZsWyLCHZAXRvKaB6Ng")]
    restaking_program_id: Pubkey,
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let rpc_client = RpcClient::new_with_timeout(args.rpc_url.clone(), Duration::from_secs(60));
    let payer = read_keypair_file(args.keypair).expect("read keypair file");

    let config_address =
        jito_vault_core::config::Config::find_program_address(&args.vault_program_id).0;

    let account = rpc_client
        .get_account(&config_address)
        .await
        .expect("Failed to read Jito vault config address");
    let config = jito_vault_core::config::Config::try_from_slice_unchecked(&account.data)
        .expect("Failed to deserialize Jito vault config");

    let restaking_handler = RestakingHandler::new(&args.rpc_url);
    let vault_handler =
        VaultHandler::new(&args.rpc_url, &payer, args.vault_program_id, config_address);
    let vault_update_state_tracker_handler =
        VaultUpdateStateTrackerHandler::new(&args.rpc_url, args.vault_program_id);

    loop {
        let slot = rpc_client.get_slot().await.context("get slot")?;
        let epoch = slot.checked_div(config.epoch_length()).unwrap();

        log::info!("Slot: {slot}, Current Epoch: {epoch}");

        let vaults = vault_handler.get_vaults().await?;
        let delegations = vault_handler.get_vault_operator_delegation().await?;

        let vaults_need_update: Vec<(Pubkey, Vault)> = vaults
            .into_iter()
            .filter(|(_pubkey, vault)| {
                vault
                    .last_full_state_update_slot()
                    .checked_div(config.epoch_length())
                    .unwrap()
                    != epoch
            })
            .collect();

        let delegations_need_update: Vec<(Pubkey, VaultOperatorDelegation)> = delegations
            .into_iter()
            .filter(|(_pubkey, delegation)| {
                delegation
                    .last_update_slot()
                    .checked_div(config.epoch_length())
                    .unwrap()
                    != epoch
            })
            .collect();

        let mut grouped_delegations: HashMap<Pubkey, Vec<(Pubkey, VaultOperatorDelegation)>> =
            HashMap::new();
        for (pubkey, delegation) in delegations_need_update {
            if vaults_need_update
                .iter()
                .any(|(vault_pubkey, _)| *vault_pubkey == delegation.vault)
            {
                grouped_delegations
                    .entry(delegation.vault)
                    .or_default()
                    .push((pubkey, delegation));
            }
        }

        for grouped_delegation in grouped_delegations {
            let operator_pubkeys: Vec<Pubkey> = grouped_delegation
                .1
                .iter()
                .map(|(_pubkey, delegation)| delegation.operator)
                .collect();
            let operators = restaking_handler.get_operators(&operator_pubkeys).await?;

            match vault_update_state_tracker_handler
                .get_update_state_tracker(&grouped_delegation.0, epoch)
                .await
            {
                Ok(_) => {
                    if let Err(e) = vault_handler
                        .crank(epoch, &grouped_delegation.0, &operators)
                        .await
                    {
                        log::error!("{e}");
                    }
                }
                Err(e) => {
                    log::error!("{e}");
                }
            }
        }

        // ---------- SLEEP (1 hour)----------
        info!("Sleep 1 hour");
        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}
