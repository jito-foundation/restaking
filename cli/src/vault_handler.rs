use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::Subcommand;
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_restaking_core::operator_vault_ticket::OperatorVaultTicket;
use jito_vault_client::{
    instructions::{
        AddDelegationBuilder, BurnWithdrawTicketBuilder, CloseVaultUpdateStateTrackerBuilder,
        CrankVaultUpdateStateTrackerBuilder, EnqueueWithdrawalBuilder, InitializeConfigBuilder,
        InitializeVaultBuilder, InitializeVaultOperatorDelegationBuilder,
        InitializeVaultUpdateStateTrackerBuilder, MintToBuilder,
    },
    types::WithdrawalAllocationMethod,
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
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
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};

use crate::cli_args::CliConfig;

#[derive(Subcommand)]
pub enum VaultCommands {
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
    Vault {
        #[command(subcommand)]
        action: VaultActions,
    },
}

#[derive(Subcommand)]
pub enum ConfigActions {
    Initialize,
    Get,
}

/// Vault commands
#[derive(Subcommand)]
pub enum VaultActions {
    /// Initializes the vault
    Initialize {
        /// The token which is allowed to be deposited into the vault
        token_mint: String,
        /// The deposit fee in bips
        deposit_fee_bps: u16,
        /// The withdrawal fee in bips
        withdrawal_fee_bps: u16,
        /// The reward fee in bips
        reward_fee_bps: u16,
    },
    InitializeUpdateStateTracker {
        /// Vault account
        vault: String,
    },
    CloseUpdateStateTracker {
        /// Vault account
        vault: String,
        /// Optional NCN epoch to close
        ncn_epoch: Option<u64>,
    },
    MintVRT {
        /// Vault account
        vault: String,
        /// Amount to deposit
        amount_in: u64,
        /// Minimum amount of VRT to mint
        min_amount_out: u64,
    },
    InitializeOperatorDelegation {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
    },
    DelegateToOperator {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
        /// Amount to delegate
        amount: u64,
    },
    EnqueueWithdrawal {
        /// Vault account
        vault: String,
        /// Amount to withdraw
        amount: u64,
    },
    CrankUpdateStateTracker {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
        /// NCN epoch to crank
        ncn_epoch: Option<u64>,
    },
    BurnWithdrawalTicket {
        /// Vault account
        vault: String,
        /// Minimum amount of VRT to mint
        min_amount_out: u64,
    },
    GetStateTracker {
        /// Vault account
        vault: String,
        /// NCN epoch
        ncn_epoch: u64,
    },
    GetOperatorDelegation {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
    },
    GetWithdrawalTicket {
        /// Vault account
        vault: String,
        /// Staker account
        staker: Option<String>,
    },
    /// Gets a vault
    Get {
        /// The vault pubkey
        pubkey: String,
    },
    /// List all vaults
    List,
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

