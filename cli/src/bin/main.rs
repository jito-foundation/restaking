use std::str::FromStr;

use clap::Parser;
use clap_markdown::MarkdownOptions;
use env_logger::Env;
use jito_restaking_cli::{
    cli_args::{Cli, ProgramCommand},
    cli_config::CliConfig,
    restaking_handler::RestakingCliHandler,
    vault_handler::VaultCliHandler,
};
use jito_restaking_client::programs::JITO_RESTAKING_ID;
use jito_vault_client::programs::JITO_VAULT_ID;
use solana_sdk::pubkey::Pubkey;

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

    let cli_config = CliConfig::new(&args)?;

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
