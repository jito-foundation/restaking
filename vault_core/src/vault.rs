use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_vault_sdk::error::VaultError;
use solana_program::pubkey::Pubkey;

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
            bump,
            reserved: [0; 3],
        }
    }

    // ------------------------------------------
    // Asset accounting and tracking
    // ------------------------------------------

    pub fn max_delegation_amount(&self) -> Result<u64, VaultError> {
        self.tokens_deposited
            .checked_sub(self.withdrawable_reserve_amount)
            .ok_or(VaultError::VaultOverflow)
    }

    pub fn calculate_assets_returned_amount(&self, lrt_amount: u64) -> Result<u64, VaultError> {
        if self.lrt_supply == 0 {
            return Err(VaultError::VaultLrtEmpty);
        } else if lrt_amount > self.lrt_supply {
            return Err(VaultError::VaultInsufficientFunds);
        }

        lrt_amount
            .checked_mul(self.tokens_deposited)
            .and_then(|x| x.checked_div(self.lrt_supply))
            .ok_or(VaultError::VaultOverflow)
    }

    pub fn calculate_lrt_mint_amount(&self, amount: u64) -> Result<u64, VaultError> {
        if self.tokens_deposited == 0 {
            return Ok(amount);
        }

        amount
            .checked_mul(self.lrt_supply)
            .and_then(|x| x.checked_div(self.tokens_deposited))
            .ok_or(VaultError::VaultOverflow)
    }

    pub const fn deposit_fee_bps(&self) -> u16 {
        self.deposit_fee_bps
    }

    pub fn calculate_deposit_fee(&self, lrt_amount: u64) -> Result<u64, VaultError> {
        let fee = lrt_amount
            .checked_mul(self.deposit_fee_bps() as u64)
            .and_then(|x| x.checked_div(10_000))
            .ok_or(VaultError::VaultOverflow)?;
        Ok(fee)
    }

    pub const fn withdrawal_fee_bps(&self) -> u16 {
        self.withdrawal_fee_bps
    }

    pub fn calculate_withdraw_fee(&self, lrt_amount: u64) -> Result<u64, VaultError> {
        let fee = lrt_amount
            .checked_mul(self.withdrawal_fee_bps() as u64)
            .and_then(|x| x.checked_div(10_000))
            .ok_or(VaultError::VaultOverflow)?;
        Ok(fee)
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
        let num_minted = vault.calculate_lrt_mint_amount(100).unwrap();
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
        vault.lrt_supply = 100;

        let num_minted = vault.calculate_lrt_mint_amount(100).unwrap();
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

        vault.lrt_supply = 100_000;
        vault.tokens_deposited = 100_000;
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            50_000
        );

        vault.tokens_deposited = 90_000;
        vault.lrt_supply = 100_000;
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            45_000
        );

        vault.tokens_deposited = 110_000;
        vault.lrt_supply = 100_000;
        assert_eq!(
            vault.calculate_assets_returned_amount(50_000).unwrap(),
            55_000
        );

        vault.tokens_deposited = 100;
        vault.lrt_supply = 0;
        assert_eq!(
            vault.calculate_assets_returned_amount(100),
            Err(VaultError::VaultLrtEmpty)
        );

        vault.tokens_deposited = 100;
        vault.lrt_supply = 1;
        assert_eq!(
            vault.calculate_assets_returned_amount(100),
            Err(VaultError::VaultInsufficientFunds)
        );

        vault.tokens_deposited = 100;
        vault.lrt_supply = 13;
        assert_eq!(vault.calculate_assets_returned_amount(1).unwrap(), 7);
    }
}