    pub async fn handle(&self, action: VaultCommands) -> Result<()> {
        match action {
            VaultCommands::Config {
                action: ConfigActions::Initialize,
            } => self.initialize_config().await,
            VaultCommands::Config {
                action: ConfigActions::Get,
            } => self.get_config().await,
            VaultCommands::Vault {
                action:
                    VaultActions::Initialize {
                        token_mint,
                        deposit_fee_bps,
                        withdrawal_fee_bps,
                        reward_fee_bps,
                    },
            } => {
                self.initialize_vault(
                    token_mint,
                    deposit_fee_bps,
                    withdrawal_fee_bps,
                    reward_fee_bps,
                )
                .await
            }
            VaultCommands::Vault {
                action: VaultActions::InitializeUpdateStateTracker { vault },
            } => self.initialize_vault_update_state_tracker(vault).await,
            VaultCommands::Vault {
                action: VaultActions::CloseUpdateStateTracker { vault, ncn_epoch },
            } => {
                self.close_vault_update_state_tracker(vault, ncn_epoch)
                    .await
            }
            VaultCommands::Vault {
                action:
                    VaultActions::MintVRT {
                        vault,
                        amount_in,
                        min_amount_out,
                    },
            } => self.mint_vrt(vault, amount_in, min_amount_out).await,
            VaultCommands::Vault {
                action: VaultActions::InitializeOperatorDelegation { vault, operator },
            } => {
                self.initialize_vault_operator_delegation(vault, operator)
                    .await
            }
            VaultCommands::Vault {
                action:
                    VaultActions::DelegateToOperator {
                        vault,
                        operator,
                        amount,
                    },
            } => self.delegate_to_operator(vault, operator, amount).await,

            VaultCommands::Vault {
                action: VaultActions::EnqueueWithdrawal { vault, amount },
            } => self.enqueue_withdrawal(vault, amount).await,
            VaultCommands::Vault {
                action:
                    VaultActions::CrankUpdateStateTracker {
                        vault,
                        operator,
                        ncn_epoch,
                    },
            } => {
                self.crank_vault_update_state_tracker(vault, operator, ncn_epoch)
                    .await
            }
            VaultCommands::Vault {
                action:
                    VaultActions::BurnWithdrawalTicket {
                        vault,
                        min_amount_out,
                    },
            } => self.burn_withdrawal_ticket(vault, min_amount_out).await,
            VaultCommands::Vault {
                action: VaultActions::GetStateTracker { vault, ncn_epoch },
            } => self.get_vault_update_state_tracker(vault, ncn_epoch).await,
            VaultCommands::Vault {
                action: VaultActions::GetOperatorDelegation { vault, operator },
            } => self.get_vault_operator_delegation(vault, operator).await,
            VaultCommands::Vault {
                action: VaultActions::GetWithdrawalTicket { vault, staker },
            } => self.get_withdrawal_ticket(vault, staker).await,
            VaultCommands::Vault {
                action: VaultActions::Get { pubkey },
            } => self.get_vault(pubkey).await,
            VaultCommands::Vault {
                action: VaultActions::List,
            } => self.list_vaults().await,
        }
    }

