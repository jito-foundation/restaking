//! The NCN (Node Consensus Network) account is a program-owned account that
//! represents a network of nodes that participate in consensus. The NCN
//! account is used to manage the operators, vaults, and slashers that are
//! associated with the network.

use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use jito_restaking_sdk::error::RestakingError;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

/// The NCN manages the operators, vaults, and slashers associated with a network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Ncn {
    /// The base account used as a PDA seed
    pub base: Pubkey,

    /// The admin of the NCN
    pub admin: Pubkey,

    /// The operator admin of the NCN
    pub operator_admin: Pubkey,

    /// The vault admin of the NCN
    pub vault_admin: Pubkey,

    /// The slasher admin of the NCN
    pub slasher_admin: Pubkey,

    /// The withdraw admin of the NCN
    pub withdraw_admin: Pubkey,

    /// The withdraw fee wallet of the NCN
    pub withdraw_fee_wallet: Pubkey,

    /// The index of the NCN
    index: PodU64,

    /// Number of operator accounts associated with the NCN
    operator_count: PodU64,

    /// Number of vault accounts associated with the NCN
    vault_count: PodU64,

    /// Number of slasher accounts associated with the NCN
    slasher_count: PodU64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl Discriminator for Ncn {
    const DISCRIMINATOR: u8 = 2;
}

impl Ncn {
    #[allow(clippy::too_many_arguments)]
    pub fn new(base: Pubkey, admin: Pubkey, ncn_index: u64, bump: u8) -> Self {
        Self {
            base,
            admin,
            operator_admin: admin,
            vault_admin: admin,
            slasher_admin: admin,
            withdraw_admin: admin,
            withdraw_fee_wallet: admin,
            index: PodU64::from(ncn_index),
            operator_count: PodU64::from(0),
            vault_count: PodU64::from(0),
            slasher_count: PodU64::from(0),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn index(&self) -> u64 {
        self.index.into()
    }

    pub fn operator_count(&self) -> u64 {
        self.operator_count.into()
    }

    pub fn vault_count(&self) -> u64 {
        self.vault_count.into()
    }

    pub fn slasher_count(&self) -> u64 {
        self.slasher_count.into()
    }

    pub fn increment_operator_count(&mut self) -> Result<(), RestakingError> {
        let mut operator_count: u64 = self.operator_count.into();
        operator_count = operator_count
            .checked_add(1)
            .ok_or(RestakingError::OperatorOverflow)?;
        self.operator_count = PodU64::from(operator_count);
        Ok(())
    }

    pub fn increment_vault_count(&mut self) -> Result<(), RestakingError> {
        let mut vault_count: u64 = self.vault_count.into();
        vault_count = vault_count
            .checked_add(1)
            .ok_or(RestakingError::VaultOverflow)?;
        self.vault_count = PodU64::from(vault_count);
        Ok(())
    }

    pub fn increment_slasher_count(&mut self) -> Result<(), RestakingError> {
        let mut slasher_count: u64 = self.slasher_count.into();
        slasher_count = slasher_count
            .checked_add(1)
            .ok_or(RestakingError::SlasherOverflow)?;
        self.slasher_count = PodU64::from(slasher_count);
        Ok(())
    }

    /// Replace all secondary admins that were equal to the old admin to the new admin
    ///
    /// # Arguments
    /// * `old_admin` - The old admin Pubkey
    /// * `new_admin` - The new admin Pubkey
    pub fn update_secondary_admin(&mut self, old_admin: &Pubkey, new_admin: &Pubkey) {
        if self.operator_admin.eq(old_admin) {
            self.operator_admin = *new_admin;
            msg!("Operator admin set to {:?}", new_admin);
        }

        if self.vault_admin.eq(old_admin) {
            self.vault_admin = *new_admin;
            msg!("Vault admin set to {:?}", new_admin);
        }

        if self.slasher_admin.eq(old_admin) {
            self.slasher_admin = *new_admin;
            msg!("Slasher admin set to {:?}", new_admin);
        }

        if self.withdraw_admin.eq(old_admin) {
            self.withdraw_admin = *new_admin;
            msg!("Withdraw admin set to {:?}", new_admin);
        }

        if self.withdraw_fee_wallet.eq(old_admin) {
            self.withdraw_fee_wallet = *new_admin;
            msg!("Withdraw fee wallet set to {:?}", new_admin);
        }
    }

    /// Returns the seeds for the PDA
    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"ncn".to_vec(), base.as_ref().to_vec()])
    }

    /// Find the program address for the NCN account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Attempts to load the account as [`Ncn`], returning an error if it's not valid.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load the NCN from
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
            msg!("NCN account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("NCN account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("NCN account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("NCN account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let base = Self::try_from_slice_unchecked(&account.data.borrow())?.base;
        if account
            .key
            .ne(&Self::find_program_address(program_id, &base).0)
        {
            msg!("NCN account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use super::Ncn;

    #[test]
    fn test_update_secondary_admin_ok() {
        let old_admin = Pubkey::new_unique();
        let mut ncn = Ncn::new(Pubkey::new_unique(), old_admin, 0, 0);

        assert_eq!(ncn.operator_admin, old_admin);
        assert_eq!(ncn.vault_admin, old_admin);
        assert_eq!(ncn.slasher_admin, old_admin);
        assert_eq!(ncn.withdraw_admin, old_admin);
        assert_eq!(ncn.withdraw_fee_wallet, old_admin);

        let new_admin = Pubkey::new_unique();
        ncn.update_secondary_admin(&old_admin, &new_admin);

        assert_eq!(ncn.operator_admin, new_admin);
        assert_eq!(ncn.vault_admin, new_admin);
        assert_eq!(ncn.slasher_admin, new_admin);
        assert_eq!(ncn.withdraw_admin, new_admin);
        assert_eq!(ncn.withdraw_fee_wallet, new_admin);
    }
}
