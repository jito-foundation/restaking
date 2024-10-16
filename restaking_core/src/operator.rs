//! The Operator account stores global information for a particular operator
//! including the admin, voter, and the number of NCN and vault accounts.

use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{
    types::{PodU16, PodU64},
    AccountDeserialize, Discriminator,
};
use jito_restaking_sdk::error::RestakingError;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for Operator {
    const DISCRIMINATOR: u8 = 3;
}

/// The Operator account stores global information for a particular operator
/// including the admin, voter, and the number of NCN and vault accounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Operator {
    /// The base pubkey used as a seed for the PDA
    pub base: Pubkey,

    /// The admin pubkey
    pub admin: Pubkey,

    /// The NCN admin can add and remove support for NCNs in the restaking protocol
    pub ncn_admin: Pubkey,

    /// The vault admin can add and remove support for vaults in the restaking protocol
    pub vault_admin: Pubkey,

    /// The delegate admin can delegate assets from the operator
    pub delegate_admin: Pubkey,

    /// ( For future use ) Authority to update the operators's metadata
    pub metadata_admin: Pubkey,

    /// The voter pubkey can be used as the voter for signing transactions for interacting
    /// with various NCN programs. NCNs can also opt for their own signing infrastructure.
    pub voter: Pubkey,

    /// The operator index
    index: PodU64,

    /// The number of NcnOperatorTickets associated with the operator.
    /// Helpful for indexing all available OperatorNcnTickets.
    ncn_count: PodU64,

    /// The number of NcnVaultTickets associated with the operator.
    /// Helpful for indexing all available OperatorVaultTickets.
    vault_count: PodU64,

    /// The operator fee in basis points
    pub operator_fee_bps: PodU16,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved_space: [u8; 261],
}

impl Operator {
    /// Create a new Operator account
    /// # Arguments
    /// * `base` - The base account used as a PDA seed
    /// * `admin` - The admin of the Operator
    /// * `index` - The index of the Operator
    /// * `bump` - The bump seed for the PDA
    pub fn new(base: Pubkey, admin: Pubkey, index: u64, operator_fee_bps: u16, bump: u8) -> Self {
        Self {
            base,
            admin,
            ncn_admin: admin,
            vault_admin: admin,
            delegate_admin: admin,
            metadata_admin: admin,
            voter: admin,
            index: PodU64::from(index),
            ncn_count: PodU64::from(0),
            vault_count: PodU64::from(0),
            operator_fee_bps: PodU16::from(operator_fee_bps),
            bump,
            reserved_space: [0; 261],
        }
    }

    pub fn index(&self) -> u64 {
        self.index.into()
    }

    pub fn ncn_count(&self) -> u64 {
        self.ncn_count.into()
    }

    pub fn vault_count(&self) -> u64 {
        self.vault_count.into()
    }

