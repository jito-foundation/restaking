use std::{collections::HashMap, io::Write, path::PathBuf, str::FromStr};

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use env_logger::{
    fmt::{Color, Formatter, Style, StyledValue},
    Env,
};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_restaking_client::programs::JITO_RESTAKING_ID;
use jito_vault_client::programs::JITO_VAULT_ID;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation, vault_update_state_tracker::VaultUpdateStateTracker
};
use log::{error, Record};
use solana_account_decoder::UiAccountEncoding;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    account::ReadableAccount, commitment_config::CommitmentConfig, pubkey::Pubkey,
    signature::Keypair, signer::keypair::read_keypair_file,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The RPC URL to connect to
    #[arg(short, long)]
    rpc_url: String,

    /// Path to the keypair file
    #[arg(short, long)]
    keypair: PathBuf,

    /// The program ID of the vault program
    #[arg(short, long)]
    vault_program_id: Option<String>,

    /// The program ID of the restaking program
    #[arg(short, long)]
    restaking_program_id: Option<String>,
}

pub fn init_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(format_log_message)
        .init();
}

fn format_log_message(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let mut style = buf.style();
    let level = colored_level(&mut style, record.level());

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

    writeln!(
        buf,
        "[{} {} {}] {}",
        timestamp,
        level,
        record.target(),
        record.args()
    )
}

fn colored_level(style: &mut Style, level: log::Level) -> StyledValue<&'static str> {
    match level {
        log::Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        log::Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        log::Level::Info => style.set_color(Color::Green).value("INFO "),
        log::Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        log::Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let args = Args::parse();
    let keypair = read_keypair_file(args.keypair)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

    let rpc_client = RpcClient::new(args.rpc_url);

    let restaking_program_id = if let Some(restaking_program_id) = &args.restaking_program_id {
        Pubkey::from_str(restaking_program_id)?
    } else {
        JITO_RESTAKING_ID
    };

    let vault_program_id = if let Some(vault_program_id) = &args.vault_program_id {
        Pubkey::from_str(vault_program_id)?
    } else {
        JITO_VAULT_ID
    };

    loop {
        match run_crank_loop(
            &rpc_client,
            &keypair,
            &restaking_program_id,
            &vault_program_id,
        )
        .await
        {
            Ok(_) => (),
            Err(e) => {
                error!("Error: {}", e);
                continue;
            }
        }
    }
}
async fn run_crank_loop(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    restaking_program_id: &Pubkey,
    vault_program_id: &Pubkey,
) -> Result<()> {
    let slot = rpc_client
        .get_slot_with_commitment(CommitmentConfig::confirmed())
        .await?;

    let config = rpc_client
        .get_account(&Config::find_program_address(&vault_program_id).0)
        .await?;
    let config = Config::try_from_slice_unchecked(config.data())?;
    let vault_epoch_length = config.epoch_length();

    let vault_accounts = rpc_client
        .get_program_accounts_with_config(
            vault_program_id,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                    0,
                    MemcmpEncodedBytes::Bytes(vec![Vault::DISCRIMINATOR]),
                ))]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await?;

    let all_vault_operator_delegations = rpc_client
        .get_program_accounts_with_config(
            vault_program_id,
            RpcProgramAccountsConfig {
                filters: Some(vec![
                    RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![VaultOperatorDelegation::DISCRIMINATOR]),
                    )),
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await?;

    let vaults = vault_accounts
        .iter()
        .map(|(pubkey, account)| Ok((pubkey, Vault::try_from_slice_unchecked(account.data())?)))
        .collect::<Result<Vec<_>>>()?;

    let current_epoch = slot / vault_epoch_length;
    let vaults_need_updating: Vec<_> = vaults
        .iter()
        .filter(|(_, vault)| {
            vault.last_full_state_update_slot() / vault_epoch_length != current_epoch
        })
        .collect();

    // Group vault operator delegations by vaults that need updating
    let mut grouped_delegations: HashMap<Pubkey, Vec<(Pubkey, VaultOperatorDelegation)>> = HashMap::new();
    for (pubkey, account) in all_vault_operator_delegations {
        let delegation = VaultOperatorDelegation::try_from_slice_unchecked(account.data())?;
        if vaults_need_updating.iter().any(|(vault_pubkey, _)| **vault_pubkey == delegation.vault) {
            grouped_delegations
                .entry(delegation.vault)
                .or_default()
                .push((pubkey, *delegation));
        }
    }

    // TODO: Process the grouped vault operator delegations

    Ok(())
}
