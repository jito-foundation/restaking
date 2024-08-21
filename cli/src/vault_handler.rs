use anyhow::anyhow;
use clap::Subcommand;
use jito_bytemuck::AccountDeserialize;
use jito_vault_client::instructions::InitializeConfigBuilder;
use jito_vault_core::config::Config;
use log::{debug, info};
use solana_program::pubkey::Pubkey;
use solana_rpc_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_sdk::{signature::Signer, transaction::Transaction};

use crate::cli_args::CliConfig;

#[derive(Subcommand)]
pub enum VaultCommands {
    Config {
        #[command(subcommand)]
        action: VaultConfigActions,
    },
}

#[derive(Subcommand)]
pub enum VaultConfigActions {
    Initialize,
    Get,
}

pub struct VaultCliHandler {
    cli_config: CliConfig,
    restaking_program_id: Pubkey,
    vault_program_id: Pubkey,
}

impl VaultCliHandler {
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

    pub async fn handle(&self, action: VaultCommands) -> Result<(), anyhow::Error> {
        match action {
            VaultCommands::Config { action } => self.handle_vault_config(action).await,
        }
    }

    async fn handle_vault_config(&self, args: VaultConfigActions) -> Result<(), anyhow::Error> {
        match args {
            VaultConfigActions::Initialize => {
                let keypair = self
                    .cli_config
                    .keypair
                    .as_ref()
                    .ok_or_else(|| anyhow!("Keypair not provided"))?;
                let rpc_client = self.get_rpc_client();

                let mut ix_builder = InitializeConfigBuilder::new();
                let config_address = Config::find_program_address(&self.vault_program_id).0;
                let ix_builder = ix_builder
                    .config(config_address)
                    .admin(keypair.pubkey())
                    .restaking_program(self.restaking_program_id);

                let blockhash = rpc_client.get_latest_blockhash().await?;
                let tx = Transaction::new_signed_with_payer(
                    &[ix_builder.instruction()],
                    Some(&keypair.pubkey()),
                    &[keypair],
                    blockhash,
                );
                info!("Initializing vault config parameters: {:?}", ix_builder);
                info!(
                    "Initializing vault config transaction: {:?}",
                    tx.get_signature()
                );
                rpc_client.send_and_confirm_transaction(&tx).await?;
                info!("Transaction confirmed: {:?}", tx.get_signature());
            }
            VaultConfigActions::Get => {
                let rpc_client = self.get_rpc_client();

                let config_address = Config::find_program_address(&self.vault_program_id).0;
                debug!(
                    "Reading the restaking configuration account at address: {}",
                    config_address
                );

                let account = rpc_client.get_account(&config_address).await?;
                let config = Config::try_from_slice_unchecked(&account.data)?;
                info!("Vault config at address {} : {:?}", config_address, config);
            }
        }
        Ok(())
    }

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }
}
