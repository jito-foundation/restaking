use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use clap_markdown::MarkdownOptions;
use env_logger::Env;
use jito_restaking_cli::{
    cli_args::{Cli, ProgramCommand},
    restaking_handler::RestakingCliHandler,
    vault_handler::VaultCliHandler,
    CliConfig,
};
use jito_restaking_client::programs::JITO_RESTAKING_ID;
use jito_vault_client::programs::JITO_VAULT_ID;
use solana_cli_config::Config;
use solana_program::pubkey::Pubkey;
use solana_sdk::{commitment_config::CommitmentConfig, signature::read_keypair_file};

pub fn get_cli_config(args: &Cli) -> Result<CliConfig, anyhow::Error> {
    let cli_config = if let Some(config_file) = &args.config_file {
        let config = Config::load(config_file.as_os_str().to_str().unwrap())?;
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
        if let Ok(config) = Config::load(config_file) {
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
                    .ok_or_else(|| anyhow!("RPC URL not provided"))?
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
        }
    };

    Ok(cli_config)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Cli = Cli::parse();

    if args.markdown_help {
        let markdown = clap_markdown::help_markdown_custom::<Cli>(&MarkdownOptions::new().show_table_of_contents(false));
        println!("---");
        println!("title: CLI");
        println!("category: Jekyll");
        println!("layout: page");
        println!("weight: 1");
        println!("---");
        println!("");
        println!("{}", markdown);
        return Ok(());
    }

    let cli_config = get_cli_config(&args)?;

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

    match args.command.expect("Command not found") {
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
