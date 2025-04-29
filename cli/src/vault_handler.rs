use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::get_epoch;
use jito_restaking_client_common::log::PrettyDisplay;
use jito_restaking_core::{
    ncn_vault_ticket::NcnVaultTicket, operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_client::{
    instructions::{
        AddDelegationBuilder, BurnWithdrawalTicketBuilder, ChangeWithdrawalTicketOwnerBuilder,
        CloseVaultUpdateStateTrackerBuilder, CooldownDelegationBuilder,
        CooldownVaultNcnTicketBuilder, CrankVaultUpdateStateTrackerBuilder,
        CreateTokenMetadataBuilder, DelegateTokenAccountBuilder, EnqueueWithdrawalBuilder,
        InitializeConfigBuilder, InitializeVaultBuilder, InitializeVaultNcnTicketBuilder,
        InitializeVaultOperatorDelegationBuilder, InitializeVaultUpdateStateTrackerBuilder,
        MintToBuilder, SetAdminBuilder, SetConfigAdminBuilder, SetDepositCapacityBuilder,
        SetFeesBuilder, SetIsPausedBuilder, SetProgramFeeBuilder, SetProgramFeeWalletBuilder,
        SetSecondaryAdminBuilder, UpdateTokenMetadataBuilder, UpdateVaultBalanceBuilder,
        WarmupVaultNcnTicketBuilder,
    },
    types::{VaultAdminRole, WithdrawalAllocationMethod},
};
use jito_vault_core::{
    burn_vault::BurnVault, config::Config, vault::Vault, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::inline_mpl_token_metadata;
use log::{debug, info};
use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::SerializableTransaction;
use solana_sdk::{
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};
use spl_token::instruction::transfer;

use crate::{
    cli_config::CliConfig,
    cli_signer::CliSigner,
    vault::{ConfigActions, VaultActions, VaultCommands},
    CliHandler,
};

pub struct VaultCliHandler {
    /// The configuration of CLI
    cli_config: CliConfig,

    /// The Pubkey of Jito Restaking Program ID
    restaking_program_id: Pubkey,

    /// The Pubkey of Jito Vault Program ID
    vault_program_id: Pubkey,

    /// This will print out the raw TX instead of running it
    print_tx: bool,
}

impl CliHandler for VaultCliHandler {
    fn cli_config(&self) -> &CliConfig {
        &self.cli_config
    }

    fn print_tx(&self) -> bool {
        self.print_tx
    }
}

impl VaultCliHandler {
    pub const fn new(
        cli_config: CliConfig,
        restaking_program_id: Pubkey,
        vault_program_id: Pubkey,
        print_tx: bool,
    ) -> Self {
        Self {
            cli_config,
            restaking_program_id,
            vault_program_id,
            print_tx,
        }
    }

    #[allow(clippy::future_not_send)]
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
            VaultCommands::Config {
                action: ConfigActions::SetAdmin { new_admin },
            } => self.set_config_admin(new_admin).await,
            VaultCommands::Config {
                action: ConfigActions::SetProgramFee { new_fee_bps },
            } => self.set_program_fee(new_fee_bps).await,
            VaultCommands::Config {
                action: ConfigActions::SetProgramFeeWallet { program_fee_wallet },
            } => self.set_program_fee_wallet(&program_fee_wallet).await,
            VaultCommands::Vault {
                action:
                    VaultActions::Initialize {
                        token_mint,
                        deposit_fee_bps,
                        withdrawal_fee_bps,
                        reward_fee_bps,
                        decimals,
                        initialize_token_amount,
                        vrt_mint_address_file_path,
                    },
            } => {
                self.initialize_vault(
                    token_mint,
                    deposit_fee_bps,
                    withdrawal_fee_bps,
                    reward_fee_bps,
                    decimals,
                    initialize_token_amount,
                    vrt_mint_address_file_path,
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
                action:
                    VaultActions::UpdateTokenMetadata {
                        vault,
                        name,
                        symbol,
                        uri,
                    },
            } => self.update_token_metadata(vault, name, symbol, uri).await,
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
                action: VaultActions::InitializeVaultNcnTicket { vault, ncn },
            } => self.initialize_vault_ncn_ticket(vault, ncn).await,
            VaultCommands::Vault {
                action: VaultActions::WarmupVaultNcnTicket { vault, ncn },
            } => self.warmup_vault_ncn_ticket(vault, ncn).await,
            VaultCommands::Vault {
                action: VaultActions::CooldownVaultNcnTicket { vault, ncn },
            } => self.cooldown_vault_ncn_ticket(vault, ncn).await,

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
                action:
                    VaultActions::ChangeWithdrawalTicketOwner {
                        vault,
                        old_ticket_owner_keypair,
                        new_ticket_owner,
                    },
            } => {
                self.change_withdrawal_ticket_owner(
                    &vault,
                    &old_ticket_owner_keypair,
                    &new_ticket_owner,
                )
                .await
            }
            VaultCommands::Vault {
                action: VaultActions::BurnWithdrawalTicket { vault },
            } => self.burn_withdrawal_ticket(vault).await,
            VaultCommands::Vault {
                action: VaultActions::GetVaultUpdateStateTracker { vault },
            } => self.get_vault_update_state_tracker(vault).await,
            VaultCommands::Vault {
                action: VaultActions::GetOperatorDelegations { vault },
            } => self.get_vault_operator_delegations(vault, None).await,
            VaultCommands::Vault {
                action: VaultActions::GetOperatorDelegation { vault, operator },
            } => {
                self.get_vault_operator_delegations(vault, Some(operator))
                    .await
            }
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
                action:
                    VaultActions::SetAdmin {
                        vault,
                        old_admin_keypair,
                        new_admin_keypair,
                    },
            } => {
                self.set_admin(&vault, &old_admin_keypair, &new_admin_keypair)
                    .await
            }
            VaultCommands::Vault {
                action: VaultActions::SetCapacity { vault, amount },
            } => self.set_capacity(vault, amount).await,
            VaultCommands::Vault {
                action:
                    VaultActions::SetFees {
                        vault,
                        deposit_fee_bps,
                        withdrawal_fee_bps,
                        reward_fee_bps,
                    },
            } => {
                self.set_fees(&vault, deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps)
                    .await
            }
            VaultCommands::Vault {
                action: VaultActions::SetIsPaused { vault, set_pause },
            } => self.set_is_paused(&vault, set_pause).await,
            VaultCommands::Vault {
                action:
                    VaultActions::SetSecondaryAdmin {
                        vault,
                        new_admin,
                        set_delegation_admin,
                        set_operator_admin,
                        set_ncn_admin,
                        set_slasher_admin,
                        set_capacity_admin,
                        set_fee_wallet,
                        set_mint_burn_admin,
                        set_delegate_asset_admin,
                        set_fee_admin,
                        set_metadata_admin,
                    },
            } => {
                self.set_secondary_admin(
                    &vault,
                    &new_admin,
                    set_delegation_admin,
                    set_operator_admin,
                    set_ncn_admin,
                    set_slasher_admin,
                    set_capacity_admin,
                    set_fee_wallet,
                    set_mint_burn_admin,
                    set_delegate_asset_admin,
                    set_fee_admin,
                    set_metadata_admin,
                )
                .await
            }
            VaultCommands::Vault {
                action: VaultActions::UpdateVaultBalance { vault },
            } => self.update_vault_balance(&vault).await,
            VaultCommands::Vault {
                action:
                    VaultActions::DelegateTokenAccount {
                        vault,
                        delegate,
                        token_mint,
                        token_account,
                    },
            } => {
                self.delegate_token_account(vault, delegate, token_mint, token_account)
                    .await
            }
            VaultCommands::Vault {
                action:
                    VaultActions::DelegatedTokenTransfer {
                        token_account,
                        recipient_pubkey,
                        amount,
                    },
            } => {
                self.delegated_token_transfer(token_account, recipient_pubkey, amount)
                    .await
            }
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn initialize_config(
        &self,
        program_fee_bps: u16,
        program_fee_wallet: Pubkey,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No Signer"))?;

        let mut ix_builder = InitializeConfigBuilder::new();
        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let ix_builder = ix_builder
            .config(config_address)
            .admin(signer.pubkey())
            .restaking_program(self.restaking_program_id)
            .program_fee_wallet(program_fee_wallet)
            .program_fee_bps(program_fee_bps);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Initializing vault config parameters: {:?}", ix_builder);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Config>(&config_address)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments, clippy::future_not_send)]
    pub async fn initialize_vault(
        &self,
        token_mint: String,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        decimals: u8,
        initialize_token_amount: u64,
        vrt_mint_address_file_path: Option<PathBuf>,
    ) -> Result<()> {
        let token_mint = Pubkey::from_str(&token_mint)?;
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No Signer"))?;

        let admin = signer.pubkey();

        let base_signer = CliSigner::new(Some(Keypair::new()), None);
        let vault = Vault::find_program_address(&self.vault_program_id, &base_signer.pubkey()).0;

        let vrt_mint_signer = match vrt_mint_address_file_path {
            Some(file_path) => {
                let keypair = read_keypair_file(file_path)
                    .map_err(|e| anyhow!("Could not read VRT mint address file path: {e}"))?;
                info!("Found VRT mint address: {}", keypair.pubkey());

                CliSigner::new(Some(keypair), None)
            }
            None => CliSigner::new(Some(Keypair::new()), None),
        };

        let admin_st_token_account = get_associated_token_address(&admin, &token_mint);
        let vault_st_token_account = get_associated_token_address(&vault, &token_mint);

        let (burn_vault, _, _) =
            BurnVault::find_program_address(&self.vault_program_id, &base_signer.pubkey());

        let burn_vault_vrt_token_account =
            get_associated_token_address(&burn_vault, &vrt_mint_signer.pubkey());

        let mut ix_builder = InitializeVaultBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .vrt_mint(vrt_mint_signer.pubkey())
            .st_mint(token_mint)
            .admin(admin)
            .base(base_signer.pubkey())
            .admin_st_token_account(admin_st_token_account)
            .vault_st_token_account(vault_st_token_account)
            .burn_vault(burn_vault)
            .burn_vault_vrt_token_account(burn_vault_vrt_token_account)
            .associated_token_program(spl_associated_token_account::id())
            .deposit_fee_bps(deposit_fee_bps)
            .withdrawal_fee_bps(withdrawal_fee_bps)
            .reward_fee_bps(reward_fee_bps)
            .decimals(decimals)
            .initialize_token_amount(initialize_token_amount);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        let admin_st_token_account_ix =
            create_associated_token_account_idempotent(&admin, &admin, &token_mint, &spl_token::ID);

        let vault_st_token_account_ix =
            create_associated_token_account_idempotent(&admin, &vault, &token_mint, &spl_token::ID);

        info!("Initializing Vault at address: {}", vault);

        let ixs = [admin_st_token_account_ix, vault_st_token_account_ix, ix];
        self.process_transaction(
            &ixs,
            &signer.pubkey(),
            &[signer, &base_signer, &vrt_mint_signer],
        )
        .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Vault>(&vault)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    async fn create_token_metadata(
        &self,
        vault: String,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;
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

        let mut ix = CreateTokenMetadataBuilder::new()
            .vault(vault_pubkey)
            .admin(signer.pubkey())
            .vrt_mint(vault.vrt_mint)
            .payer(signer.pubkey())
            .metadata(metadata)
            .name(name)
            .symbol(symbol)
            .uri(uri)
            .instruction();
        ix.program_id = self.vault_program_id;

        info!("Creating token metadata transaction",);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    async fn update_token_metadata(
        &self,
        vault: String,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
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

        let ix = UpdateTokenMetadataBuilder::new()
            .vault(vault_pubkey)
            .admin(signer.pubkey())
            .vrt_mint(vault.vrt_mint)
            .metadata(metadata)
            .name(name)
            .symbol(symbol)
            .uri(uri)
            .instruction();

        let recent_blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&signer.pubkey()),
            &[signer],
            recent_blockhash,
        );

        info!(
            "Updating token metadata transaction: {:?}",
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
    #[allow(clippy::future_not_send)]
    pub async fn initialize_vault_update_state_tracker(&self, vault: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let config = Config::find_program_address(&self.vault_program_id).0;

        let config_account_raw = rpc_client.get_account(&config).await?;
        let config_account = Config::try_from_slice_unchecked(&config_account_raw.data)?;

        let current_slot = rpc_client.get_slot().await?;

        let ncn_epoch = get_epoch(current_slot, config_account.epoch_length())?;

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
            .payer(signer.pubkey())
            .withdrawal_allocation_method(WithdrawalAllocationMethod::Greedy); // Only withdrawal allocation method supported for now

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&signer.pubkey()),
            &[signer],
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

    #[allow(clippy::future_not_send)]
    pub async fn crank_vault_update_state_tracker(
        &self,
        vault: String,
        operator: String,
    ) -> Result<()> {
        //TODO V2: Make it so the operator needed is automatically fetched from the vault

        let signer = self
            .cli_config
            .signer
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
            get_epoch(current_slot, config_account.epoch_length()).unwrap()
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
            Some(&signer.pubkey()),
            &[signer],
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

    #[allow(clippy::future_not_send)]
    pub async fn close_vault_update_state_tracker(
        &self,
        vault: String,
        ncn_epoch: Option<u64>,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
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
                get_epoch(current_slot, config_account.epoch_length()).unwrap()
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
            .payer(signer.pubkey());

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&signer.pubkey()),
            &[signer],
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
    #[allow(clippy::future_not_send)]
    pub async fn mint_vrt(&self, vault: String, amount_in: u64, min_amount_out: u64) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;

        let vault_account_raw = rpc_client.get_account(&vault).await?;
        let vault_account = Vault::try_from_slice_unchecked(&vault_account_raw.data)?;

        let depositor = signer.pubkey();
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
            &vault_account.vrt_mint,
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
            Some(&signer.pubkey()),
            &[signer],
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

    #[allow(clippy::future_not_send)]
    pub async fn initialize_vault_ncn_ticket(&self, vault: String, ncn: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

        let vault = Pubkey::from_str(&vault)?;
        let ncn = Pubkey::from_str(&ncn)?;

        let (vault_ncn_ticket, _, _) =
            VaultNcnTicket::find_program_address(&self.vault_program_id, &vault, &ncn);

        let (ncn_vault_ticket, _, _) =
            NcnVaultTicket::find_program_address(&self.restaking_program_id, &ncn, &vault);

        let mut ix_builder = InitializeVaultNcnTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .ncn(ncn)
            .vault_ncn_ticket(vault_ncn_ticket)
            .ncn_vault_ticket(ncn_vault_ticket)
            .payer(signer.pubkey())
            .admin(signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Initialize Vault NCN Ticket");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::VaultNcnTicket>(&vault_ncn_ticket)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn warmup_vault_ncn_ticket(&self, vault: String, ncn: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

        let vault = Pubkey::from_str(&vault)?;
        let ncn = Pubkey::from_str(&ncn)?;

        let (vault_ncn_ticket, _, _) =
            VaultNcnTicket::find_program_address(&self.vault_program_id, &vault, &ncn);

        let mut ix_builder = WarmupVaultNcnTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .ncn(ncn)
            .vault_ncn_ticket(vault_ncn_ticket)
            .admin(signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Warmup Vault NCN Ticket");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::VaultNcnTicket>(&vault_ncn_ticket)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn cooldown_vault_ncn_ticket(&self, vault: String, ncn: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

        let vault = Pubkey::from_str(&vault)?;
        let ncn = Pubkey::from_str(&ncn)?;

        let (vault_ncn_ticket, _, _) =
            VaultNcnTicket::find_program_address(&self.restaking_program_id, &vault, &ncn);

        let mut ix_builder = CooldownVaultNcnTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .ncn(ncn)
            .vault_ncn_ticket(vault_ncn_ticket)
            .admin(signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Cooldown Vault NCN Ticket");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::VaultNcnTicket>(&vault_ncn_ticket)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn initialize_vault_operator_delegation(
        &self,
        vault: String,
        operator: String,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

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
            .payer(signer.pubkey())
            .admin(signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Initializing vault operator delegation",);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::VaultOperatorDelegation>(
                    &vault_operator_delegation,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn delegate_to_operator(
        &self,
        vault: String,
        operator: String,
        amount: u64,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

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
            .admin(signer.pubkey())
            .amount(amount);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Delegating to operator");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::VaultOperatorDelegation>(
                    &vault_operator_delegation,
                )
                .await?;
            info!("{}", account.pretty_display());
            info!("Delegated {} tokens to {}", amount, operator);
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn cooldown_operator_delegation(
        &self,
        vault: String,
        operator: String,
        amount: u64,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

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
            .admin(signer.pubkey())
            .amount(amount);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Cooling down delegation");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::VaultOperatorDelegation>(
                    &vault_operator_delegation,
                )
                .await?;
            info!("{}", account.pretty_display());
            info!("Cooldown {} tokens for {}", amount, operator);
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn delegate_token_account(
        &self,
        vault: String,
        delegate: String,
        token_mint: String,
        token_account: String,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;
        let delegate = Pubkey::from_str(&delegate)?;
        let token_mint = Pubkey::from_str(&token_mint)?;
        let token_account = Pubkey::from_str(&token_account)?;

        let mut ix_builder = DelegateTokenAccountBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault)
            .delegate_asset_admin(signer.pubkey())
            .token_mint(token_mint)
            .token_account(token_account)
            .delegate(delegate)
            .token_program(spl_token::ID);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix_builder.instruction()],
            Some(&signer.pubkey()),
            &[signer],
            blockhash,
        );
        info!("Delegating token account: {:?}", tx.get_signature());
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());
        info!("Delegated token account: {:?}", token_account);

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn delegated_token_transfer(
        &self,
        token_account: String,
        recipient_pubkey: String,
        amount: u64,
    ) -> Result<()> {
        let keypair = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let token_account = Pubkey::from_str(&token_account)?;
        let recipient_pubkey = Pubkey::from_str(&recipient_pubkey)?;

        let transfer_ix = transfer(
            &spl_token::id(),
            &token_account,
            &recipient_pubkey,
            &keypair.pubkey(),
            &[],
            amount,
        )?;

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        info!("Delegating token transfer: {:?}", tx.get_signature());
        let result = rpc_client.send_and_confirm_transaction(&tx).await;

        if result.is_err() {
            return Err(anyhow::anyhow!("Transaction failed: {:?}", result.err()));
        }

        info!("Transaction confirmed: {:?}", tx.get_signature());
        info!("Transferred {} tokens to {}", amount, recipient_pubkey);

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn enqueue_withdrawal(&self, vault: String, amount: u64) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;
        let vault_account_raw = rpc_client.get_account(&vault).await?;
        let vault_account = Vault::try_from_slice_unchecked(&vault_account_raw.data)?;

        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &self.vault_program_id,
            &vault,
            &signer.pubkey(),
        )
        .0;

        let vault_staker_withdrawal_ticket_token_account =
            get_associated_token_address(&vault_staker_withdrawal_ticket, &vault_account.vrt_mint);

        let staker_vrt_token_account =
            get_associated_token_address(&signer.pubkey(), &vault_account.vrt_mint);

        let vault_staker_withdrawal_ticket_ata_ix = create_associated_token_account_idempotent(
            &signer.pubkey(),
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
            .staker(signer.pubkey())
            .staker_vrt_token_account(staker_vrt_token_account)
            .base(signer.pubkey())
            .amount(amount);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[
                vault_staker_withdrawal_ticket_ata_ix,
                ix_builder.instruction(),
            ],
            Some(&signer.pubkey()),
            &[signer],
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

    /// Changes the owner of a withdrawal ticket
    ///
    /// Transfers ownership of a vault staker withdrawal ticket from one account to another.
    /// This operation requires the signature of both the current ticket owner and the
    /// signer configured in the client.
    #[allow(clippy::future_not_send)]
    pub async fn change_withdrawal_ticket_owner(
        &self,
        vault: &Pubkey,
        old_ticket_owner: &str,
        new_ticket_owner: &Pubkey,
    ) -> Result<()> {
        let signer = self.signer()?;

        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &self.vault_program_id,
            vault,
            &signer.pubkey(),
        )
        .0;

        let old_ticket_owner_keypair = read_keypair_file(old_ticket_owner)
            .map_err(|e| anyhow!("Failed to read old admin keypair: {}", e))?;
        let old_ticket_owner_signer = CliSigner::new(Some(old_ticket_owner_keypair), None);

        let mut ix_builder = ChangeWithdrawalTicketOwnerBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(*vault)
            .vault_staker_withdrawal_ticket(vault_staker_withdrawal_ticket)
            .old_owner(old_ticket_owner_signer.pubkey())
            .new_owner(*new_ticket_owner);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Changing Withdrawal Ticket Owner",);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer, &old_ticket_owner_signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::VaultStakerWithdrawalTicket>(
                    &vault_staker_withdrawal_ticket,
                )
                .await?;
            info!("{}", account.pretty_display());
            info!(
                "Change withdrawal ticket owner from {} to {}",
                old_ticket_owner_signer.pubkey(),
                new_ticket_owner
            );
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn burn_withdrawal_ticket(&self, vault: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let rpc_client = self.get_rpc_client();

        let vault = Pubkey::from_str(&vault)?;
        let vault_account_raw = rpc_client.get_account(&vault).await?;
        let vault_account = Vault::try_from_slice_unchecked(&vault_account_raw.data)?;

        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &self.vault_program_id,
            &vault,
            &signer.pubkey(),
        )
        .0;

        let staker = signer.pubkey();
        let staker_token_account =
            get_associated_token_address(&staker, &vault_account.supported_mint);

        let vault_token_account =
            get_associated_token_address(&vault, &vault_account.supported_mint);

        let vault_fee_token_account =
            get_associated_token_address(&vault_account.fee_wallet, &vault_account.vrt_mint);

        let vault_staker_withdrawal_ticket_token_account =
            get_associated_token_address(&vault_staker_withdrawal_ticket, &vault_account.vrt_mint);

        let config = Config::find_program_address(&self.vault_program_id).0;
        let config_account_raw = rpc_client.get_account(&config).await?;
        let config_account = Config::try_from_slice_unchecked(&config_account_raw.data)?;

        let program_fee_ata = create_associated_token_account_idempotent(
            &signer.pubkey(),
            &config_account.program_fee_wallet,
            &vault_account.vrt_mint,
            &spl_token::ID,
        );

        let program_fee_token_account = get_associated_token_address(
            &config_account.program_fee_wallet,
            &vault_account.vrt_mint,
        );

        let mut ix_builder = BurnWithdrawalTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vrt_mint(vault_account.vrt_mint)
            .vault(vault)
            .vault_staker_withdrawal_ticket(vault_staker_withdrawal_ticket)
            .vault_staker_withdrawal_ticket_token_account(
                vault_staker_withdrawal_ticket_token_account,
            )
            .program_fee_token_account(program_fee_token_account)
            .staker_token_account(staker_token_account)
            .vault_fee_token_account(vault_fee_token_account)
            .vault_token_account(vault_token_account)
            .staker(staker);

        let blockhash = rpc_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[program_fee_ata, ix_builder.instruction()],
            Some(&signer.pubkey()),
            &[signer],
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
    #[allow(clippy::future_not_send)]
    pub async fn get_vault(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let rpc_client = self.get_rpc_client();

        let vault_account = rpc_client.get_account(&pubkey).await?;
        let vault =
            jito_vault_client::accounts::Vault::deserialize(&mut vault_account.data.as_slice())?;

        let metadata_pubkey = Pubkey::find_program_address(
            &[
                b"metadata",
                inline_mpl_token_metadata::id().as_ref(),
                vault.vrt_mint.as_ref(),
            ],
            &inline_mpl_token_metadata::id(),
        )
        .0;

        info!("Vault at address {}", pubkey);
        info!("{}", vault.pretty_display());

        if let Ok(metadata) = self
            .get_account::<jito_vault_client::log::metadata::Metadata>(&metadata_pubkey)
            .await
        {
            info!("{}", metadata.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn list_vaults(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let config = self.get_rpc_program_accounts_config::<Vault>(None)?;
        let accounts = rpc_client
            .get_program_accounts_with_config(&self.vault_program_id, config)
            .await
            .unwrap();
        log::info!("{:?}", accounts);
        for (vault_pubkey, vault) in accounts {
            let vault =
                jito_vault_client::accounts::Vault::deserialize(&mut vault.data.as_slice())?;

            let metadata_pubkey = Pubkey::find_program_address(
                &[
                    b"metadata",
                    inline_mpl_token_metadata::id().as_ref(),
                    vault.vrt_mint.as_ref(),
                ],
                &inline_mpl_token_metadata::id(),
            )
            .0;

            info!("Vault at address {}", vault_pubkey);
            info!("{}", vault.pretty_display());

            if let Ok(metadata) = self
                .get_account::<jito_vault_client::log::metadata::Metadata>(&metadata_pubkey)
                .await
            {
                info!("{}", metadata.pretty_display());
            }
        }
        Ok(())
    }

    #[allow(clippy::future_not_send)]
    async fn get_config(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        debug!(
            "Reading the restaking configuration account at address: {}",
            config_address
        );

        let account = rpc_client.get_account(&config_address).await?;
        let config =
            jito_vault_client::accounts::Config::deserialize(&mut account.data.as_slice())?;
        info!("Vault config at address {}", config_address);
        info!("{}", config.pretty_display());
        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn get_vault_update_state_tracker(&self, vault: String) -> Result<()> {
        let vault = Pubkey::from_str(&vault)?;
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let config = self
            .get_account::<jito_vault_client::accounts::Config>(&config_address)
            .await?;

        let slot = rpc_client.get_slot().await?;
        let ncn_epoch = get_epoch(slot, config.epoch_length)?;

        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &self.vault_program_id,
            &vault,
            ncn_epoch,
        )
        .0;
        let account = rpc_client.get_account(&vault_update_state_tracker).await?;
        let state_tracker = jito_vault_client::accounts::VaultUpdateStateTracker::deserialize(
            &mut account.data.as_slice(),
        )?;
        info!(
            "Vault Update State Tracker at address {}",
            vault_update_state_tracker
        );
        info!("{}", state_tracker.pretty_display());
        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn get_vault_operator_delegations(
        &self,
        vault: String,
        operator: Option<String>,
    ) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let vault = Pubkey::from_str(&vault)?;
        let operator_pubkey = match operator {
            Some(operator) => Some(Pubkey::from_str(&operator)?),
            None => None,
        };

        match operator_pubkey {
            Some(operator) => {
                let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
                    &self.vault_program_id,
                    &vault,
                    &operator,
                )
                .0;
                let account = rpc_client.get_account(&vault_operator_delegation).await?;

                let delegation = jito_vault_client::accounts::VaultOperatorDelegation::deserialize(
                    &mut account.data.as_slice(),
                )?;

                info!(
                    "Vault Operator Delegation at address {}",
                    vault_operator_delegation
                );
                info!("{}", delegation.pretty_display());
            }
            None => {
                let config = self.get_rpc_program_accounts_config::<VaultOperatorDelegation>(
                    Some((&vault, 8)),
                )?;
                let accounts = rpc_client
                    .get_program_accounts_with_config(&self.vault_program_id, config)
                    .await?;

                for (index, (pubkey, account)) in accounts.iter().enumerate() {
                    let vault_operator_delegation =
                        jito_vault_client::accounts::VaultOperatorDelegation::deserialize(
                            &mut account.data.as_slice(),
                        )?;

                    info!("Vault Operator Delegation {} at address {}", index, pubkey);
                    info!("{}", vault_operator_delegation.pretty_display());
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn get_withdrawal_ticket(&self, vault: String, staker: Option<String>) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let vault = Pubkey::from_str(&vault)?;
        let staker = if let Some(staker) = staker {
            Pubkey::from_str(&staker)?
        } else {
            let signer = self
                .cli_config
                .signer
                .as_ref()
                .ok_or_else(|| anyhow!("Keypair not provided"))?;
            signer.pubkey()
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
        let ticket = jito_vault_client::accounts::VaultStakerWithdrawalTicket::deserialize(
            &mut account.data.as_slice(),
        )?;
        info!(
            "Vault Staker Withdrawal Ticket at address {}",
            vault_staker_withdrawal_ticket
        );
        info!("{}", ticket.pretty_display());

        Ok(())
    }

    /// Sets the primary admin for a Vault
    ///
    /// This function transfers administrative control of a Vault account from the current admin
    /// to a new admin. It supports both file-based keypairs and hardware wallets (USB devices)
    /// for both the old and new admin. The function builds and processes a transaction that
    /// updates the admin public key in the Vault account.
    #[allow(clippy::future_not_send)]
    async fn set_admin(
        &self,
        vault: &Pubkey,
        old_admin_keypair: &str,
        new_admin_keypair: &str,
    ) -> Result<()> {
        let mut old_admin_owned = None;
        let mut new_admin_owned = None;

        let old_admin_signer = self.resolve_keypair(old_admin_keypair, &mut old_admin_owned)?;
        let new_admin_signer = self.resolve_keypair(new_admin_keypair, &mut new_admin_owned)?;

        let mut ix_builder = SetAdminBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(*vault)
            .old_admin(old_admin_signer.pubkey())
            .new_admin(new_admin_signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Setting Vault admin to {}", new_admin_signer.pubkey());

        self.process_transaction(
            &[ix],
            &new_admin_signer.pubkey(),
            &[new_admin_signer, old_admin_signer],
        )
        .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Vault>(vault)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Set the capacity for Vault
    ///
    /// Updates the maximum deposit capacity for a specific vault.
    /// This operation can only be performed by the vault admin.
    #[allow(clippy::future_not_send)]
    pub async fn set_capacity(&self, vault: String, amount: u64) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;
        let vault_pubkey = Pubkey::from_str(&vault)?;

        let mut builder = SetDepositCapacityBuilder::new();
        builder
            .config(Config::find_program_address(&self.vault_program_id).0)
            .vault(vault_pubkey)
            .admin(signer.pubkey())
            .amount(amount);
        let mut ix = builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Vault capacity instruction: {:?}", builder);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Vault>(&vault_pubkey)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Sets the primary admin for Config
    ///
    /// Transfers administrative control of the Config to a new admin.
    /// This operation can only be performed by the current admin.
    #[allow(clippy::future_not_send)]
    async fn set_config_admin(&self, new_admin: Pubkey) -> Result<()> {
        let signer = self.signer()?;

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let mut ix_builder = SetConfigAdminBuilder::new();
        ix_builder
            .config(config_address)
            .old_admin(signer.pubkey())
            .new_admin(new_admin);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Setting vault config admin parameters: {:?}", ix_builder);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Config>(&config_address)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Set the fees for Vault
    ///
    /// Updates one or more fee parameters for a specific vault. Each fee type
    /// (deposit, withdrawal, reward) is specified in basis points and can be
    /// updated independently. Any fee type not provided (None) will remain unchanged.
    ///
    /// NOTE:
    /// - Fee changes are only allowed once per epoch
    #[allow(clippy::future_not_send)]
    async fn set_fees(
        &self,
        vault: &Pubkey,
        deposit_fee_bps: Option<u16>,
        withdrawal_fee_bps: Option<u16>,
        reward_fee_bps: Option<u16>,
    ) -> Result<()> {
        let signer = self.signer()?;

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let mut ix_builder = SetFeesBuilder::new();
        ix_builder
            .config(config_address)
            .vault(*vault)
            .admin(signer.pubkey());

        if let Some(deposit_fee_bps) = deposit_fee_bps {
            ix_builder.deposit_fee_bps(deposit_fee_bps);
        }

        if let Some(withdrawal_fee_bps) = withdrawal_fee_bps {
            ix_builder.withdrawal_fee_bps(withdrawal_fee_bps);
        }

        if let Some(reward_fee_bps) = reward_fee_bps {
            ix_builder.reward_fee_bps(reward_fee_bps);
        }

        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Setting Vault fees: {:?}", ix_builder);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Vault>(vault)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Sets the pause state for a specific vault
    ///
    /// Enables or disables operations on a vault by setting its pause state.
    /// When paused, most interactions with the vault will be rejected.
    /// This operation can only be performed by the vault admin.
    #[allow(clippy::future_not_send)]
    async fn set_is_paused(&self, vault: &Pubkey, set_pause: bool) -> Result<()> {
        let signer = self.signer()?;

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let mut ix_builder = SetIsPausedBuilder::new();
        ix_builder
            .config(config_address)
            .vault(*vault)
            .admin(signer.pubkey())
            .is_paused(set_pause);

        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Setting Is Paused: {:?}", ix_builder);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Vault>(vault)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Sets a new program fee (in basis points) for the Config
    ///
    /// Updates the fee percentage (specified in basis points) that the program
    /// collects for vault operation. This operation can only be performed by the
    /// current admin of the config.
    #[allow(clippy::future_not_send)]
    async fn set_program_fee(&self, new_fee_bps: u16) -> Result<()> {
        let signer = self.signer()?;

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let mut ix_builder = SetProgramFeeBuilder::new();
        ix_builder
            .config(config_address)
            .admin(signer.pubkey())
            .new_fee_bps(new_fee_bps);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!(
            "Setting vault config program fee bps parameters: {:?}",
            ix_builder
        );

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Config>(&config_address)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Sets a new program fee wallet for the Config
    ///
    /// Updates the wallet address that receives program fees collected by the Jito Vault Program.
    /// This operation can only be performed by the current program fee admin.
    #[allow(clippy::future_not_send)]
    async fn set_program_fee_wallet(&self, new_fee_wallet: &Pubkey) -> Result<()> {
        let signer = self.signer()?;

        let config_address = Config::find_program_address(&self.vault_program_id).0;
        let mut ix_builder = SetProgramFeeWalletBuilder::new();
        ix_builder
            .config(config_address)
            .program_fee_admin(signer.pubkey())
            .new_fee_wallet(*new_fee_wallet);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!(
            "Setting vault config program fee wallet parameters: {:?}",
            ix_builder
        );

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Config>(&config_address)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Sets secondary admin roles for Vault
    ///
    /// This function allows assigning a new administrator to various administrative roles
    /// for a specific Vault. Multiple roles can be assigned in a single call by enabling the
    /// corresponding boolean flags.
    #[allow(clippy::too_many_arguments, clippy::future_not_send)]
    async fn set_secondary_admin(
        &self,
        vault: &Pubkey,
        new_admin: &Pubkey,
        set_delegation_admin: bool,
        set_operator_admin: bool,
        set_ncn_admin: bool,
        set_slasher_admin: bool,
        set_capacity_admin: bool,
        set_fee_wallet: bool,
        set_mint_burn_admin: bool,
        set_delegate_asset_admin: bool,
        set_fee_admin: bool,
        set_metadata_admin: bool,
    ) -> Result<()> {
        let signer = self.signer()?;
        let config_address = Config::find_program_address(&self.vault_program_id).0;

        let mut roles: Vec<VaultAdminRole> = vec![];
        if set_delegation_admin {
            roles.push(VaultAdminRole::DelegationAdmin);
        }
        if set_operator_admin {
            roles.push(VaultAdminRole::OperatorAdmin);
        }
        if set_ncn_admin {
            roles.push(VaultAdminRole::NcnAdmin);
        }
        if set_slasher_admin {
            roles.push(VaultAdminRole::SlasherAdmin);
        }
        if set_capacity_admin {
            roles.push(VaultAdminRole::CapacityAdmin);
        }
        if set_fee_wallet {
            roles.push(VaultAdminRole::FeeWallet);
        }
        if set_mint_burn_admin {
            roles.push(VaultAdminRole::MintBurnAdmin);
        }
        if set_delegate_asset_admin {
            roles.push(VaultAdminRole::DelegateAssetAdmin);
        }
        if set_fee_admin {
            roles.push(VaultAdminRole::FeeAdmin);
        }
        if set_metadata_admin {
            roles.push(VaultAdminRole::MetadataAdmin);
        }

        for role in roles.iter() {
            let mut ix_builder = SetSecondaryAdminBuilder::new();
            ix_builder
                .config(config_address)
                .new_admin(*new_admin)
                .vault(*vault)
                .admin(signer.pubkey())
                .vault_admin_role(*role)
                .instruction();
            let mut ix = ix_builder.instruction();
            ix.program_id = self.vault_program_id;

            info!(
                "Setting {:?} Admin to {} for Vault {}",
                role, new_admin, vault
            );

            self.process_transaction(&[ix], &signer.pubkey(), &[signer])
                .await?;
        }

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Vault>(vault)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Updates the vault balance
    ///
    /// Synchronizes the vault's internal token balance with its actual token holdings and
    /// calculates rewards. This function:
    /// 1. Verifies the vault is not paused and can be updated
    /// 2. Calculates rewards based on the difference between current and tracked token balance
    /// 3. Applies the reward fee according to the vault's configuration
    /// 4. Updates the vault's tracked token balance
    /// 5. Mints VRT tokens to the fee wallet as reward fees
    #[allow(clippy::future_not_send)]
    async fn update_vault_balance(&self, vault: &Pubkey) -> Result<()> {
        let signer = self.signer()?;

        let config_address = Config::find_program_address(&self.vault_program_id).0;

        let vault_account_raw = self.get_rpc_client().get_account(vault).await?;
        let vault_account = Vault::try_from_slice_unchecked(&vault_account_raw.data)?;

        let vault_token_account =
            get_associated_token_address(vault, &vault_account.supported_mint);

        let vault_fee_token_account =
            get_associated_token_address(&vault_account.fee_wallet, &vault_account.vrt_mint);

        let mut ix_builder = UpdateVaultBalanceBuilder::new();
        ix_builder
            .config(config_address)
            .vault(*vault)
            .vault_token_account(vault_token_account)
            .vrt_mint(vault_account.vrt_mint)
            .vault_fee_token_account(vault_fee_token_account);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.vault_program_id;

        info!("Update Vault balance: {:?}", ix_builder);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_vault_client::accounts::Vault>(vault)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }
}
