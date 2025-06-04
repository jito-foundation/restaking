use std::{
    collections::HashMap,
    fmt,
    path::PathBuf,
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context};
use clap::{arg, Parser, ValueEnum};
use dotenv::dotenv;
use jito_jsm_core::get_epoch;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
};
use jito_vault_cranker::{metrics::emit_vault_metrics, vault_handler::VaultHandler};
use log::{error, info};
use solana_metrics::set_host_id;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file};

#[derive(Parser)]
struct Args {
    /// RPC URL for the cluster
    #[arg(short, long, env, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,

    /// Cluster name (e.g., devnet, mainnet)
    #[arg(short, long, env, value_enum, default_value_t = Cluster::Mainnet)]
    cluster: Cluster,

    /// Deployed region - component of metrics host_id
    #[arg(long, env, default_value = "local")]
    region: String,

    /// Path to keypair used to pay
    #[arg(short, long, env)]
    keypair_path: PathBuf,

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

    /// Interval in seconds between metrics emission (default: 5 minutes)
    #[arg(long, env, default_value = "300")]
    metrics_interval: u64,

    /// Priority fees (in microlamports per compute unit)
    #[arg(long, env, default_value = "10000")]
    priority_fees: u64,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Jito Vault Cranker Configuration:\n\
            -------------------------------\n\
            RPC URL: {}\n\
            Keypair Path: {:?}\n\
            Vault Program ID: {}\n\
            Restaking Program ID: {}\n\
            Crank Interval: {} seconds\n\
            Metrics Interval: {} seconds\n\
            Priority Fees: {} microlamports\n\
            -------------------------------",
            self.rpc_url,
            self.keypair_path,
            self.vault_program_id,
            self.restaking_program_id,
            self.crank_interval,
            self.metrics_interval,
            self.priority_fees,
        )
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum Cluster {
    Mainnet,
    Testnet,
    Localnet,
}

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Localnet => write!(f, "localnet"),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    dotenv().ok();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    info!("{}", args);

    let hostname_cmd = Command::new("hostname")
        .output()
        .expect("Failed to execute hostname command");

    let hostname = String::from_utf8_lossy(&hostname_cmd.stdout)
        .trim()
        .to_string();

    set_host_id(format!(
        "restaking-cranker_{}_{}_{}",
        args.region, args.cluster, hostname
    ));

    let rpc_client = RpcClient::new_with_timeout(args.rpc_url.clone(), Duration::from_secs(60));
    let config_address = Config::find_program_address(&args.vault_program_id).0;

    let vault_handler = Arc::new(VaultHandler::new(
        &args.rpc_url,
        args.vault_program_id,
        config_address,
        args.priority_fees,
    ));

    // Track vault metrics in separate thread
    tokio::spawn({
        let config: Config = vault_handler
            .get_vault_program_account(&config_address)
            .await?;
        async move {
            let metrics_client = RpcClient::new_with_timeout(args.rpc_url, Duration::from_secs(60));
            loop {
                if let Err(e) = emit_vault_metrics(&metrics_client, config.epoch_length()).await {
                    error!("Failed to emit metrics: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(args.metrics_interval)).await;
            }
        }
    });

    loop {
        let config: Config = vault_handler
            .get_vault_program_account(&config_address)
            .await?;

        let slot = rpc_client.get_slot().await.context("get slot")?;
        let epoch = get_epoch(slot, config.epoch_length()).unwrap();

        info!("Checking for vaults to update. Slot: {slot}, Current Epoch: {epoch}");

        let vaults = vault_handler.get_vaults().await?;
        let delegations = vault_handler.get_vault_operator_delegations().await?;

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
            HashMap::from_iter(vaults_need_update.iter().map(|(vault, _)| (*vault, vec![])));
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

        info!("Updating {} vaults", vaults_need_update.len());

        let start = Instant::now();

        let tasks: Vec<_> = grouped_delegations
            .into_iter()
            .map(|(vault, mut delegations)| {
                // Sort by VaultOperatorDelegation index for correct cranking order
                delegations.sort_by_key(|(_pubkey, delegation)| delegation.index());
                let operators: Vec<Pubkey> = delegations
                    .iter()
                    .map(|(_pubkey, delegation)| delegation.operator)
                    .collect();

                // Spawn each vault update as a separate task
                tokio::spawn({
                    let vault_handler = vault_handler.clone();
                    let payer = read_keypair_file(&args.keypair_path)
                        .map_err(|e| anyhow!("Failed to read keypair file: {}", e))
                        .unwrap();
                    async move {
                        match vault_handler
                            .do_vault_update(slot, &config, &payer, &vault, &operators)
                            .await
                        {
                            Ok(_) => {
                                info!("Successfully updated vault: {vault}");
                            }
                            Err(e) => {
                                error!("Failed to update vault: {vault}, error: {e}");
                            }
                        }
                    }
                })
            })
            .collect();

        for task in tasks {
            if let Err(e) = task.await {
                error!("Task failed to complete: {}", e);
            }
        }

        log::info!("Time elapsed: {:.2}s", start.elapsed().as_secs_f64());

        info!("Sleeping for {} seconds", args.crank_interval);
        // ---------- SLEEP (crank_interval)----------
        tokio::time::sleep(Duration::from_secs(args.crank_interval)).await;
    }
}
