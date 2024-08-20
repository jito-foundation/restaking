use anyhow::anyhow;
use clap::Parser;
use jito_restaking_cli::cli_args::{CliConfig, ProgramCommand};
use jito_restaking_cli::restaking_handler::RestakingCliHandler;
use jito_restaking_cli::vault_handler::VaultCliHandler;
use jito_restaking_client::generated::programs::JITO_RESTAKING_PROGRAM_ID;
use jito_vault_client::generated::programs::JITO_VAULT_PROGRAM_ID;
use solana_cli_config::Config;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::read_keypair_file;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
#[command(author, version, about = "A CLI for managing restaking and vault operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: ProgramCommand,

    #[arg(long, global = true, help = "Path to the configuration file")]
    config_file: Option<PathBuf>,

    #[arg(long, global = true, help = "RPC URL to use")]
    rpc_url: Option<String>,

    #[arg(long, global = true, help = "Commitment level")]
    commitment: Option<String>,

    #[arg(long, global = true, help = "Restaking program ID")]
    restaking_program_id: Option<String>,

    #[arg(long, global = true, help = "Vault program ID")]
    vault_program_id: Option<String>,

    #[arg(long, global = true, help = "Keypair")]
    keypair: Option<String>,
}

fn get_cli_config(args: &Cli) -> Result<CliConfig, anyhow::Error> {
    let cli_config = if let Some(config_file) = &args.config_file {
        let config = Config::load(config_file.as_os_str().to_str().unwrap().as_ref())?;
        CliConfig {
            rpc_url: config.json_rpc_url,
            commitment: CommitmentConfig::from_str(&config.commitment)?,
            keypair: Some(
                read_keypair_file(config.keypair_path).map_err(|e| anyhow!(e.to_string()))?,
            ),
        }
    } else {
        let config_file = solana_cli_config::CONFIG_FILE
            .as_ref()
            .ok_or_else(|| anyhow!("unable to get config file path"))?;
        let config = if let Ok(config) = Config::load(&config_file) {
            CliConfig {
                rpc_url: config.json_rpc_url,
                commitment: CommitmentConfig::from_str(&config.commitment)?,
                keypair: Some(
                    read_keypair_file(config.keypair_path).map_err(|e| anyhow!(e.to_string()))?,
                ),
            }
        } else {
            CliConfig {
                rpc_url: args
                    .rpc_url
                    .as_ref()
                    .ok_or(anyhow!("RPC URL not provided"))?
                    .to_string(),
                commitment: if let Some(commitment) = &args.commitment {
                    CommitmentConfig::from_str(commitment)?
                } else {
                    CommitmentConfig::confirmed()
                },
                keypair: if let Some(keypair) = &args.keypair {
                    Some(read_keypair_file(keypair).map_err(|e| anyhow!(e.to_string()))?)
                } else {
                    None
                },
            }
        };
        config
    };

    Ok(cli_config)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Cli = Cli::parse();

    let cli_config = get_cli_config(&args)?;

    let restaking_program_id = if let Some(restaking_program_id) = &args.restaking_program_id {
        Pubkey::from_str(restaking_program_id)?
    } else {
        JITO_RESTAKING_PROGRAM_ID
    };

    let vault_program_id = if let Some(vault_program_id) = &args.vault_program_id {
        Pubkey::from_str(vault_program_id)?
    } else {
        JITO_VAULT_PROGRAM_ID
    };

    match args.command {
        ProgramCommand::Restaking { action } => {
            RestakingCliHandler::new(cli_config, restaking_program_id, vault_program_id)
                .handle(action)
                .await?;
        }
        ProgramCommand::Vault { action } => {
            VaultCliHandler::new(cli_config, restaking_program_id, vault_program_id)
                .handle(action)
                .await?;
        }
    }

    Ok(())
}
