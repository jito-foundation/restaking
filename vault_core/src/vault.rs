//! The vault is responsible for holding tokens and minting VRT tokens.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::loader::load_signer;
use jito_vault_sdk::error::VaultError;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::delegation_state::DelegationState;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
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
    pub vrt_supply: u64,

    /// The total number of tokens deposited
    pub tokens_deposited: u64,

    /// Max capacity of tokens in the vault
    pub capacity: u64,

    /// Rolled-up stake state for all operators in the set
    pub delegation_state: DelegationState,

    /// The amount of VRT tokens in VaultStakerWithdrawalTickets enqueued for cooldown
    pub vrt_enqueued_for_cooldown_amount: u64,

    /// The amount of VRT tokens cooling down
    pub vrt_cooling_down_amount: u64,

    /// The amount of VRT tokens ready to claim
    pub vrt_ready_to_claim_amount: u64,

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

    /// The admin responsible for withdrawing tokens
    pub withdraw_admin: Pubkey,

    /// Fee wallet account
    pub fee_wallet: Pubkey,

    /// Optional mint signer
    pub mint_burn_admin: Pubkey,

    // ------------------------------------------
    // Indexing and counters
    // These are helpful when one needs to iterate through all the accounts
    // ------------------------------------------
    /// The index of the vault in the vault list
    pub vault_index: u64,

    /// Number of VaultNcnTicket accounts tracked by this vault
    pub ncn_count: u64,

    /// Number of VaultOperatorDelegation accounts tracked by this vault
    pub operator_count: u64,

    /// Number of VaultNcnSlasherTicket accounts tracked by this vault
    pub slasher_count: u64,

    /// The slot of the last fee change
    pub last_fee_change_slot: u64,

    /// The slot of the last time the delegations were updated
    pub last_full_state_update_slot: u64,

    /// The deposit fee in basis points
    pub deposit_fee_bps: u16,

    /// The withdrawal fee in basis points
    pub withdrawal_fee_bps: u16,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 11],
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
        bump: u8,
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
            withdraw_admin: admin,
            fee_wallet: admin,
            mint_burn_admin: Pubkey::default(),
            capacity: u64::MAX,
            vault_index,
            vrt_supply: 0,
            tokens_deposited: 0,
            vrt_enqueued_for_cooldown_amount: 0,
            vrt_cooling_down_amount: 0,
            last_fee_change_slot: 0,
            last_full_state_update_slot: 0,
            deposit_fee_bps,
            withdrawal_fee_bps,
            ncn_count: 0,
            operator_count: 0,
            slasher_count: 0,
            bump,
            reserved: [0; 11],
            delegation_state: DelegationState::default(),
            vrt_ready_to_claim_amount: 0,
        }
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

        if self.withdraw_admin.eq(old_admin) {
            self.withdraw_admin = *new_admin;
            msg!("Withdraw admin set to {:?}", new_admin);
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
    pub fn is_update_needed(&self, slot: u64, epoch_length: u64) -> bool {
        let last_updated_epoch = self
            .last_full_state_update_slot
            .checked_div(epoch_length)
            .unwrap();
        let current_epoch = slot.checked_div(epoch_length).unwrap();
        last_updated_epoch < current_epoch
    }

    #[inline(always)]
    pub fn check_update_state_ok(&self, slot: u64, epoch_length: u64) -> Result<(), ProgramError> {
        if self.is_update_needed(slot, epoch_length) {
            msg!("Vault update is needed");
            return Err(VaultError::VaultUpdateNeeded.into());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn check_mint_burn_admin(
        &self,
        mint_burn_admin: Option<&AccountInfo>,
    ) -> Result<(), ProgramError> {
        if self.mint_burn_admin.ne(&Pubkey::default()) {
            if let Some(burn_signer) = mint_burn_admin {
                load_signer(burn_signer, false)?;
                if burn_signer.key.ne(&self.mint_burn_admin) {
                    msg!("Burn signer does not match vault burn signer");
                    return Err(VaultError::VaultMintBurnAdminInvalid.into());
                }
            } else {
                msg!("Mint signer is required for vault mint");
                return Err(VaultError::VaultMintBurnAdminInvalid.into());
            }
        }
        Ok(())
    }

    // ------------------------------------------
    // Minting and burning
    // ------------------------------------------

    /// Calculate the maximum amount of tokens that can be withdrawn from the vault given the VRT
    /// amount. This is the pro-rata share of the total tokens deposited in the vault.
    pub fn calculate_assets_returned_amount(&self, vrt_amount: u64) -> Result<u64, VaultError> {
        vrt_amount
            .checked_mul(self.tokens_deposited)
            .and_then(|x| x.checked_div(self.vrt_supply))
            .ok_or(VaultError::VaultOverflow)
    }

    /// Calculate the amount of VRT tokens to mint based on the amount of tokens deposited in the vault.
    /// If no tokens have been deposited, the amount is equal to the amount passed in.
    /// Otherwise, the amount is calculated as the pro-rata share of the total VRT supply.
    pub fn calculate_vrt_mint_amount(&self, amount: u64) -> Result<u64, VaultError> {
        if self.tokens_deposited == 0 {
            return Ok(amount);
        }

        amount
            .checked_mul(self.vrt_supply)
            .and_then(|x| x.checked_div(self.tokens_deposited))
            .ok_or(VaultError::VaultOverflow)
    }

    /// Calculate the amount of tokens collected as a fee for depositing tokens in the vault.
    pub fn calculate_deposit_fee(&self, vrt_amount: u64) -> Result<u64, VaultError> {
        let fee = vrt_amount
            .checked_mul(self.deposit_fee_bps as u64)
            .ok_or(VaultError::VaultOverflow)?
            .div_ceil(10_000);
        Ok(fee)
    }

    /// Calculate the amount of tokens collected as a fee for withdrawing tokens from the vault.
    pub fn calculate_withdraw_fee(&self, vrt_amount: u64) -> Result<u64, VaultError> {
        let fee = vrt_amount
            .checked_mul(self.withdrawal_fee_bps as u64)
            .ok_or(VaultError::VaultOverflow)?
            .div_ceil(10_000);
        Ok(fee)
    }

    pub fn mint_with_fee(
        &mut self,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<MintSummary, VaultError> {
        let vault_token_amount_after_deposit = self
            .tokens_deposited
            .checked_add(amount_in)
            .ok_or(VaultError::VaultOverflow)?;
        if vault_token_amount_after_deposit > self.capacity {
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

        self.vrt_supply = self
            .vrt_supply
            .checked_add(vrt_mint_amount)
            .ok_or(VaultError::VaultOverflow)?;
        self.tokens_deposited = vault_token_amount_after_deposit;

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
            return Err(VaultError::VaultUnderflow);
        }
        if amount_in > self.vrt_supply {
            msg!("Amount exceeds vault VRT supply");
            return Err(VaultError::VaultInsufficientFunds);
        }
        let fee_amount = self.calculate_withdraw_fee(amount_in)?;
        let amount_to_burn = amount_in
            .checked_sub(fee_amount)
            .ok_or(VaultError::VaultUnderflow)?;

        let amount_out = amount_to_burn
            .checked_mul(self.tokens_deposited)
            .and_then(|x| x.checked_div(self.vrt_supply))
            .ok_or(VaultError::VaultOverflow)?;

        let max_withdrawable = self
            .tokens_deposited
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

        self.vrt_supply = self
            .vrt_supply
            .checked_sub(amount_to_burn)
            .ok_or(VaultError::VaultUnderflow)?;
        self.tokens_deposited = self
            .tokens_deposited
            .checked_sub(amount_out)
            .ok_or(VaultError::VaultUnderflow)?;

        Ok(BurnSummary {
            fee_amount,
            burn_amount: amount_to_burn,
            out_amount: amount_out,
        })
    }

    pub fn delegate(&mut self, amount: u64) -> Result<(), VaultError> {
        let assets_available_for_staking = self
            .tokens_deposited
            .checked_sub(self.delegation_state.total_security()?)
            .ok_or(VaultError::VaultUnderflow)?;

        if amount > assets_available_for_staking {
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

    use jito_vault_sdk::error::VaultError;
    use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

    use crate::{
        delegation_state::DelegationState,
        vault::{BurnSummary, MintSummary, Vault},
    };

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
        );
        vault.mint_burn_admin = old_admin;

        assert_eq!(vault.delegation_admin, old_admin);
        assert_eq!(vault.operator_admin, old_admin);
        assert_eq!(vault.ncn_admin, old_admin);
        assert_eq!(vault.slasher_admin, old_admin);
        assert_eq!(vault.capacity_admin, old_admin);
        assert_eq!(vault.fee_wallet, old_admin);
        assert_eq!(vault.mint_burn_admin, old_admin);
        assert_eq!(vault.withdraw_admin, old_admin);
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
        assert_eq!(vault.withdraw_admin, new_admin);
        assert_eq!(vault.fee_admin, new_admin);
    }

    #[test]
    fn test_mint_simple_ok() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        let MintSummary {
            vrt_to_depositor,
            vrt_to_fee_wallet,
        } = vault.mint_with_fee(100, 100).unwrap();
        assert_eq!(vrt_to_depositor, 100);
        assert_eq!(vrt_to_fee_wallet, 0);
    }

    #[test]
    fn test_mint_with_deposit_fee_ok() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            100,
            0,
            0,
        );
        let MintSummary {
            vrt_to_depositor,
            vrt_to_fee_wallet,
        } = vault.mint_with_fee(100, 99).unwrap();
        assert_eq!(vrt_to_depositor, 99);
        assert_eq!(vrt_to_fee_wallet, 1);
        assert_eq!(vault.tokens_deposited, 100);
        assert_eq!(vault.vrt_supply, 100);
    }

    #[test]
    fn test_mint_less_than_slippage_fails() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            100,
            1,
            0,
        );
        assert_eq!(
            vault.mint_with_fee(100, 100),
            Err(VaultError::SlippageError)
        );
    }

    #[test]
    fn test_deposit_ratio_after_slashed_ok() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.tokens_deposited = 90;
        vault.vrt_supply = 100;

        let MintSummary {
            vrt_to_depositor, ..
        } = vault.mint_with_fee(100, 111).unwrap();
        assert_eq!(vrt_to_depositor, 111);
        assert_eq!(vault.tokens_deposited, 190);
        assert_eq!(vault.vrt_supply, 211);
    }

    #[test]
    fn test_deposit_ratio_after_reward_ok() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.tokens_deposited = 200;
        vault.vrt_supply = 100;

        let MintSummary {
            vrt_to_depositor, ..
        } = vault.mint_with_fee(100, 50).unwrap();
        assert_eq!(vrt_to_depositor, 50);
        assert_eq!(vault.tokens_deposited, 300);
        assert_eq!(vault.vrt_supply, 150);
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
        );
        vault.mint_burn_admin = Pubkey::new_unique();
        let err = vault.check_mint_burn_admin(None).unwrap_err();
        assert_eq!(
            err,
            ProgramError::Custom(VaultError::VaultMintBurnAdminInvalid.into())
        );
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
        assert_eq!(err, ProgramError::MissingRequiredSignature);
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
        assert_eq!(
            err,
            ProgramError::Custom(VaultError::VaultMintBurnAdminInvalid.into())
        );
    }

    #[test]
    fn test_burn_with_fee_ok() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            100,
            0,
        );
        vault.tokens_deposited = 100;
        vault.vrt_supply = 100;

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
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            100,
            0,
        );
        vault.tokens_deposited = 100;
        vault.vrt_supply = 100;

        assert_eq!(
            vault.burn_with_fee(101, 100),
            Err(VaultError::VaultInsufficientFunds)
        );
    }

    #[test]
    fn test_burn_zero_fails() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            100,
            0,
        );
        vault.tokens_deposited = 100;
        vault.vrt_supply = 100;
        assert_eq!(vault.burn_with_fee(0, 0), Err(VaultError::VaultUnderflow));
    }

    #[test]
    fn test_burn_slippage_exceeded_fails() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            100,
            0,
        );
        vault.tokens_deposited = 100;
        vault.vrt_supply = 100;
        assert_eq!(
            vault.burn_with_fee(100, 100),
            Err(VaultError::SlippageError)
        );
    }

    #[test]
    fn test_burn_with_delegation_ok() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.vrt_supply = 100;
        vault.tokens_deposited = 100;

        vault.delegation_state = DelegationState {
            staked_amount: 10,
            enqueued_for_cooldown_amount: 10,
            cooling_down_amount: 10,
            enqueued_for_withdraw_amount: 10,
            cooling_down_for_withdraw_amount: 10,
        };

        let BurnSummary {
            fee_amount,
            burn_amount,
            out_amount,
        } = vault.burn_with_fee(50, 50).unwrap();
        assert_eq!(fee_amount, 0);
        assert_eq!(burn_amount, 50);
        assert_eq!(out_amount, 50);
        assert_eq!(vault.tokens_deposited, 50);
        assert_eq!(vault.vrt_supply, 50);
    }

    #[test]
    fn test_burn_more_than_withdrawable_fails() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.vrt_supply = 100;
        vault.tokens_deposited = 100;

        vault.delegation_state = DelegationState {
            staked_amount: 10,
            enqueued_for_cooldown_amount: 10,
            cooling_down_amount: 10,
            enqueued_for_withdraw_amount: 10,
            cooling_down_for_withdraw_amount: 10,
        };

        assert_eq!(vault.burn_with_fee(51, 50), Err(VaultError::VaultUnderflow));
    }

    #[test]
    fn test_burn_all_delegated() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.vrt_supply = 100;
        vault.tokens_deposited = 100;
        vault.delegation_state = DelegationState {
            staked_amount: 100,
            enqueued_for_cooldown_amount: 0,
            cooling_down_amount: 0,
            enqueued_for_withdraw_amount: 0,
            cooling_down_for_withdraw_amount: 0,
        };

        let result = vault.burn_with_fee(1, 0);
        assert_eq!(result, Err(VaultError::VaultUnderflow));
    }

    #[test]
    fn test_burn_rounding_issues() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.vrt_supply = 1_000_000;
        vault.tokens_deposited = 1_000_000;

        let result = vault.burn_with_fee(1, 0).unwrap();
        assert_eq!(result.out_amount, 1);
        assert_eq!(vault.tokens_deposited, 999_999);
        assert_eq!(vault.vrt_supply, 999_999);
    }

    #[test]
    fn test_burn_max_values() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.vrt_supply = u64::MAX;
        vault.tokens_deposited = u64::MAX;

        assert_eq!(
            vault.burn_with_fee(u64::MAX, u64::MAX - 1).unwrap_err(),
            VaultError::VaultOverflow
        );
    }

    #[test]
    fn test_burn_different_fees() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            500, // 5% withdrawal fee
            0,
        );
        vault.vrt_supply = 10000;
        vault.tokens_deposited = 10000;

        let result = vault.burn_with_fee(1000, 900).unwrap();
        assert_eq!(result.fee_amount, 50);
        assert_eq!(result.burn_amount, 950);
        assert_eq!(result.out_amount, 950);
    }

    #[test]
    fn test_mint_at_max_capacity() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.capacity = 1000;
        vault.vrt_supply = 1000;
        vault.tokens_deposited = 900;

        let result = vault.mint_with_fee(100, 111).unwrap();
        assert_eq!(result.vrt_to_depositor, 111);
        assert_eq!(vault.tokens_deposited, 1000);

        // Attempt to mint beyond capacity
        let result = vault.mint_with_fee(1, 1);
        assert_eq!(result, Err(VaultError::VaultCapacityExceeded));
    }

    #[test]
    fn test_mint_small_amounts() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.tokens_deposited = 1_000_000;
        vault.vrt_supply = 1_000_000;

        let result = vault.mint_with_fee(1, 1).unwrap();
        assert_eq!(result.vrt_to_depositor, 1);
        assert_eq!(vault.tokens_deposited, 1_000_001);
        assert_eq!(vault.vrt_supply, 1_000_001);
    }

    #[test]
    fn test_mint_different_fees() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            500, // 5% deposit fee
            0,
            0,
        );

        let result = vault.mint_with_fee(1000, 950).unwrap();
        assert_eq!(result.vrt_to_depositor, 950);
        assert_eq!(result.vrt_to_fee_wallet, 50);
        assert_eq!(vault.tokens_deposited, 1000);
        assert_eq!(vault.vrt_supply, 1000);
    }

    #[test]
    fn test_mint_empty_vault() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );

        let result = vault.mint_with_fee(1000, 1000).unwrap();
        assert_eq!(result.vrt_to_depositor, 1000);
        assert_eq!(result.vrt_to_fee_wallet, 0);
        assert_eq!(vault.tokens_deposited, 1000);
        assert_eq!(vault.vrt_supply, 1000);
    }

    #[test]
    fn test_mint_slippage_protection() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            100, // 1% deposit fee
            0,
            0,
        );
        vault.tokens_deposited = 10000;
        vault.vrt_supply = 10000;

        // Successful mint within slippage tolerance
        let result = vault.mint_with_fee(1000, 990).unwrap();
        assert_eq!(result.vrt_to_depositor, 990);

        // Failed mint due to slippage
        let result = vault.mint_with_fee(1000, 991);
        assert_eq!(result, Err(VaultError::SlippageError));
    }

    #[test]
    fn test_mint_small_fee() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            1,
            0,
            0,
        );
        let MintSummary {
            vrt_to_depositor,
            vrt_to_fee_wallet,
        } = vault.mint_with_fee(1, 0).unwrap();
        assert_eq!(vrt_to_depositor, 0);
        assert_eq!(vrt_to_fee_wallet, 1);
    }

    #[test]
    fn test_burn_small_fee() {
        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            1,
            0,
        );
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
}