    pub fn increment_ncn_count(&mut self) -> Result<(), RestakingError> {
        let mut ncn_count: u64 = self.ncn_count.into();
        ncn_count = ncn_count
            .checked_add(1)
            .ok_or(RestakingError::NcnOverflow)?;
        self.ncn_count = PodU64::from(ncn_count);
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

    /// Validates the admin account and ensures it matches the expected admin.
    ///
    /// # Arguments
    /// * `admin` - A reference to the [`Pubkey`] representing the admin Pubkey that is attempting
    ///   to authorize the operation.
    ///
    /// # Returns
    /// * `Result<(), RestakingError>` - Returns `Ok(())` if the admin account is valid.
    ///
    /// # Errors
    /// This function will return a [`jito_restaking_sdk::error::RestakingError::OperatorAdminInvalid`] error in the following case:
    /// * The `admin_info` account's public key does not match the expected admin public key stored in `self`.
    pub fn check_admin(&self, admin: &Pubkey) -> Result<(), RestakingError> {
        if self.admin.ne(admin) {
            msg!(
                "Incorrect admin provided, expected {}, received {}",
                self.admin,
                admin
            );
            return Err(RestakingError::OperatorAdminInvalid);
        }
        Ok(())
    }

    /// Validates the delegate_admin account and ensures it matches the expected delegate_admin.
    ///
    /// # Arguments
    /// * `delegate_admin_info` - A reference to the [`Pubkey`] representing the delegate_admin Pubkey that is attempting
    ///   to authorize the operation.
    ///
    /// # Returns
    /// * `Result<(), RestakingError>` - Returns `Ok(())` if the delegate_admin account is valid.
    ///
    /// # Errors
    /// This function will return a [`jito_restaking_sdk::error::RestakingError::OperatorDelegateAdminInvalid`] error in the following case:
    /// * The `delegate_admin_info` account's public key does not match the expected admin public key stored in `self`.
    pub fn check_delegate_admin(&self, delegate_admin: &Pubkey) -> Result<(), RestakingError> {
        if self.delegate_admin.ne(delegate_admin) {
            msg!(
                "Incorrect delegate_admin provided, expected {}, received {}",
                self.delegate_admin,
                delegate_admin
            );
            return Err(RestakingError::OperatorDelegateAdminInvalid);
        }
        Ok(())
    }

    /// Replace all secondary admins that were equal to the old admin to the new admin
    ///
    /// # Arguments
    /// * `old_admin` - The old admin Pubkey
    /// * `new_admin` - The new admin Pubkey
    pub fn update_secondary_admin(&mut self, old_admin: &Pubkey, new_admin: &Pubkey) {
        if self.ncn_admin.eq(old_admin) {
            self.ncn_admin = *new_admin;
            msg!("NCN admin set to {:?}", new_admin);
        }

        if self.vault_admin.eq(old_admin) {
            self.vault_admin = *new_admin;
            msg!("Vault admin set to {:?}", new_admin);
        }

        if self.voter.eq(old_admin) {
            self.voter = *new_admin;
            msg!("Voter set to {:?}", new_admin);
        }

        if self.delegate_admin.eq(old_admin) {
            self.delegate_admin = *new_admin;
            msg!("Delegate admin set to {:?}", new_admin);
        }

        if self.metadata_admin.eq(old_admin) {
            self.metadata_admin = *new_admin;
            msg!("Metadata admin set to {:?}", new_admin);
        }
    }

    /// Returns the seeds for the PDA
    ///
    /// # Arguments
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"operator".to_vec(), base.as_ref().to_vec()])
    }

    /// Find the program address for the Operator account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * `Pubkey` - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Attempts to load the account as [`Operator`], returning an error if it's not valid.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load the operator from
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
            msg!("Operator account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("Operator account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("Operator account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Operator account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let base = Self::try_from_slice_unchecked(&account.data.borrow())?.base;
        if account
            .key
            .ne(&Self::find_program_address(program_id, &base).0)
        {
            msg!("Operator account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use jito_bytemuck::types::PodU64;
    use solana_program::pubkey::Pubkey;

    use crate::operator::Operator;

    #[test]
    fn test_operator_no_padding() {
        let operator_size = std::mem::size_of::<Operator>();
        let sum_of_fields = std::mem::size_of::<Pubkey>() + // base
            std::mem::size_of::<Pubkey>() + // admin
            std::mem::size_of::<Pubkey>() + // ncn_admin
            std::mem::size_of::<Pubkey>() + // vault_admin
            std::mem::size_of::<Pubkey>() + // delegate_admin
            std::mem::size_of::<Pubkey>() + // metadata_admin
            std::mem::size_of::<Pubkey>() + // voter
            std::mem::size_of::<PodU64>() + // index
            std::mem::size_of::<PodU64>() + // ncn_count
            std::mem::size_of::<PodU64>() + // vault_count
            std::mem::size_of::<u8>() + // bump
            263; // reserved_space
        assert_eq!(operator_size, sum_of_fields);
    }

    #[test]
    fn test_update_secondary_admin_ok() {
        let old_admin = Pubkey::new_unique();
        let mut operator = Operator::new(Pubkey::new_unique(), old_admin, 0, 0, 0);

        assert_eq!(operator.ncn_admin, old_admin);
        assert_eq!(operator.vault_admin, old_admin);
        assert_eq!(operator.voter, old_admin);
        assert_eq!(operator.delegate_admin, old_admin);

        let new_admin = Pubkey::new_unique();
        operator.update_secondary_admin(&old_admin, &new_admin);

        assert_eq!(operator.ncn_admin, new_admin);
        assert_eq!(operator.vault_admin, new_admin);
        assert_eq!(operator.voter, new_admin);
        assert_eq!(operator.delegate_admin, new_admin);
        assert_eq!(operator.metadata_admin, new_admin);
    }
}
