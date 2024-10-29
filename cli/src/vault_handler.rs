use std::str::FromStr;

use anyhow::{anyhow, Result};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_restaking_core::operator_vault_ticket::OperatorVaultTicket;
use jito_vault_client::{
    instructions::{
        AddDelegationBuilder, BurnWithdrawalTicketBuilder, CloseVaultUpdateStateTrackerBuilder,
        CrankVaultUpdateStateTrackerBuilder, CreateTokenMetadataBuilder, EnqueueWithdrawalBuilder,
        InitializeConfigBuilder, InitializeVaultBuilder, InitializeVaultOperatorDelegationBuilder,
        InitializeVaultUpdateStateTrackerBuilder, MintToBuilder, SetDepositCapacityBuilder,
    },
    types::WithdrawalAllocationMethod,
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::inline_mpl_token_metadata;
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

use crate::{
    vault::{ConfigActions, VaultActions, VaultCommands},
    CliConfig,
};

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

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }

    pub async fn handle(&self, action: VaultCommands) -> Result<()> {
        match action {
            VaultCommands::Config {
                action:
                    ConfigActions::Initialize {
                        program_fee_bps,
                        program_fee_wallet,
                    },
            } => {
                self.initialize_config(program_fee_bps, program_fee_wallet)
                    .await
            }
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
                        decimals,
                    },
            } => {
                self.initialize_vault(
                    token_mint,
                    deposit_fee_bps,
                    withdrawal_fee_bps,
                    reward_fee_bps,
                    decimals,
                )
                .await
            }
            VaultCommands::Vault {
                action:
                    VaultActions::CreateTokenMetadata {
                        vault,
                        name,
                        symbol,
                        uri,
                    },
            } => self.create_token_metadata(vault, name, symbol, uri).await,
            VaultCommands::Vault {
                action: VaultActions::InitializeVaultUpdateStateTracker { vault },
            } => self.initialize_vault_update_state_tracker(vault).await,
            VaultCommands::Vault {
                action: VaultActions::CrankVaultUpdateStateTracker { vault, operator },
            } => self.crank_vault_update_state_tracker(vault, operator).await,
            VaultCommands::Vault {
                action: VaultActions::CloseVaultUpdateStateTracker { vault, ncn_epoch },
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
                action:
                    VaultActions::CooldownOperatorDelegation {
                        vault,
                        operator,
                        amount,
                    },
            } => {
                self.cooldown_operator_delegation(vault, operator, amount)
                    .await
            }
            VaultCommands::Vault {
                action: VaultActions::EnqueueWithdrawal { vault, amount },
            } => self.enqueue_withdrawal(vault, amount).await,
            VaultCommands::Vault {
                action: VaultActions::BurnWithdrawalTicket { vault },
            } => self.burn_withdrawal_ticket(vault).await,
            VaultCommands::Vault {
                action: VaultActions::GetVaultUpdateStateTracker { vault, ncn_epoch },
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
            VaultCommands::Vault {
                action: VaultActions::SetCapacity { vault, amount },
            } => self.set_capacity(vault, amount).await,
        }
    }

    pub async fn initialize_config(
        &self,
        program_fee_bps: u16,
        program_fee_wallet: Pubkey,
    ) -> Result<()> {
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
            .restaking_program(self.restaking_program_id)
            .program_fee_wallet(program_fee_wallet)
            .program_fee_bps(program_fee_bps);

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
        info!("Vault config initialized at address: {}", config_address);
        Ok(())
    }

    pub async fn initialize_vault(
        &self,
        token_mint: String,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        decimals: u8,
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
            .reward_fee_bps(reward_fee_bps)
            .decimals(decimals);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair, &base, &vrt_mint],
            blockhash,
        );
        info!("Initializing vault transaction: {:?}", tx.get_signature());
        rpc_client.send_and_confirm_transaction(&tx).await?;
        info!("Transaction confirmed: {:?}", tx.get_signature());
        info!("\nCreated new vault");
        info!("Vault address: {}", vault);
        info!("Base address: {}", base.pubkey());
        info!("VRT mint address: {}", vrt_mint.pubkey());
        info!("Token mint address: {}", token_mint);

        Ok(())
    }

    async fn create_token_metadata(
        &self,
        vault: String,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let vault_pubkey = Pubkey::from_str(&vault)?;

        let rpc_client = self.get_rpc_client();
        let vault_account = rpc_client.get_account(&vault_pubkey).await?;
        let vault = Vault::try_from_slice_unchecked(&vault_account.data)?;

        let metadata = Pubkey::find_program_address(
            &[
                b"metadata",
                inline_mpl_token_metadata::id().as_ref(),
                vault.vrt_mint.as_ref(),
            ],
            &inline_mpl_token_metadata::id(),
        )
        .0;

        let ix = CreateTokenMetadataBuilder::new()
            .vault(vault_pubkey)
            .admin(keypair.pubkey())
            .vrt_mint(vault.vrt_mint)
            .payer(keypair.pubkey())
            .metadata(metadata)
            .name(name)
            .symbol(symbol)
            .uri(uri)
            .instruction();

        let recent_blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[keypair],
            recent_blockhash,
        );

        info!(
            "Creating token metadata transaction: {:?}",
            tx.get_signature()
        );
        rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }

    // ---------- UPDATE ------------

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
        let ncn_epoch = current_slot.checked_div(epoch_length).unwrap();

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

    pub async fn crank_vault_update_state_tracker(
        &self,
        vault: String,
        operator: String,
    ) -> Result<()> {
        //TODO V2: Make it so the operator needed is automatically fetched from the vault

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

        let ncn_epoch = {
            let config_account_raw = rpc_client.get_account(&config).await?;
            let config_account = Config::try_from_slice_unchecked(&config_account_raw.data)?;

            let current_slot = rpc_client.get_slot().await?;
            let epoch_length = config_account.epoch_length();
            current_slot.checked_div(epoch_length).unwrap()
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
                current_slot.checked_div(epoch_length).unwrap()
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

    // ---------- FUNCTIONS --------------
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
        info!("Mint to transaction: {:?}", tx.get_signature());
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
        info!("Delegating to operator: {:?}", tx.get_signature());
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());
        info!("Delegated {} tokens to {}", amount, operator);

        Ok(())
    }

    pub async fn cooldown_operator_delegation(
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

        let mut ix_builder = CooldownDelegationBuilder::new();
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
        info!("Cooling down delegation: {:?}", tx.get_signature());
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());
        info!("Cooldown {} tokens for {}", amount, operator);

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

    pub async fn burn_withdrawal_ticket(&self, vault: String) -> Result<()> {
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

        let mut ix_builder = BurnWithdrawalTicketBuilder::new();
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

    // ------- GET ACCOUNTS --------------------
    pub async fn get_vault(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let rpc_client = self.get_rpc_client();
        let account = rpc_client.get_account(&pubkey).await?;
        let vault = Vault::try_from_slice_unchecked(&account.data)?;
        info!("vault at address {}: {:?}", pubkey, vault);
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

    pub async fn set_capacity(&self, vault: String, amount: u64) -> Result<()> {
        let keypair = self
            .cli_config
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let vault_pubkey = Pubkey::from_str(&vault)?;
        let rpc_client = self.get_rpc_client();

        let mut builder = SetDepositCapacityBuilder::new();
        builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault_pubkey)
            .admin(keypair.pubkey())
            .amount(amount);

        let recent_blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[builder.instruction()],
            Some(&keypair.pubkey()),
            &[keypair],
            recent_blockhash,
        );

        info!("Vault capacity instruction: {:?}", builder);
        info!(
            "Vault capacity transaction signature: {:?}",
            tx.get_signature()
        );
        rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        info!("Transaction confirmed: {:?}", tx.get_signature());

        Ok(())
    }
}