    pub async fn initialize_config(&self) -> Result<()> {
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
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());
        Ok(())
    }

    async fn get_config(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        debug!(
            "Reading the restaking configuration account at address: {}",
            config_address
        );

        let account = rpc_client.get_account(&config_address).await?;
        let config = Config::try_from_slice_unchecked(&account.data)?;
        info!("Vault config at address {} : {:?}", config_address, config);
        Ok(())
    }

    pub async fn initialize_vault(
        &self,
        token_mint: String,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
    ) -> Result<()> {
        let token_mint = Pubkey::from_str(&token_mint)?;
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let base = Keypair::new();
        let vault = Vault::find_program_address(&self.vault_program_id, &base.pubkey()).0;

        let vrt_mint = Keypair::new();

        let mut ix_builder = InitializeVaultBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .vrt_mint(vrt_mint.pubkey())
            .token_mint(token_mint)
            .admin(keypair.pubkey())
            .base(base.pubkey())
            .deposit_fee_bps(deposit_fee_bps)
            .withdrawal_fee_bps(withdrawal_fee_bps)
            .reward_fee_bps(reward_fee_bps);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair, &base, &vrt_mint],
            blockhash,
        );
        info!("Initializing vault transaction: {:?}", tx.get_signature());
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }
        info!("Transaction confirmed: {:?}", tx.get_signature());

        info!("\nCreated new vault");
        info!("Vault address: {}", vault);
        info!("Base address: {}", base.pubkey());
        info!("VRT mint address: {}", vrt_mint.pubkey());
        info!("Token mint address: {}", token_mint);

        Ok(())
    }

    pub async fn initialize_vault_update_state_tracker(&self, vault: String) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let config = Config::find_program_address(&self.vault_program_id).0;

        let config_account_raw = rpc_client.get_account(&config).await?;
        let config_account = Config::try_from_slice_unchecked(&config_account_raw.data)?;

        let current_slot = rpc_client.get_slot().await?;
        let epoch_length = config_account.epoch_length();
        let ncn_epoch = current_slot / epoch_length;

        let vault = Pubkey::from_str(&vault)?;
        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &self.vault_program_id,
            &vault,
            ncn_epoch,
        )
        .0;

        let mut ix_builder = InitializeVaultUpdateStateTrackerBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .vault_update_state_tracker(vault_update_state_tracker)
            .payer(keypair.pubkey())
            .withdrawal_allocation_method(WithdrawalAllocationMethod::Greedy); // Only withdrawal allocation method supported for now

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Initializing vault update state tracker transaction: {:?}",
            tx.get_signature()
        );

        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());

        info!("\nCreated Update State Tracker");
        info!("Vault address: {}", vault);
        info!(
            "Vault Update State Tracker address: {}",
            vault_update_state_tracker
        );
        info!("NCN Epoch: {}", ncn_epoch);

        Ok(())
    }

    pub async fn close_vault_update_state_tracker(
        &self,
        vault: String,
        ncn_epoch: Option<u64>,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let config = Config::find_program_address(&self.vault_program_id).0;

        let ncn_epoch = match ncn_epoch {
            Some(ncn_epoch) => ncn_epoch,
            None => {
                let config_account_raw = rpc_client.get_account(&config).await?;
                let config_account = Config::try_from_slice_unchecked(&config_account_raw.data)?;

                let current_slot = rpc_client.get_slot().await?;
                let epoch_length = config_account.epoch_length();
                current_slot / epoch_length
            }
        };

        let vault = Pubkey::from_str(&vault)?;
        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &self.vault_program_id,
            &vault,
            ncn_epoch,
        )
        .0;

        let mut ix_builder = CloseVaultUpdateStateTrackerBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .vault_update_state_tracker(vault_update_state_tracker)
            .ncn_epoch(ncn_epoch)
            .payer(keypair.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Closing vault update state tracker transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }
        info!("Transaction confirmed: {:?}", tx.get_signature());

        info!("\nClose Update State Tracker");
        Ok(())
    }

    pub async fn mint_vrt(&self, vault: String, amount_in: u64, min_amount_out: u64) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;

        let vault_account_raw = rpc_client.get_account(&vault).await?;
        let vault_account = Vault::try_from_slice_unchecked(&vault_account_raw.data)?;

        let depositor = keypair.pubkey();
        let depositor_token_account =
            get_associated_token_address(&depositor, &vault_account.supported_mint);
        let depositor_vrt_token_account =
            get_associated_token_address(&depositor, &vault_account.vrt_mint);

        let vault_token_account =
            get_associated_token_address(&vault, &vault_account.supported_mint);

        let vault_fee_token_account =
            get_associated_token_address(&vault_account.fee_wallet, &vault_account.vrt_mint);

        let depositor_ata_ix = create_associated_token_account_idempotent(
            &depositor,
            &depositor,
            &vault_account.supported_mint,
            &spl_token::ID,
        );
        let depositor_vrt_ata_ix = create_associated_token_account_idempotent(
            &depositor,
            &depositor,
            &vault_account.vrt_mint,
            &spl_token::ID,
        );
        let vault_ata_ix = create_associated_token_account_idempotent(
            &depositor,
            &vault,
            &vault_account.supported_mint,
            &spl_token::ID,
        );
        let vault_fee_ata_ix = create_associated_token_account_idempotent(
            &depositor,
            &vault_account.fee_wallet,
            &vault_account.supported_mint,
            &spl_token::ID,
        );

        let mut ix_builder = MintToBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vrt_mint(vault_account.vrt_mint)
            .depositor(depositor)
            .depositor_token_account(depositor_token_account)
            .depositor_vrt_token_account(depositor_vrt_token_account)
            .vault_token_account(vault_token_account)
            .vault_fee_token_account(vault_fee_token_account)
            .amount_in(amount_in)
            .min_amount_out(min_amount_out)
            .vault(vault);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[
                depositor_ata_ix,
                depositor_vrt_ata_ix,
                vault_ata_ix,
                vault_fee_ata_ix,
                ix_builder.instruction(),
            ],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Initializing vault update state tracker transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());

        info!("\nMinted VRT");

        Ok(())
    }

    pub async fn initialize_vault_operator_delegation(
        &self,
        vault: String,
        operator: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;
        let operator = Pubkey::from_str(&operator)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
            &self.vault_program_id,
            &vault,
            &operator,
        )
        .0;

        let mut ix_builder = InitializeVaultOperatorDelegationBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .operator(operator)
            .operator_vault_ticket(operator_vault_ticket)
            .vault_operator_delegation(vault_operator_delegation)
            .payer(keypair.pubkey())
            .admin(keypair.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Initializing vault operator delegation transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }

    pub async fn delegate_to_operator(
        &self,
        vault: String,
        operator: String,
        amount: u64,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;
        let operator = Pubkey::from_str(&operator)?;

        let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
            &self.vault_program_id,
            &vault,
            &operator,
        )
        .0;

        let mut ix_builder = AddDelegationBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .operator(operator)
            .vault_operator_delegation(vault_operator_delegation)
            .admin(keypair.pubkey())
            .amount(amount);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Initializing vault operator delegation transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }

    pub async fn enqueue_withdrawal(&self, vault: String, amount: u64) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;
        let vault_account_raw = rpc_client.get_account(&vault).await?;
        let vault_account = Vault::try_from_slice_unchecked(&vault_account_raw.data)?;

        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &self.vault_program_id,
            &vault,
            &keypair.pubkey(),
        )
        .0;

        let vault_staker_withdrawal_ticket_token_account =
            get_associated_token_address(&vault_staker_withdrawal_ticket, &vault_account.vrt_mint);

        let staker_vrt_token_account =
            get_associated_token_address(&keypair.pubkey(), &vault_account.vrt_mint);

        let vault_staker_withdrawal_ticket_ata_ix = create_associated_token_account_idempotent(
            &keypair.pubkey(),
            &vault_staker_withdrawal_ticket,
            &vault_account.vrt_mint,
            &spl_token::ID,
        );

        let mut ix_builder = EnqueueWithdrawalBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .vault_staker_withdrawal_ticket(vault_staker_withdrawal_ticket)
            .vault_staker_withdrawal_ticket_token_account(
                vault_staker_withdrawal_ticket_token_account,
            )
            .staker(keypair.pubkey())
            .staker_vrt_token_account(staker_vrt_token_account)
            .base(keypair.pubkey())
            .amount(amount);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[
                vault_staker_withdrawal_ticket_ata_ix,
                ix_builder.instruction(),
            ],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Initializing vault operator delegation transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }

    pub async fn crank_vault_update_state_tracker(
        &self,
        vault: String,
        operator: String,
        ncn_epoch: Option<u64>,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let config = Config::find_program_address(&self.vault_program_id).0;

        let vault = Pubkey::from_str(&vault)?;
        let operator = Pubkey::from_str(&operator)?;

        let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
            &self.vault_program_id,
            &vault,
            &operator,
        )
        .0;

        let ncn_epoch = match ncn_epoch {
            Some(ncn_epoch) => ncn_epoch,
            None => {
                let config_account_raw = rpc_client.get_account(&config).await?;
                let config_account = Config::try_from_slice_unchecked(&config_account_raw.data)?;

                let current_slot = rpc_client.get_slot().await?;
                let epoch_length = config_account.epoch_length();
                current_slot / epoch_length
            }
        };

        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &self.vault_program_id,
            &vault,
            ncn_epoch,
        )
        .0;

        let mut ix_builder = CrankVaultUpdateStateTrackerBuilder::new();
        ix_builder
            .config(config)
            .vault(vault)
            .operator(operator)
            .vault_operator_delegation(vault_operator_delegation)
            .vault_update_state_tracker(vault_update_state_tracker);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Cranking vault update state tracker: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }

    pub async fn burn_withdrawal_ticket(&self, vault: String, min_amount_out: u64) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;
        let vault_account_raw = rpc_client.get_account(&vault).await?;
        let vault_account = Vault::try_from_slice_unchecked(&vault_account_raw.data)?;

        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &self.vault_program_id,
            &vault,
            &keypair.pubkey(),
        )
        .0;

        let staker = keypair.pubkey();
        let staker_token_account =
            get_associated_token_address(&staker, &vault_account.supported_mint);

        let vault_token_account =
            get_associated_token_address(&vault, &vault_account.supported_mint);

        let vault_fee_token_account =
            get_associated_token_address(&vault_account.fee_wallet, &vault_account.vrt_mint);

        let vault_staker_withdrawal_ticket_token_account =
            get_associated_token_address(&vault_staker_withdrawal_ticket, &vault_account.vrt_mint);

        let mut ix_builder = BurnWithdrawTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vrt_mint(vault_account.vrt_mint)
            .vault(vault)
            .vault_staker_withdrawal_ticket(vault_staker_withdrawal_ticket)
            .vault_staker_withdrawal_ticket_token_account(
                vault_staker_withdrawal_ticket_token_account,
            )
            .staker_token_account(staker_token_account)
            .vault_fee_token_account(vault_fee_token_account)
            .vault_token_account(vault_token_account)
            .min_amount_out(min_amount_out)
            .staker(staker);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );
        info!(
            "Initializing vault operator delegation transaction: {:?}",
            tx.get_signature()
        );
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }

    pub async fn get_vault(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let rpc_client = self.get_rpc_client();
        let account = rpc_client.get_account(&pubkey).await?;
        let vault = Vault::try_from_slice_unchecked(&account.data)?;
        info!("vault at address {}: {:?}", pubkey, vault);
        Ok(())
    }

    pub async fn get_vault_update_state_tracker(
        &self,
        vault: String,
        ncn_epoch: u64,
    ) -> Result<()> {
        let vault = Pubkey::from_str(&vault)?;
        let rpc_client = self.get_rpc_client();
        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &self.vault_program_id,
            &vault,
            ncn_epoch,
        )
        .0;
        let account = rpc_client.get_account(&vault_update_state_tracker).await?;
        let state_tracker = VaultUpdateStateTracker::try_from_slice_unchecked(&account.data)?;
        info!("{:?}", state_tracker);
        Ok(())
    }

    pub async fn get_vault_operator_delegation(
        &self,
        vault: String,
        operator: String,
    ) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let vault = Pubkey::from_str(&vault)?;
        let operator = Pubkey::from_str(&operator)?;
        let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
            &self.vault_program_id,
            &vault,
            &operator,
        )
        .0;
        let account = rpc_client.get_account(&vault_operator_delegation).await?;
        let delegation = VaultOperatorDelegation::try_from_slice_unchecked(&account.data)?;
        info!("{:?}", delegation);
        Ok(())
    }

    pub async fn get_withdrawal_ticket(&self, vault: String, staker: Option<String>) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let vault = Pubkey::from_str(&vault)?;
        let staker = if let Some(staker) = staker {
            Pubkey::from_str(&staker)?
        } else {
            let keypair = self
                .cli_config
                .keypair
                .as_ref()
                .ok_or_else(|| anyhow!("Keypair not provided"))?;
            keypair.pubkey()
        };
        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &self.vault_program_id,
            &vault,
            &staker,
        )
        .0;
        let account = rpc_client
            .get_account(&vault_staker_withdrawal_ticket)
            .await?;
        let ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(&account.data)?;
        info!("{:?}", ticket);
        Ok(())
    }

    pub async fn list_vaults(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let accounts = rpc_client
            .get_program_accounts_with_config(
                &self.vault_program_id,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new(
                        0,
                        MemcmpEncodedBytes::Bytes(vec![Vault::DISCRIMINATOR]),
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
        for (vault_pubkey, vault) in accounts {
            let vault = Vault::try_from_slice_unchecked(&vault.data)?;
            info!("vault at address {}: {:?}", vault_pubkey, vault);
        }
        Ok(())
    }

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }
}
