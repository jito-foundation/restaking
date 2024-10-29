use std::{collections::HashMap, path::PathBuf, time::Duration};

use anyhow::{anyhow, Context};
use clap::{arg, Parser};
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::{vault::Vault, vault_operator_delegation::VaultOperatorDelegation};
use jito_vault_cranker::{restaking_handler::RestakingHandler, vault_handler::VaultHandler};
use log::info;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file};

#[derive(Parser)]
struct Args {
    /// RPC URL for the cluster
    #[arg(short, long, env, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,

    /// Path to keypair used to pay
    #[arg(short, long, env = "KEYPAIR_PATH")]
    keypair: PathBuf,

    /// Vault program ID (Pubkey as base58 string)
    #[arg(
        long,
        env,
        default_value = "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8"
    )]
    vault_program_id: Pubkey,

    /// Restaking program ID (Pubkey as base58 string)
    #[arg(
        long,
        env,
        default_value = "RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q"
    )]
    restaking_program_id: Pubkey,

    /// Interval in seconds between cranking attempts (default: 5 minutes)
    #[arg(long, env, default_value = "300")]
    crank_interval: u64,

    /// Priority fees (in microlamports per compute unit)
    #[arg(long, env, default_value = "10000")]
    priority_fees: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let rpc_client = RpcClient::new_with_timeout(args.rpc_url.clone(), Duration::from_secs(60));
    let payer = read_keypair_file(&args.keypair)
        .map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;

    let config_address =
        jito_vault_core::config::Config::find_program_address(&args.vault_program_id).0;

    let account = rpc_client
        .get_account(&config_address)
        .await
        .context("Failed to read Jito vault config address")?;
    let config = jito_vault_core::config::Config::try_from_slice_unchecked(&account.data)
        .context("Failed to deserialize Jito vault config")?;

    let restaking_handler = RestakingHandler::new(&args.rpc_url);
    let vault_handler = VaultHandler::new(
        &args.rpc_url,
        &payer,
        args.vault_program_id,
        config_address,
        args.priority_fees,
    );

    loop {
        let slot = rpc_client.get_slot().await.context("get slot")?;
        let epoch = slot.checked_div(config.epoch_length()).unwrap();

        info!("Checking for vaults to update. Slot: {slot}, Current Epoch: {epoch}");

        let vaults = vault_handler.get_vaults().await?;
        let delegations = vault_handler.get_vault_operator_delegation().await?;

        let vaults_need_update: Vec<(Pubkey, Vault)> = vaults
            .into_iter()
            .filter(|(_pubkey, vault)| {
                vault
                    .is_update_needed(slot, config.epoch_length())
                    .expect("Config epoch length is 0")
            })
            .collect();

        // All delegations are passed along. Delegation filtering logic is handled in `VaultHandler::crank`
        let mut grouped_delegations: HashMap<Pubkey, Vec<(Pubkey, VaultOperatorDelegation)>> =
            HashMap::new();
        for (pubkey, delegation) in delegations {
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

        for (vault, mut delegations) in grouped_delegations {
            // Sort by VaultOperatorDelegation index for correct cranking order
            delegations.sort_by_key(|(_pubkey, delegation)| delegation.index());
            let operator_pubkeys: Vec<Pubkey> = delegations
                .iter()
                .map(|(_pubkey, delegation)| delegation.operator)
                .collect();
            let operators: Vec<Pubkey> = restaking_handler
                .get_operators(&operator_pubkeys)
                .await?
                .into_iter()
                .map(|(pubkey, _)| pubkey)
                .collect();

            match vault_handler
                .do_vault_update(epoch, &vault, &operators)
                .await
            {
                Err(e) => log::error!("Failed to update vault: {vault}, error: {e}"),
                Ok(_) => info!("Successfully updated vault: {vault}"),
            }
        }

        // ---------- SLEEP (crank_interval)----------
        tokio::time::sleep(Duration::from_secs(args.crank_interval)).await;
    }
}
