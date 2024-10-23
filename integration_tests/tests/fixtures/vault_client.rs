use std::{fmt, fmt::Debug};

use borsh::BorshDeserialize;
use jito_bytemuck::AccountDeserialize;
use jito_restaking_core::{
    ncn_operator_state::NcnOperatorState, ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
    ncn_vault_ticket::NcnVaultTicket, operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    config::Config, vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};
use jito_vault_sdk::{
    error::VaultError,
    inline_mpl_token_metadata,
    instruction::{VaultAdminRole, WithdrawalAllocationMethod},
    sdk::{
        add_delegation, cooldown_delegation, initialize_config, initialize_vault,
        set_deposit_capacity, warmup_vault_ncn_slasher_ticket, warmup_vault_ncn_ticket,
    },
};
use log::info;
use solana_program::{
    clock::Clock,
    native_token::sol_to_lamports,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{create_account, transfer},
};
use solana_program_test::{BanksClient, BanksClientError, ProgramTestBanksClientExt};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};
use spl_token::state::Account as SPLTokenAccount;
use spl_token_2022::extension::ExtensionType;

use crate::fixtures::{TestError, TestResult};

pub struct VaultRoot {
    pub vault_pubkey: Pubkey,
    pub vault_admin: Keypair,
}

impl Debug for VaultRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VaultRoot {{ vault_pubkey: {}, vault_admin: {:?} }}",
            self.vault_pubkey, self.vault_admin
        )
    }
}

#[derive(Debug)]
pub struct VaultStakerWithdrawalTicketRoot {
    pub base: Pubkey,
}

pub struct VaultProgramClient {
    banks_client: BanksClient,
    payer: Keypair,
}

impl VaultProgramClient {
    pub const fn new(banks_client: BanksClient, payer: Keypair) -> Self {
        Self {
            banks_client,
            payer,
        }
    }

    pub async fn configure_depositor(
        &mut self,
        vault_root: &VaultRoot,
        depositor: &Pubkey,
        amount_to_mint: u64,
    ) -> TestResult<()> {
        self.airdrop(depositor, 100.0).await?;
        let vault = self.get_vault(&vault_root.vault_pubkey).await?;
        self.create_ata(&vault.supported_mint, depositor).await?;
        self.create_ata(&vault.vrt_mint, depositor).await?;
        self.mint_spl_to(&vault.supported_mint, depositor, amount_to_mint)
            .await?;

        Ok(())
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, TestError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(*Config::try_from_slice_unchecked(account.data.as_slice())?)
    }

