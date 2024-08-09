use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, msg, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

/// The vault is repsonsible for holding tokens and minting LRT tokens
/// based on the amount of tokens deposited.
/// It also contains several administrative functions for features inside the vault.
#[derive(Debug, Clone, Copy, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Vault {
    /// The account type
    account_type: AccountType,

    /// The base account of the LRT
    base: Pubkey,

    /// Mint of the LRT token
    lrt_mint: Pubkey,

    /// Mint of the token that is supported by the LRT
    supported_mint: Pubkey,

    /// Vault admin
    admin: Pubkey,

    /// The delegation admin responsible for adding and removing delegations from operators.
    delegation_admin: Pubkey,

    /// The operator admin responsible for adding and removing operators.
    operator_admin: Pubkey,

    /// The node consensus network admin responsible for adding and removing support for NCNs.
    ncn_admin: Pubkey,

    /// The admin responsible for adding and removing slashers.
    slasher_admin: Pubkey,

    /// Fee wallet account
    fee_wallet: Pubkey,

    /// Optional mint signer
    mint_burn_authority: Pubkey,

    /// Max capacity of tokens in the vault
    capacity: u64,

    /// The index of the vault in the vault list
    vault_index: u64,

    /// The total number of LRT in circulation
    lrt_supply: u64,

    /// The total number of tokens deposited
    tokens_deposited: u64,

    /// The amount of tokens that are reserved for withdrawal
    withdrawable_reserve_amount: u64,

    /// The deposit fee in basis points
    deposit_fee_bps: u16,

    /// The withdrawal fee in basis points
    withdrawal_fee_bps: u16,

    /// Number of VaultNcnTicket accounts tracked by this vault
    ncn_count: u64,

    /// Number of VaultOperatorTicket accounts tracked by this vault
    operator_count: u64,

    /// Number of VaultNcnSlasherTicket accounts tracked by this vault
    slasher_count: u64,

    /// Reserved space
    reserved: [u8; 128],

    /// The bump seed for the PDA
    bump: u8,
}

