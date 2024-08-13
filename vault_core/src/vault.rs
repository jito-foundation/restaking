//! The vault is responsible for holding tokens and minting VRT tokens.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_vault_sdk::error::VaultError;
use solana_program::pubkey::Pubkey;

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

    /// Mint of the VRT token
    pub vrt_mint: Pubkey,

    /// Mint of the token that is supported by the VRT
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

    /// The admin responsible for withdrawing tokens
    pub withdraw_admin: Pubkey,

    /// Fee wallet account
    pub fee_wallet: Pubkey,

    /// Optional mint signer
    pub mint_burn_admin: Pubkey,

    /// Max capacity of tokens in the vault
    pub capacity: u64,

    /// The index of the vault in the vault list
    pub vault_index: u64,

    /// The total number of VRT in circulation
    pub vrt_supply: u64,

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
            withdraw_admin: admin,
            fee_wallet: admin,
            mint_burn_admin: Pubkey::default(),
            capacity: u64::MAX,
            vault_index,
            vrt_supply: 0,
            tokens_deposited: 0,
            withdrawable_reserve_amount: 0,
            deposit_fee_bps,
            withdrawal_fee_bps,
            ncn_count: 0,
            operator_count: 0,
            slasher_count: 0,
            bump,
            reserved: [0; 3],
        }
    }

    // ------------------------------------------
    // Asset accounting and tracking
    // ------------------------------------------

    /// Calculate the maximum amount of tokens that can be delegated to operators, which
    /// is the total amount of tokens deposited in the vault minus the amount of tokens
    /// that are reserved for withdrawal.
    pub fn max_delegation_amount(&self) -> Result<u64, VaultError> {
        self.tokens_deposited
            .checked_sub(self.withdrawable_reserve_amount)
            .ok_or(VaultError::VaultOverflow)
    }

    /// Calculate the maximum amount of tokens that can be withdrawn from the vault given the VRT
    /// amount. This is the pro-rata share of the total tokens deposited in the vault.
    pub fn calculate_assets_returned_amount(&self, vrt_amount: u64) -> Result<u64, VaultError> {
        if self.vrt_supply == 0 {
            return Err(VaultError::VaultLrtEmpty);
        } else if vrt_amount > self.vrt_supply {
            return Err(VaultError::VaultInsufficientFunds);
        }

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
            .and_then(|x| x.checked_div(10_000))
            .ok_or(VaultError::VaultOverflow)?;
        Ok(fee)
    }

    /// Calculate the amount of tokens collected as a fee for withdrawing tokens from the vault.
    pub fn calculate_withdraw_fee(&self, vrt_amount: u64) -> Result<u64, VaultError> {
        let fee = vrt_amount
            .checked_mul(self.withdrawal_fee_bps as u64)
            .and_then(|x| x.checked_div(10_000))
            .ok_or(VaultError::VaultOverflow)?;
        Ok(fee)
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
}

#[cfg(test)]
mod tests {
    use jito_vault_sdk::error::VaultError;
    use solana_program::pubkey::Pubkey;

    use crate::vault::Vault;

    #[test]
    fn test_deposit_ratio_simple_ok() {
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
        let num_minted = vault.calculate_vrt_mint_amount(100).unwrap();
        assert_eq!(num_minted, 100);
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

        let num_minted = vault.calculate_vrt_mint_amount(100).unwrap();
        assert_eq!(num_minted, 111);
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

        vault.vrt_supply = 100_000;
        vault.tokens_deposited = 100_000;
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            50_000
        );

        vault.tokens_deposited = 90_000;
        vault.vrt_supply = 100_000;
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            45_000
        );

        vault.tokens_deposited = 110_000;
        vault.vrt_supply = 100_000;
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            55_000
        );

        vault.tokens_deposited = 100;
        vault.vrt_supply = 0;
        assert_eq!(
            vault.calculate_assets_returned_amount(100),
            Err(VaultError::VaultLrtEmpty)
        );

        vault.tokens_deposited = 100;
        vault.vrt_supply = 1;
        assert_eq!(
            vault.calculate_assets_returned_amount(100),
            Err(VaultError::VaultInsufficientFunds)
        );

        vault.tokens_deposited = 100;
        vault.vrt_supply = 13;
        assert_eq!(vault.calculate_assets_returned_amount(1).unwrap(), 7);
    }
}