    pub async fn get_vault(&mut self, account: &Pubkey) -> Result<Vault, TestError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(*Vault::try_from_slice_unchecked(account.data.as_slice())?)
    }

    pub async fn get_vault_ncn_ticket(
        &mut self,
        vault: &Pubkey,
        ncn: &Pubkey,
    ) -> Result<VaultNcnTicket, TestError> {
        let account = VaultNcnTicket::find_program_address(&jito_vault_program::id(), vault, ncn).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*VaultNcnTicket::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_operator_delegation(
        &mut self,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> Result<VaultOperatorDelegation, TestError> {
        let account = VaultOperatorDelegation::find_program_address(
            &jito_vault_program::id(),
            vault,
            operator,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*VaultOperatorDelegation::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_staker_withdrawal_ticket(
        &mut self,
        vault: &Pubkey,
        staker: &Pubkey,
        base: &Pubkey,
    ) -> Result<VaultStakerWithdrawalTicket, TestError> {
        let account = VaultStakerWithdrawalTicket::find_program_address(
            &jito_vault_program::id(),
            vault,
            base,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        let withdrawal_ticket =
            *VaultStakerWithdrawalTicket::try_from_slice_unchecked(account.data.as_slice())?;
        assert_eq!(withdrawal_ticket.staker, *staker);
        Ok(withdrawal_ticket)
    }

    pub async fn get_vault_ncn_slasher_ticket(
        &mut self,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<VaultNcnSlasherTicket, TestError> {
        let account = VaultNcnSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            vault,
            ncn,
            slasher,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*VaultNcnSlasherTicket::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_ncn_slasher_operator_ticket(
        &mut self,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> Result<VaultNcnSlasherOperatorTicket, TestError> {
        let account = VaultNcnSlasherOperatorTicket::find_program_address(
            &jito_vault_program::id(),
            vault,
            ncn,
            slasher,
            operator,
            epoch,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*VaultNcnSlasherOperatorTicket::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_update_state_tracker(
        &mut self,
        vault: &Pubkey,
        epoch: u64,
    ) -> Result<VaultUpdateStateTracker, TestError> {
        let account =
            VaultUpdateStateTracker::find_program_address(&jito_vault_program::id(), vault, epoch)
                .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*VaultUpdateStateTracker::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    pub async fn get_token_metadata(
        &mut self,
        vrt_mint: &Pubkey,
    ) -> Result<crate::helpers::token::Metadata, TestError> {
        let metadata_pubkey = inline_mpl_token_metadata::pda::find_metadata_account(vrt_mint).0;
        let token_metadata_account = self
            .banks_client
            .get_account(metadata_pubkey)
            .await?
            .unwrap();
        let metadata = crate::helpers::token::Metadata::deserialize(
            &mut token_metadata_account.data.as_slice(),
        )
        .unwrap();
        Ok(metadata)
    }

    pub async fn do_initialize_config(&mut self) -> Result<Keypair, TestError> {
        let config_admin = Keypair::new();

        self.airdrop(&config_admin.pubkey(), 1.0).await?;

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        self.initialize_config(&config_pubkey, &config_admin, &config_admin.pubkey(), 0)
            .await?;

        Ok(config_admin)
    }

    pub async fn initialize_config(
        &mut self,
        config: &Pubkey,
        config_admin: &Keypair,
        program_fee_wallet: &Pubkey,
        program_fee_bps: u16,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_vault_program::id(),
                config,
                &config_admin.pubkey(),
                &jito_restaking_program::id(),
                program_fee_wallet,
                program_fee_bps,
            )],
            Some(&config_admin.pubkey()),
            &[config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn setup_config_and_vault(
        &mut self,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
    ) -> Result<(Keypair, VaultRoot), TestError> {
        let config_admin = self.do_initialize_config().await?;
        let vault_root = self
            .do_initialize_vault(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                9,
                &config_admin.pubkey(),
            )
            .await?;

        Ok((config_admin, vault_root))
    }

    pub async fn do_initialize_vault(
        &mut self,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        decimals: u8,
        program_fee_wallet: &Pubkey,
    ) -> Result<VaultRoot, TestError> {
        let vault_base = Keypair::new();

        let vault_pubkey =
            Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;

        let vrt_mint = Keypair::new();
        let vault_admin = Keypair::new();
        let token_mint = Keypair::new();

        self.airdrop(&vault_admin.pubkey(), 100.0).await?;
        self.create_token_mint(&token_mint, &spl_token::id())
            .await?;

        self.initialize_vault(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_pubkey,
            &vrt_mint,
            &token_mint,
            &vault_admin,
            &vault_base,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
            decimals,
        )
        .await?;

        // for holding the backed asset in the vault
        self.create_ata(&token_mint.pubkey(), &vault_pubkey).await?;
        // for holding fees
        self.create_ata(&vrt_mint.pubkey(), &vault_admin.pubkey())
            .await?;
        // for holding program fee
        self.create_ata(&vrt_mint.pubkey(), program_fee_wallet)
            .await?;

        // for holding program fee
        Ok(VaultRoot {
            vault_admin,
            vault_pubkey,
        })
    }

    pub async fn do_initialize_vault_ncn_ticket(
        &mut self,
        vault_root: &VaultRoot,
        ncn: &Pubkey,
    ) -> Result<(), TestError> {
        let vault_ncn_ticket = VaultNcnTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn,
        )
        .0;
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn,
            &vault_root.vault_pubkey,
        )
        .0;
        self.initialize_vault_ncn_ticket(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            ncn,
            &ncn_vault_ticket,
            &vault_ncn_ticket,
            &vault_root.vault_admin,
            &self.payer.insecure_clone(),
        )
        .await?;

        Ok(())
    }

    pub async fn set_capacity(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        admin: &Keypair,
        capacity: u64,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[set_deposit_capacity(
                &jito_vault_program::id(),
                config,
                vault,
                &admin.pubkey(),
                capacity,
            )],
            Some(&admin.pubkey()),
            &[&admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_warmup_vault_ncn_ticket(
        &mut self,
        vault_root: &VaultRoot,
        ncn: &Pubkey,
    ) -> Result<(), TestError> {
        let vault_ncn_ticket = VaultNcnTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn,
        )
        .0;

        self.warmup_vault_ncn_ticket(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            ncn,
            &vault_ncn_ticket,
            &vault_root.vault_admin,
        )
        .await?;

        Ok(())
    }

    pub async fn warmup_vault_ncn_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        vault_ncn_ticket: &Pubkey,
        ncn_vault_admin: &Keypair,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[warmup_vault_ncn_ticket(
                &jito_vault_program::id(),
                config,
                vault,
                ncn,
                vault_ncn_ticket,
                &ncn_vault_admin.pubkey(),
            )],
            Some(&ncn_vault_admin.pubkey()),
            &[&ncn_vault_admin],
            blockhash,
        ))
        .await
    }

    pub async fn setup_vault_ncn_slasher_operator_ticket(
        &mut self,
        vault_root: &VaultRoot,
        ncn_pubkey: &Pubkey,
        slasher: &Pubkey,
        operator_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        let config = self
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        let clock: Clock = self.banks_client.get_sysvar().await?;

        let vault_ncn_slasher_ticket = VaultNcnSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_pubkey,
            slasher,
        )
        .0;
        let vault_ncn_slasher_operator_ticket =
            VaultNcnSlasherOperatorTicket::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                ncn_pubkey,
                slasher,
                operator_pubkey,
                clock.slot / config.epoch_length(),
            )
            .0;
        self.initialize_vault_ncn_slasher_operator_ticket(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            ncn_pubkey,
            slasher,
            operator_pubkey,
            &vault_ncn_slasher_ticket,
            &vault_ncn_slasher_operator_ticket,
            &self.payer.insecure_clone(),
        )
        .await
        .unwrap();

        Ok(())
    }

    pub async fn do_slash(
        &mut self,
        vault_root: &VaultRoot,
        ncn_pubkey: &Pubkey,
        slasher: &Keypair,
        operator_pubkey: &Pubkey,
        amount: u64,
    ) -> Result<(), TestError> {
        let ncn_operator_state_pubkey = NcnOperatorState::find_program_address(
            &jito_restaking_program::id(),
            ncn_pubkey,
            operator_pubkey,
        )
        .0;
        let ncn_vault_ticket_pubkey = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        let operator_vault_ticket_pubkey = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            operator_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        let vault_ncn_ticket_pubkey = VaultNcnTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_pubkey,
        )
        .0;
        let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            operator_pubkey,
        )
        .0;
        let ncn_slasher_ticket_pubkey = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn_pubkey,
            &vault_root.vault_pubkey,
            &slasher.pubkey(),
        )
        .0;
        let vault_slasher_ticket_pubkey = VaultNcnSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_pubkey,
            &slasher.pubkey(),
        )
        .0;
        let config = self
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await
            .unwrap();
        let clock: Clock = self.banks_client.get_sysvar().await?;

        let vault_ncn_slasher_operator_ticket =
            VaultNcnSlasherOperatorTicket::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                ncn_pubkey,
                &slasher.pubkey(),
                operator_pubkey,
                clock.slot / config.epoch_length(),
            )
            .0;

        let vault = self.get_vault(&vault_root.vault_pubkey).await.unwrap();
        let vault_token_account =
            get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint);
        let slasher_token_account =
            get_associated_token_address(&slasher.pubkey(), &vault.supported_mint);

        self.slash(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            ncn_pubkey,
            operator_pubkey,
            slasher,
            &ncn_operator_state_pubkey,
            &ncn_vault_ticket_pubkey,
            &operator_vault_ticket_pubkey,
            &vault_ncn_ticket_pubkey,
            &vault_operator_delegation,
            &ncn_slasher_ticket_pubkey,
            &vault_slasher_ticket_pubkey,
            &vault_ncn_slasher_operator_ticket,
            &vault_token_account,
            &slasher_token_account,
            amount,
        )
        .await?;

        Ok(())
    }

    pub async fn do_initialize_vault_operator_delegation(
        &mut self,
        vault_root: &VaultRoot,
        operator_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            operator_pubkey,
        )
        .0;
        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            operator_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        self.initialize_vault_operator_delegation(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            operator_pubkey,
            &operator_vault_ticket,
            &vault_operator_delegation,
            &vault_root.vault_admin,
            &vault_root.vault_admin,
        )
        .await?;

        Ok(())
    }

    pub async fn do_initialize_vault_ncn_slasher_ticket(
        &mut self,
        vault_root: &VaultRoot,
        ncn_pubkey: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<(), TestError> {
        let vault_slasher_ticket_pubkey = VaultNcnSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_pubkey,
            slasher,
        )
        .0;
        let ncn_slasher_ticket_pubkey = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn_pubkey,
            &vault_root.vault_pubkey,
            slasher,
        )
        .0;

        self.initialize_vault_ncn_slasher_ticket(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            ncn_pubkey,
            slasher,
            &ncn_slasher_ticket_pubkey,
            &vault_slasher_ticket_pubkey,
            &vault_root.vault_admin,
            &vault_root.vault_admin,
        )
        .await?;

        Ok(())
    }

    pub async fn do_warmup_vault_ncn_slasher_ticket(
        &mut self,
        vault_root: &VaultRoot,
        ncn_pubkey: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<(), TestError> {
        let vault_slasher_ticket_pubkey = VaultNcnSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_pubkey,
            slasher,
        )
        .0;

        self.warmup_vault_ncn_slasher_ticket(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            ncn_pubkey,
            slasher,
            &vault_slasher_ticket_pubkey,
            &vault_root.vault_admin,
        )
        .await?;

        Ok(())
    }

    pub async fn warmup_vault_ncn_slasher_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        vault_ncn_slasher_ticket: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[warmup_vault_ncn_slasher_ticket(
                &jito_vault_program::id(),
                config,
                vault,
                ncn,
                slasher,
                vault_ncn_slasher_ticket,
                &admin.pubkey(),
            )],
            Some(&admin.pubkey()),
            &[admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_add_delegation(
        &mut self,
        vault_root: &VaultRoot,
        operator: &Pubkey,
        amount: u64,
    ) -> Result<(), TestError> {
        self.add_delegation(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            operator,
            &VaultOperatorDelegation::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                operator,
            )
            .0,
            &vault_root.vault_admin,
            amount,
        )
        .await?;

        Ok(())
    }

    pub async fn initialize_vault(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        vrt_mint: &Keypair,
        token_mint: &Keypair,
        vault_admin: &Keypair,
        vault_base: &Keypair,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        decimals: u8,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_vault(
                &jito_vault_program::id(),
                config,
                vault,
                &vrt_mint.pubkey(),
                &token_mint.pubkey(),
                &vault_admin.pubkey(),
                &vault_base.pubkey(),
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                decimals,
            )],
            Some(&vault_admin.pubkey()),
            &[&vault_admin, &vrt_mint, &vault_base],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault_ncn_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        vault_ncn_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::initialize_vault_ncn_ticket(
                &jito_vault_program::id(),
                config,
                vault,
                ncn,
                ncn_vault_ticket,
                vault_ncn_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault_operator_delegation(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
        operator_vault_ticket: &Pubkey,
        vault_operator_delegation: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::initialize_vault_operator_delegation(
                &jito_vault_program::id(),
                config,
                vault,
                operator,
                operator_vault_ticket,
                vault_operator_delegation,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn delegate_token_account(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        delegate_asset_admin: &Keypair,
        token_mint: &Pubkey,
        token_account: &Pubkey,
        delegate: &Pubkey,
        token_program_id: &Pubkey,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::delegate_token_account(
                &jito_vault_program::id(),
                config,
                vault,
                &delegate_asset_admin.pubkey(),
                token_mint,
                token_account,
                delegate,
                token_program_id,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer, delegate_asset_admin],
            blockhash,
        ))
        .await
    }

    pub async fn set_admin(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        old_admin: &Keypair,
        new_admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::set_admin(
                &jito_vault_program::id(),
                config,
                vault,
                &old_admin.pubkey(),
                &new_admin.pubkey(),
            )],
            Some(&old_admin.pubkey()),
            &[old_admin, new_admin],
            blockhash,
        ))
        .await
    }

    pub async fn set_secondary_admin(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        admin: &Keypair,
        new_admin: &Pubkey,
        role: VaultAdminRole,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::set_secondary_admin(
                &jito_vault_program::id(),
                config,
                vault,
                &admin.pubkey(),
                new_admin,
                role,
            )],
            Some(&admin.pubkey()),
            &[admin],
            blockhash,
        ))
        .await
    }

    pub async fn set_fees(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        fee_admin: &Keypair,
        deposit_fee_bps: Option<u16>,
        withdrawal_fee_bps: Option<u16>,
        reward_fee_bps: Option<u16>,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::set_fees(
                &jito_vault_program::id(),
                config,
                vault,
                &fee_admin.pubkey(),
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
            )],
            Some(&fee_admin.pubkey()),
            &[fee_admin],
            blockhash,
        ))
        .await
    }

    pub async fn set_program_fee(
        &mut self,
        config_admin: &Keypair,
        new_fee_bps: u16,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::set_program_fee(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                &config_admin.pubkey(),
                new_fee_bps,
            )],
            Some(&config_admin.pubkey()),
            &[config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_enqueue_withdrawal(
        &mut self,
        vault_root: &VaultRoot,
        depositor: &Keypair,
        amount: u64,
    ) -> Result<VaultStakerWithdrawalTicketRoot, TestError> {
        let vault = self.get_vault(&vault_root.vault_pubkey).await.unwrap();
        let depositor_vrt_token_account =
            get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint);

        let base = Keypair::new();
        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            &base.pubkey(),
        )
        .0;
        info!(
            "vault_staker_withdrawal_ticket: {:?}",
            vault_staker_withdrawal_ticket
        );
        let vault_staker_withdrawal_ticket_token_account =
            get_associated_token_address(&vault_staker_withdrawal_ticket, &vault.vrt_mint);

        self.create_ata(&vault.vrt_mint, &vault_staker_withdrawal_ticket)
            .await?;

        self.enqueue_withdrawal(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            &vault_staker_withdrawal_ticket,
            &vault_staker_withdrawal_ticket_token_account,
            depositor,
            &depositor_vrt_token_account,
            &base,
            amount,
        )
        .await?;

        Ok(VaultStakerWithdrawalTicketRoot {
            base: base.pubkey(),
        })
    }

    pub async fn do_cooldown_delegation(
        &mut self,
        vault_root: &VaultRoot,
        operator: &Pubkey,
        amount: u64,
    ) -> TestResult<()> {
        self.cooldown_delegation(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            operator,
            &VaultOperatorDelegation::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                operator,
            )
            .0,
            &vault_root.vault_admin,
            amount,
        )
        .await
    }

    pub async fn cooldown_delegation(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
        vault_operator_delegation: &Pubkey,
        admin: &Keypair,
        amount: u64,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[cooldown_delegation(
                &jito_vault_program::id(),
                config,
                vault,
                operator,
                vault_operator_delegation,
                &admin.pubkey(),
                amount,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer, admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_full_vault_update(
        &mut self,
        vault_pubkey: &Pubkey,
        operators: &[Pubkey],
    ) -> Result<(), TestError> {
        let slot = self.banks_client.get_sysvar::<Clock>().await?.slot;

        let config = self
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await?;

        let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address(
            &jito_vault_program::id(),
            vault_pubkey,
            slot / config.epoch_length(),
        )
        .0;
        self.initialize_vault_update_state_tracker(vault_pubkey, &vault_update_state_tracker)
            .await?;

        for operator in operators {
            self.crank_vault_update_state_tracker(
                vault_pubkey,
                operator,
                &VaultOperatorDelegation::find_program_address(
                    &jito_vault_program::id(),
                    vault_pubkey,
                    operator,
                )
                .0,
                &vault_update_state_tracker,
            )
            .await?;
        }

        self.close_vault_update_state_tracker(
            vault_pubkey,
            &vault_update_state_tracker,
            slot / config.epoch_length(),
        )
        .await?;

        self.update_vault_balance(vault_pubkey).await?;

        Ok(())
    }

    pub async fn do_crank_vault_update_state_tracker(
        &mut self,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> TestResult<()> {
        let slot = self.banks_client.get_sysvar::<Clock>().await?.slot;
        let config = self
            .get_config(&Config::find_program_address(&jito_vault_program::id()).0)
            .await?;
        let ncn_epoch = slot / config.epoch_length();
        self.crank_vault_update_state_tracker(
            vault,
            operator,
            &VaultOperatorDelegation::find_program_address(
                &jito_vault_program::id(),
                vault,
                operator,
            )
            .0,
            &VaultUpdateStateTracker::find_program_address(
                &jito_vault_program::id(),
                vault,
                ncn_epoch,
            )
            .0,
        )
        .await
    }

    pub async fn crank_vault_update_state_tracker(
        &mut self,
        vault: &Pubkey,
        operator: &Pubkey,
        vault_operator_delegation: &Pubkey,
        vault_update_state_tracker: &Pubkey,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::crank_vault_update_state_tracker(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                vault,
                operator,
                vault_operator_delegation,
                vault_update_state_tracker,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        ))
        .await?;
        Ok(())
    }

    pub async fn update_vault_balance(&mut self, vault_pubkey: &Pubkey) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        let vault = self.get_vault(vault_pubkey).await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::update_vault_balance(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                vault_pubkey,
                &get_associated_token_address(vault_pubkey, &vault.supported_mint),
                &vault.vrt_mint,
                &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
                &spl_token::ID,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        ))
        .await?;

        Ok(())
    }

    pub async fn initialize_vault_update_state_tracker(
        &mut self,
        vault_pubkey: &Pubkey,
        vault_update_state_tracker: &Pubkey,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::initialize_vault_update_state_tracker(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                vault_pubkey,
                vault_update_state_tracker,
                &self.payer.pubkey(),
                WithdrawalAllocationMethod::Greedy,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        ))
        .await?;
        Ok(())
    }

    pub async fn close_vault_update_state_tracker(
        &mut self,
        vault_pubkey: &Pubkey,
        vault_update_state_tracker: &Pubkey,
        ncn_epoch: u64,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::close_vault_update_state_tracker(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                vault_pubkey,
                vault_update_state_tracker,
                &self.payer.pubkey(),
                ncn_epoch,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        ))
        .await
    }

    pub async fn enqueue_withdrawal(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        vault_staker_withdrawal_ticket: &Pubkey,
        vault_staker_withdrawal_ticket_token_account: &Pubkey,
        staker: &Keypair,
        staker_vrt_token_account: &Pubkey,
        base: &Keypair,
        amount: u64,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::enqueue_withdrawal(
                &jito_vault_program::id(),
                config,
                vault,
                vault_staker_withdrawal_ticket,
                vault_staker_withdrawal_ticket_token_account,
                &staker.pubkey(),
                staker_vrt_token_account,
                &base.pubkey(),
                amount,
            )],
            Some(&staker.pubkey()),
            &[staker, base],
            blockhash,
        ))
        .await
    }

    pub async fn do_burn_withdrawal_ticket(
        &mut self,
        vault_root: &VaultRoot,
        staker: &Keypair,
        vault_staker_withdrawal_ticket_base: &Pubkey,
        program_fee_wallet: &Pubkey,
    ) -> Result<(), TestError> {
        let vault = self.get_vault(&vault_root.vault_pubkey).await.unwrap();
        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            vault_staker_withdrawal_ticket_base,
        )
        .0;

        self.burn_withdrawal_ticket(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
            &vault.vrt_mint,
            &staker.pubkey(),
            &get_associated_token_address(&staker.pubkey(), &vault.supported_mint),
            &vault_staker_withdrawal_ticket,
            &get_associated_token_address(&vault_staker_withdrawal_ticket, &vault.vrt_mint),
            &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
            &get_associated_token_address(program_fee_wallet, &vault.vrt_mint),
        )
        .await?;

        Ok(())
    }

    pub async fn burn_withdrawal_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        vault_token_account: &Pubkey,
        vrt_mint: &Pubkey,
        staker: &Pubkey,
        staker_token_account: &Pubkey,
        vault_staker_withdrawal_ticket: &Pubkey,
        vault_staker_withdrawal_ticket_token_account: &Pubkey,
        vault_fee_token_account: &Pubkey,
        program_fee_vrt_token_account: &Pubkey,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::burn_withdrawal_ticket(
                &jito_vault_program::id(),
                config,
                vault,
                vault_token_account,
                vrt_mint,
                staker,
                staker_token_account,
                vault_staker_withdrawal_ticket,
                vault_staker_withdrawal_ticket_token_account,
                vault_fee_token_account,
                program_fee_vrt_token_account,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        ))
        .await
    }

    pub async fn add_delegation(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
        vault_operator_delegation: &Pubkey,
        admin: &Keypair,
        amount: u64,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[add_delegation(
                &jito_vault_program::id(),
                config,
                vault,
                operator,
                vault_operator_delegation,
                &admin.pubkey(),
                amount,
            )],
            Some(&admin.pubkey()),
            &[admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_mint_to(
        &mut self,
        vault_root: &VaultRoot,
        depositor: &Keypair,
        amount_in: u64,
        min_amount_out: u64,
    ) -> TestResult<()> {
        let vault = self.get_vault(&vault_root.vault_pubkey).await.unwrap();
        self.mint_to(
            &vault_root.vault_pubkey,
            &vault.vrt_mint,
            depositor,
            &get_associated_token_address(&depositor.pubkey(), &vault.supported_mint),
            &get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint),
            &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint),
            &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint),
            None,
            amount_in,
            min_amount_out,
        )
        .await
    }

    pub async fn mint_to(
        &mut self,
        vault: &Pubkey,
        vrt_mint: &Pubkey,
        depositor: &Keypair,
        depositor_token_account: &Pubkey,
        vault_token_account: &Pubkey,
        depositor_vrt_token_account: &Pubkey,
        vault_fee_token_account: &Pubkey,
        mint_signer: Option<&Keypair>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        let mut signers = vec![depositor];
        if let Some(signer) = mint_signer {
            signers.push(signer);
        }
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::mint_to(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                vault,
                vrt_mint,
                &depositor.pubkey(),
                depositor_token_account,
                vault_token_account,
                depositor_vrt_token_account,
                vault_fee_token_account,
                mint_signer.map(|s| s.pubkey()).as_ref(),
                amount_in,
                min_amount_out,
            )],
            Some(&depositor.pubkey()),
            &signers,
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault_ncn_slasher_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        ncn_slasher_ticket: &Pubkey,
        vault_slasher_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::initialize_vault_ncn_slasher_ticket(
                &jito_vault_program::id(),
                config,
                vault,
                ncn,
                slasher,
                ncn_slasher_ticket,
                vault_slasher_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault_ncn_slasher_operator_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        vault_ncn_slasher_ticket: &Pubkey,
        vault_ncn_slasher_operator_ticket: &Pubkey,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[
                jito_vault_sdk::sdk::initialize_vault_ncn_slasher_operator_ticket(
                    &jito_vault_program::id(),
                    config,
                    vault,
                    ncn,
                    slasher,
                    operator,
                    vault_ncn_slasher_ticket,
                    vault_ncn_slasher_operator_ticket,
                    &payer.pubkey(),
                ),
            ],
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ))
        .await
    }

    pub async fn slash(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher: &Keypair,
        ncn_operator_state: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        operator_vault_ticket: &Pubkey,
        vault_ncn_ticket: &Pubkey,
        vault_operator_delegation: &Pubkey,
        ncn_vault_slasher_ticket: &Pubkey,
        vault_ncn_slasher_ticket: &Pubkey,
        vault_ncn_slasher_operator_ticket: &Pubkey,
        vault_token_account: &Pubkey,
        slasher_token_account: &Pubkey,
        amount: u64,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::slash(
                &jito_vault_program::id(),
                config,
                vault,
                ncn,
                operator,
                &slasher.pubkey(),
                ncn_operator_state,
                ncn_vault_ticket,
                operator_vault_ticket,
                vault_ncn_ticket,
                vault_operator_delegation,
                ncn_vault_slasher_ticket,
                vault_ncn_slasher_ticket,
                vault_ncn_slasher_operator_ticket,
                vault_token_account,
                slasher_token_account,
                amount,
            )],
            Some(&slasher.pubkey()),
            &[slasher],
            blockhash,
        ))
        .await
    }

    pub async fn create_token_metadata(
        &mut self,
        vault: &Pubkey,
        admin: &Keypair,
        vrt_mint: &Pubkey,
        payer: &Keypair,
        metadata: &Pubkey,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        let new_blockhash = self
            .banks_client
            .get_new_latest_blockhash(&blockhash)
            .await
            .unwrap();

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::create_token_metadata(
                &jito_vault_program::id(),
                vault,
                &admin.pubkey(),
                vrt_mint,
                &payer.pubkey(),
                metadata,
                name,
                symbol,
                uri,
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            new_blockhash,
        ))
        .await
    }

    pub async fn update_token_metadata(
        &mut self,
        vault: &Pubkey,
        admin: &Keypair,
        vrt_mint: &Pubkey,
        metadata: &Pubkey,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::update_token_metadata(
                &jito_vault_program::id(),
                vault,
                &admin.pubkey(),
                vrt_mint,
                metadata,
                name,
                symbol,
                uri,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer, admin],
            blockhash,
        ))
        .await
    }

    async fn _process_transaction(&mut self, tx: &Transaction) -> Result<(), TestError> {
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    pub async fn airdrop(&mut self, to: &Pubkey, sol: f64) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[transfer(&self.payer.pubkey(), to, sol_to_lamports(sol))],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    pub async fn create_token_mint(
        &mut self,
        mint: &Keypair,
        token_program_id: &Pubkey,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        let rent: Rent = self.banks_client.get_sysvar().await?;
        let ixs = if token_program_id.eq(&spl_token::id()) {
            vec![
                create_account(
                    &self.payer.pubkey(),
                    &mint.pubkey(),
                    rent.minimum_balance(spl_token::state::Mint::LEN),
                    spl_token::state::Mint::LEN as u64,
                    token_program_id,
                ),
                spl_token::instruction::initialize_mint2(
                    token_program_id,
                    &mint.pubkey(),
                    &self.payer.pubkey(),
                    None,
                    9,
                )
                .unwrap(),
            ]
        } else {
            let space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
                ExtensionType::MintCloseAuthority,
            ])
            .unwrap();
            vec![
                create_account(
                    &self.payer.pubkey(),
                    &mint.pubkey(),
                    rent.minimum_balance(space),
                    space as u64,
                    token_program_id,
                ),
                spl_token_2022::instruction::initialize_mint_close_authority(
                    token_program_id,
                    &mint.pubkey(),
                    None,
                )
                .unwrap(),
                spl_token_2022::instruction::initialize_mint(
                    token_program_id,
                    &mint.pubkey(),
                    &self.payer.pubkey(),
                    None,
                    9,
                )
                .unwrap(),
            ]
        };
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &ixs,
                    Some(&self.payer.pubkey()),
                    &[&self.payer, mint],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    pub async fn create_ata(&mut self, mint: &Pubkey, owner: &Pubkey) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[create_associated_token_account_idempotent(
                        &self.payer.pubkey(),
                        owner,
                        mint,
                        &spl_token::id(),
                    )],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    /// Mints tokens to an ATA owned by the `to` address
    pub async fn mint_spl_to(
        &mut self,
        mint: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[
                        create_associated_token_account_idempotent(
                            &self.payer.pubkey(),
                            to,
                            mint,
                            &spl_token::id(),
                        ),
                        spl_token::instruction::mint_to(
                            &spl_token::id(),
                            mint,
                            &get_associated_token_address(to, mint),
                            &self.payer.pubkey(),
                            &[],
                            amount,
                        )
                        .unwrap(),
                    ],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn get_reward_fee_token_account(
        &mut self,
        vault: &Pubkey,
    ) -> Result<SPLTokenAccount, BanksClientError> {
        let vault = self.get_vault(vault).await.unwrap();

        let vault_fee_token_account =
            get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint);

        let account = self
            .banks_client
            .get_account(vault_fee_token_account)
            .await
            .unwrap()
            .unwrap();

        Ok(SPLTokenAccount::unpack(&account.data).unwrap())
    }

    pub async fn create_and_fund_reward_vault(
        &mut self,
        vault: &Pubkey,
        rewarder: &Keypair,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let vault_account = self.get_vault(vault).await.unwrap();

        let rewarder_token_account =
            get_associated_token_address(&rewarder.pubkey(), &vault_account.supported_mint);

        let vault_token_account =
            get_associated_token_address(vault, &vault_account.supported_mint);

        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[
                        create_associated_token_account_idempotent(
                            &rewarder.pubkey(),
                            &vault_token_account,
                            &vault_account.supported_mint,
                            &spl_token::id(),
                        ),
                        spl_token::instruction::transfer(
                            &spl_token::id(),
                            &rewarder_token_account,
                            &vault_token_account,
                            &rewarder.pubkey(),
                            &[],
                            amount,
                        )
                        .unwrap(),
                    ],
                    Some(&rewarder.pubkey()),
                    &[&rewarder],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn set_program_fee_wallet(
        &mut self,
        program_fee_admin: &Keypair,
        new_fee_wallet: &Pubkey,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::set_program_fee_wallet(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                &program_fee_admin.pubkey(),
                new_fee_wallet,
            )],
            Some(&program_fee_admin.pubkey()),
            &[program_fee_admin],
            blockhash,
        ))
        .await
    }

    pub async fn set_is_paused(
        &mut self,
        vault: &Pubkey,
        admin: &Keypair,
        is_paused: bool,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::sdk::set_is_paused(
                &jito_vault_program::id(),
                &Config::find_program_address(&jito_vault_program::id()).0,
                vault,
                &admin.pubkey(),
                is_paused,
            )],
            Some(&admin.pubkey()),
            &[admin],
            blockhash,
        ))
        .await
    }
}

#[inline(always)]
#[track_caller]
pub fn assert_vault_error<T>(test_error: Result<T, TestError>, vault_error: VaultError) {
    assert!(test_error.is_err());
    assert_eq!(
        test_error.err().unwrap().to_transaction_error().unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(vault_error as u32))
    );
}