impl Vault {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lrt_mint: Pubkey,
        supported_mint: Pubkey,
        admin: Pubkey,
        vault_index: u64,
        base: Pubkey,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::Vault,
            base,
            lrt_mint,
            supported_mint,
            admin,
            delegation_admin: admin,
            operator_admin: admin,
            ncn_admin: admin,
            slasher_admin: admin,
            fee_wallet: admin,
            mint_burn_authority: Pubkey::default(),
            capacity: u64::MAX,
            vault_index,
            lrt_supply: 0,
            tokens_deposited: 0,
            withdrawable_reserve_amount: 0,
            deposit_fee_bps,
            withdrawal_fee_bps,
            ncn_count: 0,
            operator_count: 0,
            slasher_count: 0,
            reserved: [0; 128],
            bump,
        }
    }

    // ------------------------------------------
    // Asset accounting and tracking
    // ------------------------------------------

    pub const fn withdrawable_reserve_amount(&self) -> u64 {
        self.withdrawable_reserve_amount
    }

    pub fn increment_withdrawable_reserve_amount(&mut self, amount: u64) -> VaultCoreResult<()> {
        self.withdrawable_reserve_amount = self
            .withdrawable_reserve_amount
            .checked_add(amount)
            .ok_or(VaultCoreError::VaultDepositOverflow)?;
        Ok(())
    }

    pub fn decrement_withdrawable_reserve_amount(&mut self, amount: u64) -> VaultCoreResult<()> {
        self.withdrawable_reserve_amount = self
            .withdrawable_reserve_amount
            .checked_sub(amount)
            .ok_or(VaultCoreError::VaultDepositUnderflow)?;
        Ok(())
    }

    pub const fn tokens_deposited(&self) -> u64 {
        self.tokens_deposited
    }

    pub fn max_delegation_amount(&self) -> VaultCoreResult<u64> {
        self.tokens_deposited
            .checked_sub(self.withdrawable_reserve_amount)
            .ok_or(VaultCoreError::VaultDelegationUnderflow)
    }

    pub fn set_tokens_deposited(&mut self, tokens_deposited: u64) {
        self.tokens_deposited = tokens_deposited;
    }

    pub fn calculate_assets_returned_amount(&self, lrt_amount: u64) -> VaultCoreResult<u64> {
        if self.lrt_supply == 0 {
            return Err(VaultCoreError::VaultEmpty);
        } else if lrt_amount > self.lrt_supply {
            return Err(VaultCoreError::VaultInsufficientFunds);
        }

        lrt_amount
            .checked_mul(self.tokens_deposited)
            .ok_or(VaultCoreError::VaultWithdrawOverflow)?
            .checked_div(self.lrt_supply)
            .ok_or(VaultCoreError::VaultWithdrawOverflow)
    }

    pub fn calculate_lrt_mint_amount(&self, amount: u64) -> VaultCoreResult<u64> {
        if self.tokens_deposited == 0 {
            return Ok(amount);
        }

        amount
            .checked_mul(self.lrt_supply)
            .ok_or(VaultCoreError::VaultDepositOverflow)?
            .checked_div(self.tokens_deposited)
            .ok_or(VaultCoreError::VaultDepositOverflow)
    }

    pub const fn deposit_fee_bps(&self) -> u16 {
        self.deposit_fee_bps
    }

    pub fn calculate_deposit_fee(&self, lrt_amount: u64) -> VaultCoreResult<u64> {
        let fee = lrt_amount
            .checked_mul(self.deposit_fee_bps as u64)
            .ok_or(VaultCoreError::VaultFeeCalculationOverflow)?
            .div_ceil(10_000);
        Ok(fee)
    }

    pub const fn withdrawal_fee_bps(&self) -> u16 {
        self.withdrawal_fee_bps
    }

    pub fn calculate_withdraw_fee(&self, lrt_amount: u64) -> VaultCoreResult<u64> {
        let fee = lrt_amount
            .checked_mul(self.withdrawal_fee_bps as u64)
            .ok_or(VaultCoreError::VaultFeeCalculationOverflow)?
            .div_ceil(10_000);
        Ok(fee)
    }

    /// Deposit tokens into the vault
    pub fn deposit_and_mint_with_capacity_check(&mut self, amount: u64) -> VaultCoreResult<u64> {
        // the number of tokens to mint is the pro-rata amount of the total tokens deposited and the LRT supply
        let num_tokens_to_mint = self.calculate_lrt_mint_amount(amount)?;

        // deposit tokens + check against capacity
        let total_post_deposit = self
            .tokens_deposited
            .checked_add(amount)
            .ok_or(VaultCoreError::VaultDepositOverflow)?;
        if total_post_deposit > self.capacity {
            return Err(VaultCoreError::VaultDepositExceedsCapacity);
        }

        let lrt_supply = self
            .lrt_supply
            .checked_add(num_tokens_to_mint)
            .ok_or(VaultCoreError::VaultDepositOverflow)?;

        self.lrt_supply = lrt_supply;
        self.tokens_deposited = total_post_deposit;

        Ok(num_tokens_to_mint)
    }

    pub fn set_lrt_supply(&mut self, lrt_supply: u64) {
        self.lrt_supply = lrt_supply;
    }

    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn set_capacity(&mut self, capacity: u64) {
        self.capacity = capacity;
    }

    // ------------------------------------------
    // Entity Management
    // ------------------------------------------

    pub const fn ncn_count(&self) -> u64 {
        self.ncn_count
    }

    pub fn increment_ncn_count(&mut self) -> VaultCoreResult<()> {
        self.ncn_count = self
            .ncn_count
            .checked_add(1)
            .ok_or(VaultCoreError::VaultNcnOverflow)?;
        Ok(())
    }

    pub const fn operator_count(&self) -> u64 {
        self.operator_count
    }

    pub fn increment_operator_count(&mut self) -> VaultCoreResult<()> {
        self.operator_count = self
            .operator_count
            .checked_add(1)
            .ok_or(VaultCoreError::VaultOperatorOverflow)?;
        Ok(())
    }

    pub const fn slasher_count(&self) -> u64 {
        self.slasher_count
    }

    pub fn increment_slasher_count(&mut self) -> VaultCoreResult<()> {
        self.slasher_count = self
            .slasher_count
            .checked_add(1)
            .ok_or(VaultCoreError::VaultSlasherOverflow)?;
        Ok(())
    }

    pub const fn lrt_mint(&self) -> Pubkey {
        self.lrt_mint
    }

    pub const fn supported_mint(&self) -> Pubkey {
        self.supported_mint
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    // ------------------------------------------
    // Administration
    // ------------------------------------------

    pub const fn fee_wallet(&self) -> Pubkey {
        self.fee_wallet
    }

    pub fn set_fee_wallet(&mut self, fee_wallet: Pubkey) {
        self.fee_wallet = fee_wallet;
    }

    pub fn mint_burn_authority(&self) -> Option<Pubkey> {
        if self.mint_burn_authority != Pubkey::default() {
            Some(self.mint_burn_authority)
        } else {
            None
        }
    }

    pub fn set_mint_burn_authority(&mut self, mint_burn_authority: Pubkey) {
        self.mint_burn_authority = mint_burn_authority;
    }

    pub const fn vault_index(&self) -> u64 {
        self.vault_index
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub fn set_admin(&mut self, admin: Pubkey) {
        self.admin = admin;
    }

    pub fn check_admin(&self, admin: &Pubkey) -> VaultCoreResult<()> {
        if self.admin != *admin {
            return Err(VaultCoreError::VaultInvalidAdmin);
        }
        Ok(())
    }

    pub fn set_delegation_admin(&mut self, delegation_admin: Pubkey) {
        self.delegation_admin = delegation_admin;
    }

    pub const fn delegation_admin(&self) -> Pubkey {
        self.delegation_admin
    }

    pub fn check_delegation_admin(&self, delegation_admin: &Pubkey) -> VaultCoreResult<()> {
        if self.delegation_admin != *delegation_admin {
            return Err(VaultCoreError::VaultInvalidDelegationAdmin);
        }
        Ok(())
    }

    pub fn set_ncn_admin(&mut self, ncn_admin: Pubkey) {
        self.ncn_admin = ncn_admin;
    }

    pub const fn ncn_admin(&self) -> Pubkey {
        self.ncn_admin
    }

    pub fn check_ncn_admin(&self, ncn_admin: &Pubkey) -> VaultCoreResult<()> {
        if self.ncn_admin != *ncn_admin {
            return Err(VaultCoreError::VaultInvalidNcnAdmin);
        }
        Ok(())
    }

    pub fn set_operator_admin(&mut self, operator_admin: Pubkey) {
        self.operator_admin = operator_admin;
    }

    pub const fn operator_admin(&self) -> Pubkey {
        self.operator_admin
    }

    pub fn check_operator_admin(&self, operator_admin: &Pubkey) -> VaultCoreResult<()> {
        if self.operator_admin != *operator_admin {
            return Err(VaultCoreError::VaultInvalidOperatorAdmin);
        }
        Ok(())
    }

    pub fn set_slasher_admin(&mut self, slasher_admin: Pubkey) {
        self.slasher_admin = slasher_admin;
    }

    pub const fn slasher_admin(&self) -> Pubkey {
        self.slasher_admin
    }

    pub fn check_slasher_admin(&self, slasher_admin: &Pubkey) -> VaultCoreResult<()> {
        if self.slasher_admin != *slasher_admin {
            return Err(VaultCoreError::VaultInvalidSlasherAdmin);
        }
        Ok(())
    }

    pub const fn lrt_supply(&self) -> u64 {
        self.lrt_supply
    }

    // ------------------------------------------
    // Serialization & Deserialization
    // ------------------------------------------

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn is_struct_valid(&self) -> bool {
        self.account_type == AccountType::Vault
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"vault".as_ref().to_vec(), base.to_bytes().to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            msg!("Vault account data is empty");
            return Err(VaultCoreError::VaultDataEmpty);
        }
        if account.owner != program_id {
            msg!("Vault account owner is not the program id");
            return Err(VaultCoreError::VaultInvalidProgramOwner);
        }

        let state = Self::deserialize(&mut account.data.borrow_mut().as_ref()).map_err(|e| {
            msg!("Vault account deserialization failed: {}", e);
            VaultCoreError::VaultInvalidData
        })?;
        if !state.is_struct_valid() {
            msg!("Vault account header is invalid");
            return Err(VaultCoreError::VaultInvalidData);
        }

        let mut seeds = Self::seeds(&state.base);
        seeds.push(vec![state.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey =
            Pubkey::create_program_address(&seeds_iter, program_id).map_err(|e| {
                msg!("Vault account PDA creation failed: {}", e);
                VaultCoreError::VaultInvalidPda
            })?;
        if expected_pubkey != *account.key {
            msg!("Vault account PDA is invalid");
            return Err(VaultCoreError::VaultInvalidPda);
        }

        Ok(state)
    }
}

pub struct SanitizedVault<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault: Box<Vault>,
}

impl<'a, 'info> SanitizedVault<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> VaultCoreResult<SanitizedVault<'a, 'info>> {
        if expect_writable && !account.is_writable {
            msg!("Vault account is not writable");
            return Err(VaultCoreError::VaultExpectedWritable);
        }
        let vault = Box::new(Vault::deserialize_checked(program_id, account)?);

        Ok(SanitizedVault { account, vault })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault(&self) -> &Vault {
        &self.vault
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        &mut self.vault
    }

    pub fn save(&self) -> VaultCoreResult<()> {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.vault)
            .map_err(|e| VaultCoreError::VaultSerializationFailed(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use crate::vault::{Vault, VaultCoreError};

    #[test]
    fn test_deposit_ratio_simple_ok() {
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
        let num_minted = vault.deposit_and_mint_with_capacity_check(100).unwrap();
        assert_eq!(num_minted, 100);

        assert_eq!(vault.tokens_deposited(), 100);
        assert_eq!(vault.lrt_supply(), 100);
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
        vault.set_tokens_deposited(90);
        vault.set_lrt_supply(100);

        let num_minted = vault.deposit_and_mint_with_capacity_check(100).unwrap();
        assert_eq!(num_minted, 111);

        assert_eq!(vault.tokens_deposited(), 190);
        assert_eq!(vault.lrt_supply(), 211);
    }

    #[test]
    fn test_deposit_capacity_exceeded_fails() {
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
        vault.set_capacity(100);

        assert_eq!(
            vault.deposit_and_mint_with_capacity_check(101),
            Err(VaultCoreError::VaultDepositExceedsCapacity)
        );
    }

    #[test]
    fn test_deposit_capacity_ok() {
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
        vault.set_capacity(100);

        vault.deposit_and_mint_with_capacity_check(50).unwrap();
        vault.deposit_and_mint_with_capacity_check(50).unwrap();
        assert_eq!(
            vault.deposit_and_mint_with_capacity_check(1),
            Err(VaultCoreError::VaultDepositExceedsCapacity)
        );
    }

    #[test]
    fn test_calculate_assets_returned_amount_ok() {
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

        vault.set_lrt_supply(100_000);
        vault.set_tokens_deposited(100_000);
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            50_000
        );

        vault.set_tokens_deposited(90_000);
        vault.set_lrt_supply(100_000);
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            45_000
        );

        vault.set_tokens_deposited(110_000);
        vault.set_lrt_supply(100_000);
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            55_000
        );

        vault.set_tokens_deposited(100);
        vault.set_lrt_supply(0);
        assert_eq!(
            vault.calculate_assets_returned_amount(100),
            Err(VaultCoreError::VaultEmpty)
        );

        vault.set_tokens_deposited(100);
        vault.set_lrt_supply(1);
        assert_eq!(
            vault.calculate_assets_returned_amount(100),
            Err(VaultCoreError::VaultInsufficientFunds)
        );

        vault.set_tokens_deposited(100);
        vault.set_lrt_supply(13);
        assert_eq!(vault.calculate_assets_returned_amount(1).unwrap(), 7);
    }
}
