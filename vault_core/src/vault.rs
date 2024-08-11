use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::pubkey::Pubkey;

use crate::result::{VaultCoreError, VaultCoreResult};

impl Discriminator for Vault {
    const DISCRIMINATOR: u8 = 2;
}

/// The vault is repsonsible for holding tokens and minting LRT tokens
/// based on the amount of tokens deposited.
/// It also contains several administrative functions for features inside the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Vault {
    /// The base account of the LRT
    pub base: Pubkey,

    /// Mint of the LRT token
    pub lrt_mint: Pubkey,

    /// Mint of the token that is supported by the LRT
    pub supported_mint: Pubkey,

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

    pub withdraw_admin: Pubkey,

    /// Fee wallet account
    pub fee_wallet: Pubkey,

    /// Optional mint signer
    pub mint_burn_admin: Pubkey,

    /// Max capacity of tokens in the vault
    pub capacity: u64,

    /// The index of the vault in the vault list
    pub vault_index: u64,

    /// The total number of LRT in circulation
    pub lrt_supply: u64,

    /// The total number of tokens deposited
    pub tokens_deposited: u64,

    /// The amount of tokens that are reserved for withdrawal
    pub withdrawable_reserve_amount: u64,

    /// Number of VaultNcnTicket accounts tracked by this vault
    pub ncn_count: u64,

    /// Number of VaultOperatorTicket accounts tracked by this vault
    pub operator_count: u64,

    /// Number of VaultNcnSlasherTicket accounts tracked by this vault
    pub slasher_count: u64,

    /// The maximum amount of tokens that can be withdrawn per epoch
    pub epoch_withdraw_cap: u64,

    /// The current epoch number
    pub current_epoch: u64,

    /// The amount of tokens withdrawn in the current epoch
    pub epoch_withdrawn_amount: u64,

    /// The deposit fee in basis points
    pub deposit_fee_bps: u16,

    /// The withdrawal fee in basis points
    pub withdrawal_fee_bps: u16,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 3],
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
        current_epoch: u64,
        epoch_withdraw_cap: u64,
    ) -> Self {
        Self {
            base,
            lrt_mint,
            supported_mint,
            admin,
            delegation_admin: admin,
            operator_admin: admin,
            ncn_admin: admin,
            slasher_admin: admin,
            capacity_admin: admin,
            withdraw_admin: admin,
            fee_wallet: admin,
            mint_burn_admin: Pubkey::default(),
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
            epoch_withdraw_cap,
            current_epoch,
            epoch_withdrawn_amount: 0,
            bump,
            reserved: [0; 3],
        }
    }

    // ------------------------------------------
    // Asset accounting and tracking
    // ------------------------------------------

    pub fn max_delegation_amount(&self) -> VaultCoreResult<u64> {
        self.tokens_deposited
            .checked_sub(self.withdrawable_reserve_amount)
            .ok_or(VaultCoreError::VaultDelegationUnderflow)
    }

    pub fn calculate_assets_returned_amount(
        &mut self,
        lrt_amount: u64,
        current_epoch: u64,
    ) -> VaultCoreResult<u64> {
        if self.lrt_supply == 0 {
            return Err(VaultCoreError::VaultEmpty);
        } else if lrt_amount > self.lrt_supply {
            return Err(VaultCoreError::VaultInsufficientFunds);
        }

        let returned_amount = lrt_amount
            .checked_mul(self.tokens_deposited)
            .ok_or(VaultCoreError::VaultWithdrawOverflow)?
            .checked_div(self.lrt_supply)
            .ok_or(VaultCoreError::VaultWithdrawOverflow)?;

        if current_epoch == self.current_epoch {
            // Check if the returned amount exceeds the remaining withdrawal cap for the epoch
            if self.epoch_withdrawn_amount + returned_amount > self.epoch_withdraw_cap {
                return Err(VaultCoreError::VaultWithdrawOverflow);
            }
        } else {
            self.current_epoch = current_epoch;
            self.epoch_withdrawn_amount = 0;
        }

        // Update the epoch withdrawn amount
        self.epoch_withdrawn_amount += returned_amount;

        Ok(returned_amount)
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

    // ------------------------------------------
    // Serialization & Deserialization
    // ------------------------------------------

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"vault".as_ref().to_vec(), base.to_bytes().to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use crate::vault::{Vault, VaultCoreError};

    #[test]
    fn test_deposit_ratio_simple_ok() {
        let current_epoch = 100;

        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            current_epoch,
            100,
        );
        let num_minted = vault.deposit_and_mint_with_capacity_check(100).unwrap();
        assert_eq!(num_minted, 100);

        assert_eq!(vault.tokens_deposited, 100);
        assert_eq!(vault.lrt_supply, 100);
    }

    #[test]
    fn test_deposit_ratio_after_slashed_ok() {
        let current_epoch = 100;

        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            current_epoch,
            100,
        );
        vault.tokens_deposited = 90;
        vault.lrt_supply = 100;

        let num_minted = vault.deposit_and_mint_with_capacity_check(100).unwrap();
        assert_eq!(num_minted, 111);

        assert_eq!(vault.tokens_deposited, 190);
        assert_eq!(vault.lrt_supply, 211);
    }

    #[test]
    fn test_deposit_capacity_exceeded_fails() {
        let current_epoch = 100;

        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            current_epoch,
            100,
        );
        vault.capacity = 100;

        assert_eq!(
            vault.deposit_and_mint_with_capacity_check(101),
            Err(VaultCoreError::VaultDepositExceedsCapacity)
        );
    }

    #[test]
    fn test_deposit_capacity_ok() {
        let current_epoch = 100;

        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            current_epoch,
            100,
        );
        vault.capacity = 100;

        vault.deposit_and_mint_with_capacity_check(50).unwrap();
        vault.deposit_and_mint_with_capacity_check(50).unwrap();
        assert_eq!(
            vault.deposit_and_mint_with_capacity_check(1),
            Err(VaultCoreError::VaultDepositExceedsCapacity)
        );
    }

    #[test]
    fn test_calculate_assets_returned_amount_ok() {
        let mut current_epoch = 100;

        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
            current_epoch,
            100_000,
        );

        current_epoch = 101;
        vault.lrt_supply = 100_000;
        vault.tokens_deposited = 100_000;
        assert_eq!(
            vault
                .calculate_assets_returned_amount(50_000, current_epoch)
                .unwrap(),
            50_000
        );

        current_epoch = 102;
        vault.tokens_deposited = 90_000;
        vault.lrt_supply = 100_000;
        vault.epoch_withdrawn_amount = 100_000;
        assert_eq!(
            vault
                .calculate_assets_returned_amount(50_000, current_epoch)
                .unwrap(),
            45_000
        );

        current_epoch = 103;
        vault.tokens_deposited = 110_000;
        vault.lrt_supply = 100_000;
        vault.epoch_withdrawn_amount = 100_000;
        assert_eq!(
            vault
                .calculate_assets_returned_amount(50_000, current_epoch)
                .unwrap(),
            55_000
        );

        current_epoch = 104;
        vault.tokens_deposited = 100;
        vault.lrt_supply = 0;
        assert_eq!(
            vault.calculate_assets_returned_amount(100, current_epoch),
            Err(VaultCoreError::VaultEmpty)
        );

        current_epoch = 105;
        vault.tokens_deposited = 100;
        vault.lrt_supply = 1;
        assert_eq!(
            vault.calculate_assets_returned_amount(100, current_epoch),
            Err(VaultCoreError::VaultInsufficientFunds)
        );

        current_epoch = 106;
        vault.tokens_deposited = 100;
        vault.lrt_supply = 13;
        assert_eq!(
            vault
                .calculate_assets_returned_amount(1, current_epoch)
                .unwrap(),
            7
        );

        current_epoch = 107;
        vault.current_epoch = 107;
        vault.tokens_deposited = 1_000_000;
        vault.lrt_supply = 1_000_000;
        vault.epoch_withdrawn_amount = 1;
        assert_eq!(
            vault.calculate_assets_returned_amount(100_000, current_epoch),
            Err(VaultCoreError::VaultWithdrawOverflow)
        );

        current_epoch = 108;
        vault.current_epoch = 107;
        vault.tokens_deposited = 1_000_000;
        vault.lrt_supply = 1_000_000;
        vault.epoch_withdrawn_amount = 0;
        assert_eq!(
            vault.calculate_assets_returned_amount(100_000, current_epoch),
            Ok(100_000)
        );
    }
}
