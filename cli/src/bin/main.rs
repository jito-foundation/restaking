use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use clap_markdown::MarkdownOptions;
use env_logger::Env;
use jito_restaking_cli::{
    cli_args::{Cli, ProgramCommand},
    cli_config::CliConfig,
    cli_signer::CliSigner,
    restaking_handler::RestakingCliHandler,
    vault_handler::VaultCliHandler,
};
use jito_restaking_client::programs::JITO_RESTAKING_ID;
use jito_vault_client::programs::JITO_VAULT_ID;
use solana_cli_config::Config;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

pub fn get_cli_config(args: &Cli) -> Result<CliConfig, anyhow::Error> {
    let cli_config = if let Some(config_file) = &args.config_file {
        let config = Config::load(config_file.as_os_str().to_str().unwrap())?;
        let signer = if let Some(keypair_path) = &args.signer {
            if keypair_path.starts_with("usb://") {
                CliSigner::new_ledger(keypair_path)
            } else {
                CliSigner::new_keypair_from_path(keypair_path)?
            }
        } else {
            CliSigner::new_keypair_from_path(&config.keypair_path)?
        };

        CliConfig {
            rpc_url: config.json_rpc_url,
            commitment: CommitmentConfig::from_str(&config.commitment)?,
            signer: Some(signer),
        }
    } else {
        let config_file = solana_cli_config::CONFIG_FILE
            .as_ref()
            .ok_or_else(|| anyhow!("unable to get config file path"))?;
        if let Ok(config) = Config::load(config_file) {
            let signer = if let Some(keypair_path) = &args.signer {
                if keypair_path.starts_with("usb://") {
                    CliSigner::new_ledger(keypair_path)
                } else {
                    CliSigner::new_keypair_from_path(keypair_path)?
                }
            } else {
                CliSigner::new_keypair_from_path(&config.keypair_path)?
            };

            let rpc = if let Some(rpc) = &args.rpc_url {
                rpc.to_string()
            } else {
                config.json_rpc_url
            };

            CliConfig {
                rpc_url: rpc,
                commitment: CommitmentConfig::from_str(&config.commitment)?,
                signer: Some(signer),
            }
        } else {
            let signer = match args.signer.as_ref() {
                Some(keypair_path) => {
                    let signer = if keypair_path.starts_with("usb://") {
                        CliSigner::new_ledger(keypair_path)
                    } else {
                        CliSigner::new_keypair_from_path(keypair_path)?
                    };
                    Some(signer)
                }
                None => None,
            };
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
                signer,
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
        let markdown = clap_markdown::help_markdown_custom::<Cli>(
            &MarkdownOptions::new().show_table_of_contents(false),
        );
        println!("---");
        println!("title: CLI");
        println!("category: Jekyll");
        println!("layout: post");
        println!("weight: 1");
        println!("---");
        println!();
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
            RestakingCliHandler::new(
                cli_config,
                restaking_program_id,
                vault_program_id,
                args.print_tx,
                args.print_json,
                args.print_json_with_reserves,
            )
            .handle(action)
            .await?;
        }
        ProgramCommand::Vault { action } => {
            VaultCliHandler::new(
                cli_config,
                restaking_program_id,
                vault_program_id,
                args.print_tx,
                args.print_json,
                args.print_json_with_reserves,
            )
            .handle(action)
            .await?;
        }
    }

    Ok(())
}
