//! The vault is responsible for holding tokens and minting VRT tokens.
use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{
    types::{PodU16, PodU64},
    AccountDeserialize, Discriminator,
};
use jito_jsm_core::loader::load_signer;
use jito_vault_sdk::error::VaultError;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::{delegation_state::DelegationState, MAX_FEE_BPS};

#[derive(Debug, PartialEq, Eq)]
pub struct BurnSummary {
    /// How much of the VRT shall be transferred to the vault fee account
    pub fee_amount: u64,
    /// How much of the staker's VRT shall be burned
    pub burn_amount: u64,
    /// How much of the staker's tokens shall be returned
    pub out_amount: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MintSummary {
    pub vrt_to_depositor: u64,
    pub vrt_to_fee_wallet: u64,
}

impl Discriminator for Vault {
    const DISCRIMINATOR: u8 = 2;
}

/// The vault is responsible for holding tokens and minting VRT tokens
/// based on the amount of tokens deposited.
/// It also contains several administrative functions for features inside the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Vault {
    /// The base account of the VRT
    pub base: Pubkey,

    // ------------------------------------------
    // Token information and accounting
    // ------------------------------------------
    /// Mint of the VRT token
    pub vrt_mint: Pubkey,

    /// Mint of the token that is supported by the VRT
    pub supported_mint: Pubkey,

    /// The total number of VRT in circulation
    vrt_supply: PodU64,

    /// The total number of tokens deposited
    tokens_deposited: PodU64,

    /// The maximum deposit capacity allowed in the mint_to instruction.
    /// The deposited assets in the vault may exceed the deposit_capacity during other operations, such as vault balance updates.
    deposit_capacity: PodU64,

    /// Rolled-up stake state for all operators in the set
    pub delegation_state: DelegationState,

    /// The amount of VRT tokens in VaultStakerWithdrawalTickets enqueued for cooldown
    vrt_enqueued_for_cooldown_amount: PodU64,

    /// The amount of VRT tokens cooling down
    vrt_cooling_down_amount: PodU64,

    /// The amount of VRT tokens ready to claim
    vrt_ready_to_claim_amount: PodU64,

    // ------------------------------------------
    // Admins
    // ------------------------------------------
    /// Vault admin
    pub admin: Pubkey,

    /// The delegation admin responsible for adding and removing delegations from operators.
    pub delegation_admin: Pubkey,

    /// The operator admin responsible for adding and removing operators.
    pub operator_admin: Pubkey,

    /// The node consensus network admin responsible for adding and removing support for NCNs.
    pub ncn_admin: Pubkey,

    /// The admin responsible for adding and removing slashers.
    pub slasher_admin: Pubkey,

    /// The admin responsible for setting the capacity
    pub capacity_admin: Pubkey,

    /// The admin responsible for setting the fees
    pub fee_admin: Pubkey,

    /// The delegate_admin responsible for delegating assets
    pub delegate_asset_admin: Pubkey,

    /// Fee wallet account
    pub fee_wallet: Pubkey,

    /// Optional mint signer
    pub mint_burn_admin: Pubkey,

    // ------------------------------------------
    // Indexing and counters
    // These are helpful when one needs to iterate through all the accounts
    // ------------------------------------------
    /// The index of the vault in the vault list
    vault_index: PodU64,

    /// Number of VaultNcnTicket accounts tracked by this vault
    ncn_count: PodU64,

    /// Number of VaultOperatorDelegation accounts tracked by this vault
    operator_count: PodU64,

    /// Number of VaultNcnSlasherTicket accounts tracked by this vault
    slasher_count: PodU64,

    /// The slot of the last fee change
    last_fee_change_slot: PodU64,

    /// The slot of the last time the delegations were updated
    last_full_state_update_slot: PodU64,

    /// The deposit fee in basis points
    deposit_fee_bps: PodU16,

    /// The withdrawal fee in basis points
    withdrawal_fee_bps: PodU16,

    /// Fee for each epoch
    reward_fee_bps: PodU16,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 263],
}

