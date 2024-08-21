use anyhow::anyhow;
use clap::Subcommand;
use jito_bytemuck::AccountDeserialize;
use jito_restaking_client::instructions::InitializeConfigBuilder;
use jito_restaking_core::config::Config;
use log::{debug, info};
use solana_program::pubkey::Pubkey;
use solana_rpc_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_sdk::{signature::Signer, transaction::Transaction};

use crate::cli_args::CliConfig;

/// The CLI handler for the restaking program
#[derive(Subcommand)]
pub enum RestakingCommands {
    /// Initialize, get, and set the config struct
    Config {
        #[command(subcommand)]
        action: RestakingConfigActions,
    },
}

/// The actions that can be performed on the restaking config
#[derive(Subcommand)]
pub enum RestakingConfigActions {
    /// Initialize the config
    Initialize,
    /// Get the config
    Get,
}

pub struct RestakingCliHandler {
    cli_config: CliConfig,
    restaking_program_id: Pubkey,
    vault_program_id: Pubkey,
}

impl RestakingCliHandler {
    pub const fn new(
        cli_config: CliConfig,
        restaking_program_id: Pubkey,
        vault_program_id: Pubkey,
    ) -> Self {
        Self {
            cli_config,
            restaking_program_id,
            vault_program_id,
        }
    }

    pub async fn handle(&self, action: RestakingCommands) -> Result<(), anyhow::Error> {
        match action {
            RestakingCommands::Config { action } => self.handle_restaking_config(action).await,
        }
    }
    async fn handle_restaking_config(
        &self,
        args: RestakingConfigActions,
    ) -> Result<(), anyhow::Error> {
        match args {
            RestakingConfigActions::Initialize => {
                let keypair = self
                    .cli_config
                    .keypair
                    .as_ref()
                    .ok_or_else(|| anyhow!("No keypair"))?;
                let rpc_client = self.get_rpc_client();

                let config_address = Config::find_program_address(&self.restaking_program_id).0;
                let mut ix_builder = InitializeConfigBuilder::new();
                ix_builder
                    .config(config_address)
                    .admin(keypair.pubkey())
                    .vault_program(self.vault_program_id);
                let blockhash = rpc_client.get_latest_blockhash().await?;
                let tx = Transaction::new_signed_with_payer(
                    &[ix_builder.instruction()],
                    Some(&keypair.pubkey()),
                    &[keypair],
                    blockhash,
                );
                info!("Initializing restaking config parameters: {:?}", ix_builder);
                info!(
                    "Initializing restaking config transaction: {:?}",
                    tx.get_signature()
                );
                rpc_client.send_and_confirm_transaction(&tx).await?;
                info!("Transaction confirmed: {:?}", tx.get_signature());
            }
            RestakingConfigActions::Get => {
                let rpc_client = self.get_rpc_client();

                let config_address = Config::find_program_address(&self.restaking_program_id).0;
                debug!(
                    "Reading the restaking configuration account at address: {}",
                    config_address
                );

                let account = rpc_client.get_account(&config_address).await?;
                let config = Config::try_from_slice_unchecked(&account.data)?;
                info!(
                    "Restaking config at address {}: {:?}",
                    config_address, config
                );
            }
        }
        Ok(())
    }

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }
}
