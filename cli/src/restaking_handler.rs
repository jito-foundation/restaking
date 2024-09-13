use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::Subcommand;
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_restaking_client::instructions::{
    InitializeConfigBuilder, InitializeNcnBuilder, InitializeOperatorBuilder,
    InitializeOperatorVaultTicketBuilder, WarmupOperatorVaultTicketBuilder,
};
use jito_restaking_core::{
    config::Config, ncn::Ncn, operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use log::{debug, info};
use solana_account_decoder::UiAccountEncoding;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::cli_args::CliConfig;

/// The CLI handler for the restaking program
#[derive(Subcommand)]
pub enum RestakingCommands {
    /// Initialize, get, and set the config struct
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
    Ncn {
        #[command(subcommand)]
        action: NcnActions,
    },
    Operator {
        #[command(subcommand)]
        action: OperatorActions,
    },
}

/// The actions that can be performed on the restaking config
#[derive(Subcommand)]
pub enum ConfigActions {
    /// Initialize the config
    Initialize,
    /// Get the config
    Get,
}

#[derive(Subcommand)]
pub enum NcnActions {
    /// Initialize NCN
    Initialize,
    /// Get NCN
    Get { pubkey: String },
    /// List all NCNs
    List,
}

#[derive(Subcommand)]
pub enum OperatorActions {
    /// Initialize Operator
    Initialize,
    /// Initialize Operator Vault Ticket
    InitializeOperatorVaultTicket { operator: String, vault: String },
    /// Warmup Operator Vault Ticket
    WarmupOperatorVaultTicket { operator: String, vault: String },
    /// Get operator
    Get { pubkey: String },
    /// List all operators
    List,
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

    pub async fn handle(&self, action: RestakingCommands) -> Result<()> {
        match action {
            RestakingCommands::Config {
                action: ConfigActions::Initialize,
            } => self.initialize_config().await,
            RestakingCommands::Config {
                action: ConfigActions::Get,
            } => self.get_config().await,
            RestakingCommands::Ncn {
                action: NcnActions::Initialize,
            } => self.initialize_ncn().await,
            RestakingCommands::Ncn {
                action: NcnActions::Get { pubkey },
            } => self.get_ncn(pubkey).await,
            RestakingCommands::Ncn {
                action: NcnActions::List,
            } => self.list_ncn().await,
            RestakingCommands::Operator {
                action: OperatorActions::Initialize,
            } => self.initialize_operator().await,
            RestakingCommands::Operator {
                action: OperatorActions::InitializeOperatorVaultTicket { operator, vault },
            } => self.initialize_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::WarmupOperatorVaultTicket { operator, vault },
            } => self.warmup_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::Get { pubkey },
            } => self.operator_get(pubkey).await,
            RestakingCommands::Operator {
                action: OperatorActions::List,
            } => self.operator_list().await,
        }
    }

    pub async fn get_config(&self) -> Result<()> {
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
        Ok(())
    }

    async fn initialize_config(&self) -> Result<()> {
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
        Ok(())
    }

    pub async fn initialize_ncn(&self) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let base = Keypair::new();
        let ncn = Ncn::find_program_address(&self.restaking_program_id, &base.pubkey()).0;

        let mut ix_builder = InitializeNcnBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .admin(keypair.pubkey())
            .base(base.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair, &base],
            blockhash,
        );
        info!("Initializing NCN: {:?}", ncn);
        info!("Initializing NCN transaction: {:?}", tx.get_signature());
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);
        let statuses = rpc_client
            .get_signature_statuses(&[*tx.get_signature()])
            .await?;

        let tx_status = statuses
            .value
            .first()
            .unwrap()
            .as_ref()
            .ok_or_else(|| anyhow!("No signature status"))?;
        info!("Transaction status: {:?}", tx_status);

        Ok(())
    }

    pub async fn get_ncn(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let account = self.get_rpc_client().get_account(&pubkey).await?;
        let ncn = Ncn::try_from_slice_unchecked(&account.data)?;
        info!("NCN at address {}: {:?}", pubkey, ncn);
        Ok(())
    }

    pub async fn list_ncn(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let accounts = rpc_client
            .get_program_accounts_with_config(
                &self.restaking_program_id,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![Ncn::DISCRIMINATOR]),
                    ))]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        data_slice: None,
                        commitment: None,
                        min_context_slot: None,
                    },
                    with_context: None,
                },
            )
            .await?;
        for (ncn_pubkey, ncn) in accounts {
            let ncn = Ncn::try_from_slice_unchecked(&ncn.data)?;
            info!("NCN at address {}: {:?}", ncn_pubkey, ncn);
        }
        Ok(())
    }

    pub async fn initialize_operator(&self) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("No keypair"))?;
        let rpc_client = self.get_rpc_client();

        let base = Keypair::new();
        let operator = Operator::find_program_address(&self.restaking_program_id, &base.pubkey()).0;

        let mut ix_builder = InitializeOperatorBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .admin(keypair.pubkey())
            .base(base.pubkey())
            .instruction();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair, &base],
            blockhash,
        );
        info!("Initializing operator: {:?}", operator);
        info!(
            "Initializing operator transaction: {:?}",
            tx.get_signature()
        );
        rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed");
        let statuses = rpc_client
            .get_signature_statuses(&[*tx.get_signature()])
            .await?;

        let tx_status = statuses
            .value
            .first()
            .unwrap()
            .as_ref()
            .ok_or_else(|| anyhow!("No signature status"))?;
        info!("Transaction status: {:?}", tx_status);

        info!("Operator initialized");
        info!("Operator: {:?}", operator);
        info!("Base: {:?}", base.pubkey());

        Ok(())
    }

    pub async fn initialize_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = InitializeOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .admin(keypair.pubkey())
            .operator_vault_ticket(operator_vault_ticket)
            .payer(keypair.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        info!(
            "Initializing operator vault ticket transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        info!("\nCreated Operator Vault Ticket");
        info!("Operator address: {}", operator);
        info!("Vault address: {}", vault);
        info!("Operator Vault Ticket address: {}", operator_vault_ticket);

        Ok(())
    }

    pub async fn warmup_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = WarmupOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .operator_vault_ticket(operator_vault_ticket)
            .admin(keypair.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        info!(
            "Warming up operator vault ticket transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", result);

        Ok(())
    }

    pub async fn operator_get(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let account = self.get_rpc_client().get_account(&pubkey).await?;
        let operator = Operator::try_from_slice_unchecked(&account.data)?;
        info!("Operator at address {}: {:?}", pubkey, operator);

        Ok(())
    }

    pub async fn operator_list(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let accounts = rpc_client
            .get_program_accounts_with_config(
                &self.restaking_program_id,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![Operator::DISCRIMINATOR]),
                    ))]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        data_slice: None,
                        commitment: None,
                        min_context_slot: None,
                    },
                    with_context: None,
                },
            )
            .await?;
        for (operator_pubkey, operator) in accounts {
            let operator = Operator::try_from_slice_unchecked(&operator.data)?;
            info!("Operator at address {}: {:?}", operator_pubkey, operator);
        }
        Ok(())
    }

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }
}