impl Vault {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vrt_mint: Pubkey,
        supported_mint: Pubkey,
        admin: Pubkey,
        vault_index: u64,
        base: Pubkey,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        bump: u8,
        current_slot: u64,
    ) -> Self {
        Self {
            base,
            vrt_mint,
            supported_mint,
            admin,
            delegation_admin: admin,
            operator_admin: admin,
            ncn_admin: admin,
            slasher_admin: admin,
            capacity_admin: admin,
            fee_admin: admin,
            delegate_asset_admin: admin,
            fee_wallet: admin,
            mint_burn_admin: Pubkey::default(),
            deposit_capacity: PodU64::from(u64::MAX),
            vault_index: PodU64::from(vault_index),
            vrt_supply: PodU64::from(0),
            tokens_deposited: PodU64::from(0),
            vrt_enqueued_for_cooldown_amount: PodU64::from(0),
            vrt_cooling_down_amount: PodU64::from(0),
            vrt_ready_to_claim_amount: PodU64::from(0),
            last_fee_change_slot: PodU64::from(current_slot),
            last_full_state_update_slot: PodU64::from(current_slot),
            deposit_fee_bps: PodU16::from(deposit_fee_bps),
            withdrawal_fee_bps: PodU16::from(withdrawal_fee_bps),
            reward_fee_bps: PodU16::from(reward_fee_bps),
            ncn_count: PodU64::from(0),
            operator_count: PodU64::from(0),
            slasher_count: PodU64::from(0),
            bump,
            delegation_state: DelegationState::default(),
            reserved: [0; 263],
        }
    }

    pub fn ncn_count(&self) -> u64 {
        self.ncn_count.into()
    }

    pub fn last_fee_change_slot(&self) -> u64 {
        self.last_fee_change_slot.into()
    }

    pub fn deposit_capacity(&self) -> u64 {
        self.deposit_capacity.into()
    }

    pub fn vault_index(&self) -> u64 {
        self.vault_index.into()
    }

    pub fn set_last_fee_change_slot(&mut self, slot: u64) {
        self.last_fee_change_slot = PodU64::from(slot);
    }

    pub fn last_full_state_update_slot(&self) -> u64 {
        self.last_full_state_update_slot.into()
    }

    pub fn vrt_supply(&self) -> u64 {
        self.vrt_supply.into()
    }

    pub fn slasher_count(&self) -> u64 {
        self.slasher_count.into()
    }

    pub fn tokens_deposited(&self) -> u64 {
        self.tokens_deposited.into()
    }

    pub fn increment_tokens_deposited(&mut self, amount: u64) -> Result<(), VaultError> {
        let mut tokens_deposited: u64 = self.tokens_deposited.into();
        tokens_deposited = tokens_deposited
            .checked_add(amount)
            .ok_or(VaultError::VaultOverflow)?;
        self.tokens_deposited = PodU64::from(tokens_deposited);
        Ok(())
    }

    pub fn decrement_tokens_deposited(&mut self, amount: u64) -> Result<(), VaultError> {
        let mut tokens_deposited: u64 = self.tokens_deposited.into();
        tokens_deposited = tokens_deposited
            .checked_sub(amount)
            .ok_or(VaultError::VaultUnderflow)?;
        self.tokens_deposited = PodU64::from(tokens_deposited);
        Ok(())
    }

    pub fn increment_slasher_count(&mut self) -> Result<(), VaultError> {
        let mut slasher_count: u64 = self.slasher_count.into();
        slasher_count = slasher_count
            .checked_add(1)
            .ok_or(VaultError::SlasherOverflow)?;
        self.slasher_count = PodU64::from(slasher_count);
        Ok(())
    }

    pub fn increment_ncn_count(&mut self) -> Result<(), VaultError> {
        let mut ncn_count: u64 = self.ncn_count.into();
        ncn_count = ncn_count.checked_add(1).ok_or(VaultError::NcnOverflow)?;
        self.ncn_count = PodU64::from(ncn_count);
        Ok(())
    }

    pub fn increment_operator_count(&mut self) -> Result<(), VaultError> {
        let mut operator_count: u64 = self.operator_count.into();
        operator_count = operator_count
            .checked_add(1)
            .ok_or(VaultError::OperatorOverflow)?;
        self.operator_count = PodU64::from(operator_count);
        Ok(())
    }

    pub fn vrt_enqueued_for_cooldown_amount(&self) -> u64 {
        self.vrt_enqueued_for_cooldown_amount.into()
    }

    pub fn vrt_cooling_down_amount(&self) -> u64 {
        self.vrt_cooling_down_amount.into()
    }

    pub fn vrt_ready_to_claim_amount(&self) -> u64 {
        self.vrt_ready_to_claim_amount.into()
    }

    pub fn deposit_fee_bps(&self) -> u16 {
        self.deposit_fee_bps.into()
    }

    pub fn withdrawal_fee_bps(&self) -> u16 {
        self.withdrawal_fee_bps.into()
    }

    pub fn reward_fee_bps(&self) -> u16 {
        self.reward_fee_bps.into()
    }

    pub fn operator_count(&self) -> u64 {
        self.operator_count.into()
    }

    pub fn set_capacity(&mut self, capacity: u64) {
        self.deposit_capacity = PodU64::from(capacity);
    }

    pub fn set_vrt_cooling_down_amount(&mut self, amount: u64) {
        self.vrt_cooling_down_amount = PodU64::from(amount);
    }

    pub fn increment_vrt_supply(&mut self, amount: u64) -> Result<(), VaultError> {
        let mut vrt_supply: u64 = self.vrt_supply.into();
        vrt_supply = vrt_supply
            .checked_add(amount)
            .ok_or(VaultError::VaultOverflow)?;
        self.vrt_supply = PodU64::from(vrt_supply);
        Ok(())
    }

    pub fn decrement_vrt_supply(&mut self, amount: u64) -> Result<(), VaultError> {
        let mut vrt_supply: u64 = self.vrt_supply.into();
        vrt_supply = vrt_supply
            .checked_sub(amount)
            .ok_or(VaultError::VaultUnderflow)?;
        self.vrt_supply = PodU64::from(vrt_supply);
        Ok(())
    }

    pub fn set_last_full_state_update_slot(&mut self, slot: u64) {
        self.last_full_state_update_slot = PodU64::from(slot);
    }

    pub fn decrement_vrt_ready_to_claim_amount(&mut self, amount: u64) -> Result<(), VaultError> {
        let mut vrt_ready_to_claim_amount: u64 = self.vrt_ready_to_claim_amount.into();
        vrt_ready_to_claim_amount = vrt_ready_to_claim_amount
            .checked_sub(amount)
            .ok_or(VaultError::VaultUnderflow)?;
        self.vrt_ready_to_claim_amount = PodU64::from(vrt_ready_to_claim_amount);
        Ok(())
    }

    pub fn increment_vrt_ready_to_claim_amount(&mut self, amount: u64) -> Result<(), VaultError> {
        let mut vrt_ready_to_claim_amount: u64 = self.vrt_ready_to_claim_amount.into();
        vrt_ready_to_claim_amount = vrt_ready_to_claim_amount
            .checked_add(amount)
            .ok_or(VaultError::VaultOverflow)?;
        self.vrt_ready_to_claim_amount = PodU64::from(vrt_ready_to_claim_amount);
        Ok(())
    }

    pub fn increment_vrt_enqueued_for_cooldown_amount(
        &mut self,
        amount: u64,
    ) -> Result<(), VaultError> {
        let mut vrt_enqueued_for_cooldown_amount: u64 =
            self.vrt_enqueued_for_cooldown_amount.into();
        vrt_enqueued_for_cooldown_amount = vrt_enqueued_for_cooldown_amount
            .checked_add(amount)
            .ok_or(VaultError::VaultOverflow)?;
        self.vrt_enqueued_for_cooldown_amount = PodU64::from(vrt_enqueued_for_cooldown_amount);
        Ok(())
    }

    pub fn set_vrt_enqueued_for_cooldown_amount(&mut self, amount: u64) {
        self.vrt_enqueued_for_cooldown_amount = PodU64::from(amount);
    }

    pub fn set_tokens_deposited(&mut self, tokens_deposited: u64) {
        self.tokens_deposited = PodU64::from(tokens_deposited);
    }

    pub fn set_vrt_supply(&mut self, vrt_supply: u64) {
        self.vrt_supply = PodU64::from(vrt_supply);
    }

    pub fn check_vrt_mint(&self, vrt_mint: &Pubkey) -> Result<(), ProgramError> {
        if self.vrt_mint.ne(vrt_mint) {
            msg!("Vault VRT mint does not match the provided VRT mint");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    /// Check admin validity and signature
    #[inline(always)]
    pub fn check_admin(&self, admin: &Pubkey) -> Result<(), ProgramError> {
        if self.admin.ne(admin) {
            msg!("Vault admin does not match the provided admin");
            return Err(VaultError::VaultAdminInvalid.into());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn check_delegation_admin(&self, delegation_admin: &Pubkey) -> Result<(), VaultError> {
        if self.delegation_admin.ne(delegation_admin) {
            msg!("Vault delegation admin does not match the provided delegation admin");
            return Err(VaultError::VaultDelegationAdminInvalid);
        }
        Ok(())
    }

    pub fn check_operator_admin(&self, operator_admin: &Pubkey) -> Result<(), VaultError> {
        if self.operator_admin.ne(operator_admin) {
            msg!("Vault operator admin does not match the provided operator admin");
            return Err(VaultError::VaultOperatorAdminInvalid);
        }
        Ok(())
    }

    pub fn check_ncn_admin(&self, ncn_admin: &Pubkey) -> Result<(), VaultError> {
        if self.ncn_admin.ne(ncn_admin) {
            msg!("Vault NCN admin does not match the provided NCN admin");
            return Err(VaultError::VaultNcnAdminInvalid);
        }
        Ok(())
    }

    pub fn check_slasher_admin(&self, slasher_admin: &Pubkey) -> Result<(), VaultError> {
        if self.slasher_admin.ne(slasher_admin) {
            msg!("Vault slasher admin does not match the provided slasher admin");
            return Err(VaultError::VaultSlasherAdminInvalid);
        }
        Ok(())
    }

    pub fn check_capacity_admin(&self, capacity_admin: &Pubkey) -> Result<(), VaultError> {
        if self.capacity_admin.ne(capacity_admin) {
            msg!("Vault capacity admin does not match the provided capacity admin");
            return Err(VaultError::VaultCapacityAdminInvalid);
        }
        Ok(())
    }

    pub fn check_fee_admin(&self, fee_admin: &Pubkey) -> Result<(), VaultError> {
        if self.fee_admin.ne(fee_admin) {
            msg!("Vault fee admin does not match the provided fee admin");
            return Err(VaultError::VaultFeeAdminInvalid);
        }
        Ok(())
    }

    /// Validates the delegate_asset_admin account and ensures it matches the expected delegate_asset_admin.
    ///
    /// # Arguments
    /// * `delegate_asset_admin` - A reference to the [`Pubkey`] representing the delegate_asset_admin Pubkey that is attempting
    ///   to authorize the operation.
    ///
    /// # Returns
    /// * `Result<(), VaultError>` - Returns `Ok(())` if the delegate_asset_admin Pubkey is valid.
    ///
    /// # Errors
    /// This function will return a [`jito_vault_sdk::error::VaultError::VaultDelegateAssetAdminInvalid`] error in the following case:
    /// * The `delegate_asset_admin` 's public key does not match the expected delegate_asset_admin public key stored in `self`.
    pub fn check_delegate_asset_admin(
        &self,
        delegate_asset_admin: &Pubkey,
    ) -> Result<(), VaultError> {
        if self.delegate_asset_admin.ne(delegate_asset_admin) {
            msg!("Vault delegate asset admin does not match the provided delegate asset admin");
            return Err(VaultError::VaultDelegateAssetAdminInvalid);
        }
        Ok(())
    }

    /// Replace all secondary admins that were equal to the old admin to the new admin
    pub fn update_secondary_admin(&mut self, old_admin: &Pubkey, new_admin: &Pubkey) {
        if self.delegation_admin.eq(old_admin) {
            self.delegation_admin = *new_admin;
            msg!("Delegation admin set to {:?}", new_admin);
        }

        if self.operator_admin.eq(old_admin) {
            self.operator_admin = *new_admin;
            msg!("Operator admin set to {:?}", new_admin);
        }

        if self.ncn_admin.eq(old_admin) {
            self.ncn_admin = *new_admin;
            msg!("Ncn admin set to {:?}", new_admin);
        }

        if self.slasher_admin.eq(old_admin) {
            self.slasher_admin = *new_admin;
            msg!("Slasher admin set to {:?}", new_admin);
        }

        if self.capacity_admin.eq(old_admin) {
            self.capacity_admin = *new_admin;
            msg!("Capacity admin set to {:?}", new_admin);
        }

        if self.fee_wallet.eq(old_admin) {
            self.fee_wallet = *new_admin;
            msg!("Fee wallet set to {:?}", new_admin);
        }

        if self.mint_burn_admin.eq(old_admin) {
            self.mint_burn_admin = *new_admin;
            msg!("Mint burn admin set to {:?}", new_admin);
        }

        if self.delegate_asset_admin.eq(old_admin) {
            self.delegate_asset_admin = *new_admin;
            msg!("Delegate asset admin set to {:?}", new_admin);
        }

        if self.fee_admin.eq(old_admin) {
            self.fee_admin = *new_admin;
            msg!("Fee admin set to {:?}", new_admin);
        }
    }

    // ------------------------------------------
    // Asset accounting and tracking
    // ------------------------------------------

    #[inline(always)]
    fn is_update_needed(&self, slot: u64, epoch_length: u64) -> Result<bool, ProgramError> {
        let last_updated_epoch = self
            .last_full_state_update_slot()
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;
        let current_epoch = slot
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;
        Ok(last_updated_epoch < current_epoch)
    }

    #[inline(always)]
    pub fn check_update_state_ok(&self, slot: u64, epoch_length: u64) -> Result<(), ProgramError> {
        if self.is_update_needed(slot, epoch_length)? {
            msg!("Vault update is needed");
            return Err(VaultError::VaultUpdateNeeded.into());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn check_mint_burn_admin(
        &self,
        mint_burn_admin: Option<&AccountInfo>,
    ) -> Result<(), VaultError> {
        if self.mint_burn_admin.ne(&Pubkey::default()) {
            if let Some(burn_signer) = mint_burn_admin {
                load_signer(burn_signer, false)
                    .map_err(|_| VaultError::VaultMintBurnAdminInvalid)?;
                if burn_signer.key.ne(&self.mint_burn_admin) {
                    msg!("Burn signer does not match vault burn signer");
                    return Err(VaultError::VaultMintBurnAdminInvalid);
                }
            } else {
                msg!("Mint signer is required for vault mint");
                return Err(VaultError::VaultMintBurnAdminInvalid);
            }
        }
        Ok(())
    }

    // ------------------------------------------
    // Fees
    // ------------------------------------------

    /// Fees can be changed at most one per epoch, and a **full** epoch must pass before a fee can be changed again.
    #[inline(always)]
    pub fn check_can_modify_fees(&self, slot: u64, epoch_length: u64) -> Result<(), VaultError> {
        let current_epoch = slot
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;
        let last_fee_change_epoch = self
            .last_fee_change_slot()
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;

        if current_epoch
            <= last_fee_change_epoch
                .checked_add(1)
                .ok_or(VaultError::ArithmeticOverflow)?
        {
            msg!("Fee changes are only allowed once per epoch");
            return Err(VaultError::VaultFeeChangeTooSoon);
        }

        Ok(())
    }

    pub fn set_withdrawal_fee_bps(
        &mut self,
        withdrawal_fee_bps: u16,
        deposit_withdrawal_fee_cap_bps: u16,
        fee_bump_bps: u16,
        fee_rate_of_change_bps: u16,
    ) -> Result<(), VaultError> {
        if withdrawal_fee_bps > MAX_FEE_BPS {
            msg!("Withdrawal fee exceeds maximum allowed of {}", MAX_FEE_BPS);
            return Err(VaultError::VaultFeeCapExceeded);
        } else if withdrawal_fee_bps > deposit_withdrawal_fee_cap_bps {
            msg!(
                "Withdrawal fee exceeds maximum allowed of {}",
                deposit_withdrawal_fee_cap_bps
            );
            return Err(VaultError::VaultFeeCapExceeded);
        }
        Self::check_fee_change_ok(
            self.withdrawal_fee_bps(),
            withdrawal_fee_bps,
            deposit_withdrawal_fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps,
        )?;
        self.withdrawal_fee_bps = PodU16::from(withdrawal_fee_bps);
        Ok(())
    }

    pub fn set_deposit_fee_bps(
        &mut self,
        deposit_fee_bps: u16,
        deposit_withdrawal_fee_cap_bps: u16,
        fee_bump_bps: u16,
        fee_rate_of_change_bps: u16,
    ) -> Result<(), VaultError> {
        if deposit_fee_bps > MAX_FEE_BPS {
            msg!("Deposit fee exceeds maximum allowed of {}", MAX_FEE_BPS);
            return Err(VaultError::VaultFeeCapExceeded);
        } else if deposit_fee_bps > deposit_withdrawal_fee_cap_bps {
            msg!(
                "Deposit fee exceeds maximum allowed of {}",
                deposit_withdrawal_fee_cap_bps
            );
            return Err(VaultError::VaultFeeCapExceeded);
        }

        Self::check_fee_change_ok(
            self.deposit_fee_bps(),
            deposit_fee_bps,
            deposit_withdrawal_fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps,
        )?;

        self.deposit_fee_bps = PodU16::from(deposit_fee_bps);
        Ok(())
    }

    pub fn set_reward_fee_bps(&mut self, reward_fee_bps: u16) -> Result<(), VaultError> {
        if reward_fee_bps > MAX_FEE_BPS {
            msg!("Reward fee exceeds maximum allowed of {}", MAX_FEE_BPS);
            return Err(VaultError::VaultFeeCapExceeded);
        }
        self.reward_fee_bps = PodU16::from(reward_fee_bps);
        Ok(())
    }

    fn check_fee_change_ok(
        current_fee_bps: u16,
        new_fee_bps: u16,
        fee_cap_bps: u16,
        fee_bump_bps: u16,
        fee_rate_of_change_bps: u16,
    ) -> Result<(), VaultError> {
        let fee_delta = new_fee_bps.saturating_sub(current_fee_bps);
        let fee_cap_bps = fee_cap_bps.min(MAX_FEE_BPS);

        if new_fee_bps > fee_cap_bps {
            msg!("Fee exceeds maximum allowed of {}", fee_cap_bps);
            return Err(VaultError::VaultFeeCapExceeded);
        }

        if fee_delta > fee_bump_bps {
            let deposit_percentage_increase_bps: u64 = (fee_delta as u128)
                .checked_mul(MAX_FEE_BPS as u128)
                .and_then(|product| product.checked_div(current_fee_bps as u128))
                .and_then(|result| result.try_into().ok())
                .unwrap_or(u64::MAX); // Divide by zero should result in max value

            if deposit_percentage_increase_bps > fee_rate_of_change_bps as u64 {
                msg!(
                    "Fee increase exceeds maximum rate of change {} bps or flat bump of {} bps",
                    fee_rate_of_change_bps,
                    fee_bump_bps
                );
                return Err(VaultError::VaultFeeBumpTooLarge);
            }
        }

        Ok(())
    }

    // ------------------------------------------
    // Minting and burning
    // ------------------------------------------

    /// Calculate the rewards fee. This is used in `update_vault_balance` to mint
    /// the `fee` amount in VRTs to the `fee_wallet`.
    pub fn calculate_rewards_fee(&self, new_balance: u64) -> Result<u64, VaultError> {
        let rewards = new_balance.saturating_sub(self.tokens_deposited());

        let vrt_rewards = self.calculate_vrt_mint_amount(rewards)?;

        if vrt_rewards == 0 {
            return Ok(0);
        }

        let fee = (vrt_rewards as u128)
            .checked_mul(self.reward_fee_bps() as u128)
            .map(|x| x.div_ceil(MAX_FEE_BPS as u128))
            .and_then(|x| x.try_into().ok())
            .ok_or(VaultError::VaultOverflow)?;

        Ok(fee)
    }

    /// Calculate the amount of VRT tokens to mint based on the amount of tokens deposited in the vault.
    /// If no tokens have been deposited, the amount is equal to the amount passed in.
    /// Otherwise, the amount is calculated as the pro-rata share of the total VRT supply.
    fn calculate_vrt_mint_amount(&self, amount: u64) -> Result<u64, VaultError> {
        if self.tokens_deposited() == 0 {
            return Ok(amount);
        }

        (amount as u128)
            .checked_mul(self.vrt_supply() as u128)
            .and_then(|x| x.checked_div(self.tokens_deposited() as u128))
            .and_then(|result| result.try_into().ok())
            .ok_or(VaultError::VaultOverflow)
    }

    /// Calculate the amount of tokens collected as a fee for depositing tokens in the vault.
    fn calculate_deposit_fee(&self, vrt_amount: u64) -> Result<u64, VaultError> {
        let fee = (vrt_amount as u128)
            .checked_mul(self.deposit_fee_bps() as u128)
            .map(|x| x.div_ceil(MAX_FEE_BPS as u128))
            .and_then(|x| x.try_into().ok())
            .ok_or(VaultError::VaultOverflow)?;
        Ok(fee)
    }

    /// Calculate the amount of tokens collected as a fee for withdrawing tokens from the vault.
    fn calculate_withdrawal_fee(&self, vrt_amount: u64) -> Result<u64, VaultError> {
        let fee = (vrt_amount as u128)
            .checked_mul(self.withdrawal_fee_bps() as u128)
            .map(|x| x.div_ceil(MAX_FEE_BPS as u128))
            .and_then(|x| x.try_into().ok())
            .ok_or(VaultError::VaultOverflow)?;
        Ok(fee)
    }

    pub fn mint_with_fee(
        &mut self,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<MintSummary, VaultError> {
        if amount_in == 0 {
            msg!("Amount in is zero");
            return Err(VaultError::VaultMintZero);
        }

        let vault_token_amount_after_deposit = self
            .tokens_deposited()
            .checked_add(amount_in)
            .ok_or(VaultError::VaultOverflow)?;
        if vault_token_amount_after_deposit > self.deposit_capacity() {
            msg!("Amount exceeds vault capacity");
            return Err(VaultError::VaultCapacityExceeded);
        }

        let vrt_mint_amount = self.calculate_vrt_mint_amount(amount_in)?;
        let vrt_to_fee_wallet = self.calculate_deposit_fee(vrt_mint_amount)?;
        let vrt_to_depositor = vrt_mint_amount
            .checked_sub(vrt_to_fee_wallet)
            .ok_or(VaultError::VaultUnderflow)?;

        if vrt_to_depositor < min_amount_out {
            msg!(
                "Slippage error, expected more than {} out, got {}",
                min_amount_out,
                vrt_to_depositor
            );
            return Err(VaultError::SlippageError);
        }

        let vrt_supply = self
            .vrt_supply()
            .checked_add(vrt_mint_amount)
            .ok_or(VaultError::VaultOverflow)?;
        self.vrt_supply = PodU64::from(vrt_supply);
        self.tokens_deposited = PodU64::from(vault_token_amount_after_deposit);

        Ok(MintSummary {
            vrt_to_depositor,
            vrt_to_fee_wallet,
        })
    }

    pub fn burn_with_fee(
        &mut self,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<BurnSummary, VaultError> {
        if amount_in == 0 {
            msg!("Amount in is zero");
            return Err(VaultError::VaultBurnZero);
        } else if amount_in > self.vrt_supply() {
            msg!("Amount exceeds vault VRT supply");
            return Err(VaultError::VaultInsufficientFunds);
        }

        let fee_amount = self.calculate_withdrawal_fee(amount_in)?;
        let amount_to_burn = amount_in
            .checked_sub(fee_amount)
            .ok_or(VaultError::VaultUnderflow)?;

        let amount_out = (amount_to_burn as u128)
            .checked_mul(self.tokens_deposited() as u128)
            .and_then(|x| x.checked_div(self.vrt_supply() as u128))
            .and_then(|x| x.try_into().ok())
            .ok_or(VaultError::VaultOverflow)?;

        let max_withdrawable = self
            .tokens_deposited()
            .checked_sub(self.delegation_state.total_security()?)
            .ok_or(VaultError::VaultUnderflow)?;

        // The vault shall not be able to withdraw more than the max withdrawable amount
        if amount_out > max_withdrawable {
            msg!("Amount out exceeds max withdrawable amount");
            return Err(VaultError::VaultUnderflow);
        }

        // Slippage check
        if amount_out < min_amount_out {
            msg!(
                "Slippage error, expected more than {} out, got {}",
                min_amount_out,
                amount_out
            );
            return Err(VaultError::SlippageError);
        }

        let vrt_supply = self
            .vrt_supply()
            .checked_sub(amount_to_burn)
            .ok_or(VaultError::VaultUnderflow)?;
        self.vrt_supply = PodU64::from(vrt_supply);

        let tokens_deposited = self
            .tokens_deposited()
            .checked_sub(amount_out)
            .ok_or(VaultError::VaultUnderflow)?;
        self.tokens_deposited = PodU64::from(tokens_deposited);

        Ok(BurnSummary {
            fee_amount,
            burn_amount: amount_to_burn,
            out_amount: amount_out,
        })
    }

    /// Calculates the amount of tokens, denominated in the supported_mint asset,
    /// that should be reserved for the VRTs in the vault
    pub fn calculate_vrt_reserve_amount(&self) -> Result<u64, VaultError> {
        if self.vrt_supply() == 0 {
            return Ok(0);
        }
        let vrt_reserve = self
            .vrt_enqueued_for_cooldown_amount()
            .checked_add(self.vrt_cooling_down_amount())
            .and_then(|x| x.checked_add(self.vrt_ready_to_claim_amount()))
            .ok_or(VaultError::VaultOverflow)?;

        let fee_amount = self.calculate_withdrawal_fee(vrt_reserve)?;

        let vrt_reserve_with_fee = vrt_reserve
            .checked_add(fee_amount)
            .ok_or(VaultError::VaultOverflow)?;

        let amount_to_reserve_for_vrts: u64 = (vrt_reserve_with_fee as u128)
            .checked_mul(self.tokens_deposited() as u128)
            .and_then(|x| x.checked_div(self.vrt_supply() as u128))
            .and_then(|result| result.try_into().ok())
            .ok_or(VaultError::VaultOverflow)?;

        Ok(amount_to_reserve_for_vrts)
    }

    pub fn calculate_assets_needed_for_withdrawals(
        &self,
        slot: u64,
        epoch_length: u64,
    ) -> Result<u64, VaultError> {
        // Calculate the total amount of assets needed to be set aside for all potential withdrawals
        let amount_needed_set_aside_for_withdrawals = self.calculate_vrt_reserve_amount()?;

        // Clone the current delegation state to simulate updates without modifying the original
        let mut delegation_state_after_update = self.delegation_state;

        // Calculate the epoch of the last full state update and the current epoch
        let last_epoch_update = self
            .last_full_state_update_slot()
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;
        let this_epoch = slot
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;

        // Update the simulated delegation state based on the number of epochs passed
        let epoch_diff = this_epoch
            .checked_sub(last_epoch_update)
            .ok_or(VaultError::ArithmeticUnderflow)?;
        match epoch_diff {
            0 => {
                // no-op
            }
            1 => {
                delegation_state_after_update.update();
            }
            _ => {
                // More than one epoch has passed, but we only need to update twice at most
                // (enqueued -> cooling down and cooling down -> not allocated)
                delegation_state_after_update.update();
                delegation_state_after_update.update();
            }
        }

        // Calculate the total amount of assets delegated after the simulated update
        let total_delegated_after_update = delegation_state_after_update.total_security()?;

        // Calculate the amount of assets that are not delegated after the simulated update
        let undelegated_after_update = self
            .tokens_deposited()
            .checked_sub(total_delegated_after_update)
            .ok_or(VaultError::VaultUnderflow)?;

        // Calculate the total amount of assets that are in the process of being withdrawn
        // after the simulated update
        let assets_withdrawing_after_update = delegation_state_after_update
            .enqueued_for_cooldown_amount()
            .checked_add(delegation_state_after_update.cooling_down_amount())
            .ok_or(VaultError::VaultOverflow)?;

        // Calculate the total amount of assets available for withdrawal, which includes
        // both undelegated assets and assets in the withdrawal process
        let available_for_withdrawal = undelegated_after_update
            .checked_add(assets_withdrawing_after_update)
            .ok_or(VaultError::VaultOverflow)?;

        // Calculate how many additional assets need to be undelegated to meet the withdrawal needs
        // If available assets exceed the needed amount, this will be zero due to saturating subtraction
        let additional_assets_need_undelegating =
            amount_needed_set_aside_for_withdrawals.saturating_sub(available_for_withdrawal);

        Ok(additional_assets_need_undelegating)
    }

    pub fn delegate(&mut self, amount: u64) -> Result<(), VaultError> {
        if amount == 0 {
            msg!("Delegation amount is zero");
            return Err(VaultError::VaultDelegationZero);
        } else if self.tokens_deposited() == 0 || self.vrt_supply() == 0 {
            msg!("No tokens deposited in vault");
            return Err(VaultError::VaultUnderflow);
        }

        // there is some protection built-in to the vault to avoid over delegating assets
        // this number is denominated in the supported token units
        let amount_to_reserve_for_vrts = self.calculate_vrt_reserve_amount()?;

        let amount_available_for_delegation = self
            .tokens_deposited()
            .checked_sub(self.delegation_state.total_security()?)
            .and_then(|x| x.checked_sub(amount_to_reserve_for_vrts))
            .ok_or(VaultError::VaultUnderflow)?;

        if amount > amount_available_for_delegation {
            msg!("Insufficient funds in vault for delegation");
            return Err(VaultError::VaultInsufficientFunds);
        }

        self.delegation_state.delegate(amount)?;

        Ok(())
    }

    // ------------------------------------------
    // Serialization & Deserialization
    // ------------------------------------------

    /// Returns the seeds for the PDA
    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"vault".as_ref().to_vec(), base.to_bytes().to_vec()]
    }

    /// Returns the seeds for the PDA used for signing
    pub fn signing_seeds(&self) -> Vec<Vec<u8>> {
        let mut vault_seeds = Self::seeds(&self.base);
        vault_seeds.push(vec![self.bump]);
        vault_seeds
    }

    /// Find the program address for the Vault
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the [`Vault`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        account: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if account.owner.ne(program_id) {
            msg!("Vault account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("Vault account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("Vault account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Vault account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let base = Self::try_from_slice_unchecked(&account.data.borrow())?.base;
        if account
            .key
            .ne(&Self::find_program_address(program_id, &base).0)
        {
            msg!("Vault account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use jito_bytemuck::types::{PodU16, PodU64};
    use jito_vault_sdk::error::VaultError;
    use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

    use crate::{
        delegation_state::DelegationState,
        vault::{BurnSummary, MintSummary, Vault},
        MAX_FEE_BPS,
    };

    fn make_test_vault(
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        tokens_deposited: u64,
        vrt_supply: u64,
        delegation_state: DelegationState,
    ) -> Vault {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            deposit_fee_bps,
            withdrawal_fee_bps,
            0,
            0,
            0,
        );

        vault.set_tokens_deposited(tokens_deposited);
        vault.set_vrt_supply(vrt_supply);
        vault.delegation_state = delegation_state;
        vault
    }

    #[test]
    fn test_vault_no_padding() {
        let vault_size = std::mem::size_of::<Vault>();
        let sum_of_fields = std::mem::size_of::<Pubkey>() + // base
            std::mem::size_of::<Pubkey>() + // vrt_mint
            std::mem::size_of::<Pubkey>() + // supported_mint
            std::mem::size_of::<PodU64>() + // vrt_supply
            std::mem::size_of::<PodU64>() + // tokens_deposited
            std::mem::size_of::<PodU64>() + // capacity
            std::mem::size_of::<DelegationState>() + // delegation_state
            std::mem::size_of::<PodU64>() + // vrt_enqueued_for_cooldown_amount
            std::mem::size_of::<PodU64>() + // vrt_cooling_down_amount
            std::mem::size_of::<PodU64>() + // vrt_ready_to_claim_amount
            std::mem::size_of::<Pubkey>() + // admin
            std::mem::size_of::<Pubkey>() + // delegation_admin
            std::mem::size_of::<Pubkey>() + // operator_admin
            std::mem::size_of::<Pubkey>() + // ncn_admin
            std::mem::size_of::<Pubkey>() + // slasher_admin
            std::mem::size_of::<Pubkey>() + // capacity_admin
            std::mem::size_of::<Pubkey>() + // fee_admin
            std::mem::size_of::<Pubkey>() + // delegate_asset_admin
            std::mem::size_of::<Pubkey>() + // fee_wallet
            std::mem::size_of::<Pubkey>() + // mint_burn_admin
            std::mem::size_of::<PodU64>() + // vault_index
            std::mem::size_of::<PodU64>() + // ncn_count
            std::mem::size_of::<PodU64>() + // operator_count
            std::mem::size_of::<PodU64>() + // slasher_count
            std::mem::size_of::<PodU64>() + // last_fee_change_slot
            std::mem::size_of::<PodU64>() + // last_full_state_update_slot
            std::mem::size_of::<PodU16>() + // deposit_fee_bps
            std::mem::size_of::<PodU16>() + // withdrawal_fee_bps
            std::mem::size_of::<PodU16>() + // reward_fee_bps
            1 + // bump
            263; // reserved

        assert_eq!(vault_size, sum_of_fields);
    }

    #[test]
    fn test_update_secondary_admin_ok() {
        let old_admin = Pubkey::new_unique();
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            old_admin,
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            0,
            0,
        );
        vault.mint_burn_admin = old_admin;

        assert_eq!(vault.delegation_admin, old_admin);
        assert_eq!(vault.operator_admin, old_admin);
        assert_eq!(vault.ncn_admin, old_admin);
        assert_eq!(vault.slasher_admin, old_admin);
        assert_eq!(vault.capacity_admin, old_admin);
        assert_eq!(vault.fee_wallet, old_admin);
        assert_eq!(vault.mint_burn_admin, old_admin);
        assert_eq!(vault.delegate_asset_admin, old_admin);
        assert_eq!(vault.fee_admin, old_admin);

        let new_admin = Pubkey::new_unique();
        vault.update_secondary_admin(&old_admin, &new_admin);

        assert_eq!(vault.delegation_admin, new_admin);
        assert_eq!(vault.operator_admin, new_admin);
        assert_eq!(vault.ncn_admin, new_admin);
        assert_eq!(vault.slasher_admin, new_admin);
        assert_eq!(vault.capacity_admin, new_admin);
        assert_eq!(vault.fee_wallet, new_admin);
        assert_eq!(vault.mint_burn_admin, new_admin);
        assert_eq!(vault.delegate_asset_admin, new_admin);
        assert_eq!(vault.fee_admin, new_admin);
    }

    #[test]
    fn test_mint_simple_ok() {
        let mut vault = make_test_vault(0, 0, 0, 0, DelegationState::default());
        let MintSummary {
            vrt_to_depositor,
            vrt_to_fee_wallet,
        } = vault.mint_with_fee(100, 100).unwrap();
        assert_eq!(vrt_to_depositor, 100);
        assert_eq!(vrt_to_fee_wallet, 0);
    }

    #[test]
    fn test_mint_with_deposit_fee_ok() {
        let mut vault = make_test_vault(100, 0, 0, 0, DelegationState::default());
        let MintSummary {
            vrt_to_depositor,
            vrt_to_fee_wallet,
        } = vault.mint_with_fee(100, 99).unwrap();
        assert_eq!(vrt_to_depositor, 99);
        assert_eq!(vrt_to_fee_wallet, 1);
        assert_eq!(vault.tokens_deposited(), 100);
        assert_eq!(vault.vrt_supply(), 100);
    }

    #[test]
    fn test_mint_less_than_slippage_fails() {
        let mut vault = make_test_vault(100, 0, 0, 0, DelegationState::default());
        assert_eq!(
            vault.mint_with_fee(100, 100),
            Err(VaultError::SlippageError)
        );
    }

    #[test]
    fn test_deposit_ratio_after_slashed_ok() {
        let mut vault = make_test_vault(0, 0, 90, 100, DelegationState::default());

        let MintSummary {
            vrt_to_depositor, ..
        } = vault.mint_with_fee(100, 111).unwrap();
        assert_eq!(vrt_to_depositor, 111);
        assert_eq!(vault.tokens_deposited(), 190);
        assert_eq!(vault.vrt_supply(), 211);
    }

    #[test]
    fn test_deposit_ratio_after_reward_ok() {
        let mut vault = make_test_vault(0, 0, 200, 100, DelegationState::default());

        let MintSummary {
            vrt_to_depositor, ..
        } = vault.mint_with_fee(100, 50).unwrap();
        assert_eq!(vrt_to_depositor, 50);
        assert_eq!(vault.tokens_deposited(), 300);
        assert_eq!(vault.vrt_supply(), 150);
    }

    #[test]
    fn test_mint_burn_no_admin() {
        let vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            0,
            0,
        );
        assert_eq!(vault.check_mint_burn_admin(None), Ok(()));
    }

    #[test]
    fn test_mint_burn_signer_account_missing() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            0,
            0,
        );
        vault.mint_burn_admin = Pubkey::new_unique();
        let err = vault.check_mint_burn_admin(None).unwrap_err();
        assert_eq!(err, VaultError::VaultMintBurnAdminInvalid);
    }

    #[test]
    fn test_mint_burn_signer_address_not_signer() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            0,
            0,
        );
        vault.mint_burn_admin = Pubkey::new_unique();

        let mut binding_lamports = 0;
        let lamports = Rc::new(RefCell::new(&mut binding_lamports));
        let mut data: Vec<u8> = vec![0];
        let data = Rc::new(RefCell::new(data.as_mut_slice()));
        let not_signer = AccountInfo {
            key: &vault.mint_burn_admin,
            is_signer: false,
            is_writable: false,
            lamports,
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let err = vault.check_mint_burn_admin(Some(&not_signer)).unwrap_err();
        assert_eq!(err, VaultError::VaultMintBurnAdminInvalid);
    }

    #[test]
    fn test_mint_burn_signer_address_invalid() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            0,
            0,
        );
        vault.mint_burn_admin = Pubkey::new_unique();

        let mut binding_lamports = 0;
        let lamports = Rc::new(RefCell::new(&mut binding_lamports));
        let mut data: Vec<u8> = vec![0];
        let data = Rc::new(RefCell::new(data.as_mut_slice()));
        let wrong_address_and_signer = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: true,
            is_writable: false,
            lamports,
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let err = vault
            .check_mint_burn_admin(Some(&wrong_address_and_signer))
            .unwrap_err();
        assert_eq!(err, VaultError::VaultMintBurnAdminInvalid);
    }

    #[test]
    fn test_burn_with_fee_ok() {
        let mut vault = make_test_vault(0, 100, 100, 100, DelegationState::default());

        let BurnSummary {
            fee_amount,
            burn_amount,
            out_amount,
        } = vault.burn_with_fee(100, 99).unwrap();
        assert_eq!(fee_amount, 1);
        assert_eq!(burn_amount, 99);
        assert_eq!(out_amount, 99);
    }

    #[test]
    fn test_burn_too_much_fails() {
        let mut vault = make_test_vault(0, 100, 100, 100, DelegationState::default());

        assert_eq!(
            vault.burn_with_fee(101, 100),
            Err(VaultError::VaultInsufficientFunds)
        );
    }

    #[test]
    fn test_burn_zero_fails() {
        let mut vault = make_test_vault(0, 100, 100, 100, DelegationState::default());
        assert_eq!(vault.burn_with_fee(0, 0), Err(VaultError::VaultBurnZero));
    }

    #[test]
    fn test_burn_slippage_exceeded_fails() {
        let mut vault = make_test_vault(0, 100, 100, 100, DelegationState::default());
        assert_eq!(
            vault.burn_with_fee(100, 100),
            Err(VaultError::SlippageError)
        );
    }

    #[test]
    fn test_burn_with_delegation_ok() {
        let mut vault = make_test_vault(0, 0, 100, 100, DelegationState::new(10, 10, 0));

        let BurnSummary {
            fee_amount,
            burn_amount,
            out_amount,
        } = vault.burn_with_fee(50, 50).unwrap();
        assert_eq!(fee_amount, 0);
        assert_eq!(burn_amount, 50);
        assert_eq!(out_amount, 50);
        assert_eq!(vault.tokens_deposited(), 50);
        assert_eq!(vault.vrt_supply(), 50);
    }

    #[test]
    fn test_burn_more_than_withdrawable_fails() {
        let mut vault = make_test_vault(0, 0, 100, 100, DelegationState::new(50, 0, 0));

        assert_eq!(vault.burn_with_fee(51, 50), Err(VaultError::VaultUnderflow));
    }

    #[test]
    fn test_burn_all_delegated() {
        let mut vault = make_test_vault(0, 0, 100, 100, DelegationState::new(100, 0, 0));

        let result = vault.burn_with_fee(1, 0);
        assert_eq!(result, Err(VaultError::VaultUnderflow));
    }

    #[test]
    fn test_burn_rounding_issues() {
        let mut vault = make_test_vault(0, 0, 1_000_000, 1_000_000, DelegationState::default());

        let result = vault.burn_with_fee(1, 0).unwrap();
        assert_eq!(result.out_amount, 1);
        assert_eq!(vault.tokens_deposited(), 999_999);
        assert_eq!(vault.vrt_supply(), 999_999);
    }

    #[test]
    fn test_burn_max_values() {
        let mut vault = make_test_vault(0, 100, u64::MAX, u64::MAX, DelegationState::default());
        let result = vault.burn_with_fee(u64::MAX, 0).unwrap();
        let fee_amount = (((u64::MAX as u128) * 100).div_ceil(10000)) as u64;
        assert_eq!(result.fee_amount, fee_amount);
    }

    #[test]
    fn test_burn_different_fees() {
        let mut vault = make_test_vault(0, 500, 10000, 10000, DelegationState::default());

        let result = vault.burn_with_fee(1000, 900).unwrap();
        assert_eq!(result.fee_amount, 50);
        assert_eq!(result.burn_amount, 950);
        assert_eq!(result.out_amount, 950);
    }

    #[test]
    fn test_mint_at_max_capacity() {
        let mut vault = make_test_vault(0, 0, 900, 1000, DelegationState::default());
        vault.set_capacity(1000);

        let result = vault.mint_with_fee(100, 111).unwrap();
        assert_eq!(result.vrt_to_depositor, 111);
        assert_eq!(vault.tokens_deposited(), 1000);

        // Attempt to mint beyond capacity
        let result = vault.mint_with_fee(1, 1);
        assert_eq!(result, Err(VaultError::VaultCapacityExceeded));
    }

    #[test]
    fn test_mint_small_amounts() {
        let mut vault = make_test_vault(0, 0, 1_000_000, 1_000_000, DelegationState::default());

        let result = vault.mint_with_fee(1, 1).unwrap();
        assert_eq!(result.vrt_to_depositor, 1);
        assert_eq!(vault.tokens_deposited(), 1_000_001);
        assert_eq!(vault.vrt_supply(), 1_000_001);
    }

    #[test]
    fn test_mint_different_fees() {
        let mut vault = make_test_vault(500, 0, 0, 0, DelegationState::default());

        let result = vault.mint_with_fee(1000, 950).unwrap();
        assert_eq!(result.vrt_to_depositor, 950);
        assert_eq!(result.vrt_to_fee_wallet, 50);
        assert_eq!(vault.tokens_deposited(), 1000);
        assert_eq!(vault.vrt_supply(), 1000);
    }

    #[test]
    fn test_mint_empty_vault() {
        let mut vault = make_test_vault(0, 0, 0, 0, DelegationState::default());

        let result = vault.mint_with_fee(1000, 1000).unwrap();
        assert_eq!(result.vrt_to_depositor, 1000);
        assert_eq!(result.vrt_to_fee_wallet, 0);
        assert_eq!(vault.tokens_deposited(), 1000);
        assert_eq!(vault.vrt_supply(), 1000);
    }

    #[test]
    fn test_mint_slippage_protection() {
        let mut vault = make_test_vault(100, 0, 0, 0, DelegationState::default());

        // Successful mint within slippage tolerance
        let result = vault.mint_with_fee(1000, 990).unwrap();
        assert_eq!(result.vrt_to_depositor, 990);

        // Failed mint due to slippage
        let result = vault.mint_with_fee(1000, 991);
        assert_eq!(result, Err(VaultError::SlippageError));
    }

    #[test]
    fn test_mint_small_fee() {
        let mut vault = make_test_vault(1, 0, 0, 0, DelegationState::default());
        let MintSummary {
            vrt_to_depositor,
            vrt_to_fee_wallet,
        } = vault.mint_with_fee(1, 0).unwrap();
        assert_eq!(vrt_to_depositor, 0);
        assert_eq!(vrt_to_fee_wallet, 1);
    }

    #[test]
    fn test_burn_small_fee() {
        let mut vault = make_test_vault(0, 1, 0, 0, DelegationState::default());

        vault.mint_with_fee(1, 1).unwrap();
        let BurnSummary {
            fee_amount,
            burn_amount,
            out_amount,
        } = vault.burn_with_fee(1, 0).unwrap();
        assert_eq!(fee_amount, 1);
        assert_eq!(burn_amount, 0);
        assert_eq!(out_amount, 0);
    }

    #[test]
    fn test_delegate_ok() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());

        vault.delegate(1000).unwrap();
    }

    #[test]
    fn test_delegate_more_than_available_fails() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());
        assert_eq!(
            vault.delegate(1001),
            Err(VaultError::VaultInsufficientFunds)
        );
    }

    #[test]
    fn test_delegate_more_than_available_with_delegate_state_fails() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::new(500, 200, 200));
        assert_eq!(vault.delegate(101), Err(VaultError::VaultInsufficientFunds));
    }

    #[test]
    fn test_delegate_with_delegate_state_ok() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::new(500, 200, 100));
        vault.delegate(100).unwrap();
    }

    #[test]
    fn test_delegate_with_vrt_reserves_ok() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());
        vault.increment_vrt_ready_to_claim_amount(100).unwrap();

        vault.delegate(900).unwrap();
    }

    #[test]
    fn test_delegate_more_than_vrt_reserves_fails() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());
        vault.increment_vrt_ready_to_claim_amount(100).unwrap();

        assert_eq!(vault.delegate(901), Err(VaultError::VaultInsufficientFunds));
    }

    #[test]
    fn test_delegate_with_vrt_reserves_and_delegated_assets_ok() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::new(100, 100, 100));
        vault.increment_vrt_ready_to_claim_amount(100).unwrap();

        vault.delegate(400).unwrap();
    }

    #[test]
    fn test_delegate_with_vrt_reserves_and_delegated_assets_too_much_fails() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::new(100, 100, 100));
        vault.increment_vrt_ready_to_claim_amount(100).unwrap();

        assert_eq!(vault.delegate(601), Err(VaultError::VaultInsufficientFunds));
    }

    #[test]
    fn test_delegate_with_vrt_reserves_and_delegated_assets_cooling_down_fails() {
        let mut vault = make_test_vault(0, 0, 1000, 900, DelegationState::new(0, 500, 0));
        vault.increment_vrt_ready_to_claim_amount(500).unwrap();
        assert_eq!(vault.delegate(100), Err(VaultError::VaultUnderflow));
    }

    #[test]
    fn test_calculate_vrt_reserve_amount_ok() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());
        vault.set_vrt_cooling_down_amount(100);
        let result = vault.calculate_vrt_reserve_amount().unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn test_calculate_vrt_reserve_amount_with_fee() {
        let mut vault = make_test_vault(0, 100, 1000, 1000, DelegationState::default());
        vault.set_vrt_cooling_down_amount(100);
        let result = vault.calculate_vrt_reserve_amount().unwrap();

        // This is correct, because we need to account for the withdrawal fee
        // The withdrawal fee is 0.1% of the total amount, so 1000 * 0.001 = 1
        // The cooling down amount is 100, so we need to reserve 101
        assert_eq!(result, 101);
    }

    #[test]
    fn test_calculate_vrt_reserve_amount_with_fee_with_assets_in_different_stages() {
        let mut vault = make_test_vault(0, 100, 1000, 1000, DelegationState::default());
        vault.set_vrt_enqueued_for_cooldown_amount(50);
        vault.set_vrt_cooling_down_amount(25);
        vault.vrt_ready_to_claim_amount = PodU64::from(25);
        let result = vault.calculate_vrt_reserve_amount().unwrap();

        assert_eq!(result, 101);
    }

    #[test]
    fn test_calculate_assets_need_undelegating_ok() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::new(1000, 0, 0));
        vault.set_vrt_cooling_down_amount(100);
        let result = vault
            .calculate_assets_needed_for_withdrawals(100, 100)
            .unwrap();
        assert_eq!(result, 100);

        vault.delegation_state = DelegationState::new(900, 0, 100);
        let result = vault
            .calculate_assets_needed_for_withdrawals(100, 100)
            .unwrap();
        assert_eq!(result, 0);

        vault.set_vrt_cooling_down_amount(200);
        let result = vault
            .calculate_assets_needed_for_withdrawals(100, 100)
            .unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn test_calculate_assets_need_undelegating_with_assets_cooling_down() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::new(900, 0, 100));
        vault.set_vrt_cooling_down_amount(100);

        let result = vault
            .calculate_assets_needed_for_withdrawals(100, 100)
            .unwrap();
        assert_eq!(result, 0);

        let result = vault
            .calculate_assets_needed_for_withdrawals(200, 100)
            .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calculate_assets_need_undelegating_with_assets_cooling_down_2() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::new(800, 100, 100));
        vault.set_vrt_cooling_down_amount(300);

        let result = vault
            .calculate_assets_needed_for_withdrawals(100, 100)
            .unwrap();
        assert_eq!(result, 100);

        let result = vault
            .calculate_assets_needed_for_withdrawals(200, 100)
            .unwrap();
        assert_eq!(result, 100);

        vault.increment_vrt_supply(100).unwrap();
        vault.increment_tokens_deposited(100).unwrap();
        let result = vault
            .calculate_assets_needed_for_withdrawals(200, 100)
            .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calculate_reward_fee() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            1000, //10%
            0,
            0,
        );
        vault.set_tokens_deposited(0);

        let fee = vault.calculate_rewards_fee(1000).unwrap();

        assert_eq!(fee, 100);
    }

    #[test]
    fn test_calculate_negative_balance() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            1000, //10%
            0,
            0,
        );
        vault.set_tokens_deposited(1000);

        let fee = vault.calculate_rewards_fee(0).unwrap();

        assert_eq!(fee, 0);
    }

    #[test]
    fn test_calculate_100_percent_rewards() {
        let vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            10_000, //100%
            0,
            0,
        );

        let fee = vault.calculate_rewards_fee(1000).unwrap();

        assert_eq!(fee, 1000);
    }

    #[test]
    fn test_fee_change_after_two_epochs() {
        let mut vault = make_test_vault(0, 0, 0, 0, DelegationState::default());
        vault.last_fee_change_slot = PodU64::from(1);
        assert_eq!(vault.check_can_modify_fees(200, 100), Ok(()));
    }

    #[test]
    fn test_fee_change_within_same_epoch() {
        let mut vault = make_test_vault(0, 0, 0, 0, DelegationState::default());
        vault.last_fee_change_slot = PodU64::from(101);
        assert_eq!(
            vault.check_can_modify_fees(102, 100),
            Err(VaultError::VaultFeeChangeTooSoon)
        );
    }

    #[test]
    fn test_fee_change_in_next_epoch() {
        let mut vault = make_test_vault(0, 0, 0, 0, DelegationState::default());
        vault.last_fee_change_slot = PodU64::from(1);
        assert_eq!(
            vault.check_can_modify_fees(101, 100),
            Err(VaultError::VaultFeeChangeTooSoon)
        );
    }

    #[test]
    fn test_fee_change_at_epoch_boundary() {
        let mut vault = make_test_vault(0, 0, 0, 0, DelegationState::default());
        vault.last_fee_change_slot = PodU64::from(1);
        assert_eq!(
            vault.check_can_modify_fees(100, 100),
            Err(VaultError::VaultFeeChangeTooSoon)
        );
    }

    #[test]
    fn test_fee_increase_within_limits() {
        let current_fee_bps = 100;
        let new_fee_bps = 125;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: 25% increase <= 25% limit
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_fee_increase_outside_limits() {
        let current_fee_bps = 100;
        let new_fee_bps = 126;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // ERROR: 26% increase > 25% limit
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_fee_increase_inside_bump_limits() {
        let current_fee_bps = 1;
        let new_fee_bps = 10;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_fee_increase_outside_bump_limits() {
        let current_fee_bps = 1;
        let new_fee_bps = 13;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // ERROR: Δ > bump
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_zero_ok() {
        let current_fee_bps = 0;
        let new_fee_bps = 10;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_zero_bad() {
        let current_fee_bps = 0;
        let new_fee_bps = 11;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // Error: Δ > bump
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_no_difference() {
        let current_fee_bps = 100;
        let new_fee_bps = 100;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_decrease() {
        let current_fee_bps = 100;
        let new_fee_bps = 0;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_max_fee_values() {
        let max_fee_bps = MAX_FEE_BPS;

        let current_fee_bps = max_fee_bps - 1;
        let new_fee_bps = max_fee_bps;
        let fee_cap_bps = max_fee_bps;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_max_decrease() {
        let current_fee_bps = u16::MAX;
        let new_fee_bps = 0;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_max_increase() {
        let current_fee_bps = 0;
        let new_fee_bps = u16::MAX;
        let fee_cap_bps = u16::MAX;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_at_cap() {
        let current_fee_bps = 2999;
        let new_fee_bps = 3000;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_above_cap() {
        let current_fee_bps = 2999;
        let new_fee_bps = 3001;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(Vault::check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_delegation_too_small() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());
        assert_eq!(vault.delegate(0), Err(VaultError::VaultDelegationZero));
    }

    #[test]
    fn test_mint_with_fee_zero_amount() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());
        assert_eq!(vault.mint_with_fee(0, 0), Err(VaultError::VaultMintZero));
    }

    #[test]
    fn test_burn_with_fee_zero_amount() {
        let mut vault = make_test_vault(0, 0, 1000, 1000, DelegationState::default());
        assert_eq!(vault.burn_with_fee(0, 0), Err(VaultError::VaultBurnZero));
    }
}
