//! The Operator account stores global information for a particular operator
//! including the admin, voter, and the number of NCN and vault accounts.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_restaking_sdk::error::RestakingError;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for Operator {
    const DISCRIMINATOR: u8 = 3;
}

/// The Operator account stores global information for a particular operator
/// including the admin, voter, and the number of NCN and vault accounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
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

    /// The withdrawal admin can withdraw assets from the operator
    pub withdrawal_admin: Pubkey,

    /// The withdrawal fee wallet where withdrawn funds are sent
    pub withdrawal_fee_wallet: Pubkey,

    /// The voter pubkey can be used as the voter for signing transactions for interacting
    /// with various NCN programs. NCNs can also opt for their own signing infrastructure.
    pub voter: Pubkey,

    /// The operator index
    pub index: u64,

    /// The number of NcnOperatorTickets associated with the operator.
    /// Helpful for indexing all available OperatorNcnTickets.
    pub ncn_count: u64,

    /// The number of NcnVaultTickets associated with the operator.
    /// Helpful for indexing all available OperatorVaultTickets.
    pub vault_count: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    pub reserved_space: [u8; 7],
}

impl Operator {
    /// Create a new Operator account
    /// # Arguments
    /// * `base` - The base account used as a PDA seed
    /// * `admin` - The admin of the Operator
    /// * `index` - The index of the Operator
    /// * `bump` - The bump seed for the PDA
    pub const fn new(base: Pubkey, admin: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            base,
            admin,
            ncn_admin: admin,
            vault_admin: admin,
            withdrawal_admin: admin,
            withdrawal_fee_wallet: admin,
            voter: admin,
            index,
            ncn_count: 0,
            vault_count: 0,
            bump,
            reserved_space: [0; 7],
        }
    }

    /// Check admin validity and signature
    pub fn check_admin(&self, admin_info: &AccountInfo) -> Result<(), ProgramError> {
        if *admin_info.key != self.admin {
            msg!(
                "Incorrect admin provided, expected {}, received {}",
                self.admin,
                admin_info.key
            );
            return Err(RestakingError::OperatorAdminInvalid.into());
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

        if self.withdrawal_admin.eq(old_admin) {
            self.withdrawal_admin = *new_admin;
            msg!("Withdrawal admin set to {:?}", new_admin);
        }

        if self.withdrawal_fee_wallet.eq(old_admin) {
            self.withdrawal_fee_wallet = *new_admin;
            msg!("Withdrawal fee wallet set to {:?}", new_admin);
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
    use solana_program::pubkey::Pubkey;

    use crate::operator::Operator;

    #[test]
    fn test_update_secondary_admin_ok() {
        let old_admin = Pubkey::new_unique();
        let mut operator = Operator::new(Pubkey::new_unique(), old_admin, 0, 0);

        assert_eq!(operator.ncn_admin, old_admin);
        assert_eq!(operator.vault_admin, old_admin);
        assert_eq!(operator.voter, old_admin);
        assert_eq!(operator.withdrawal_admin, old_admin);
        assert_eq!(operator.withdrawal_fee_wallet, old_admin);

        let new_admin = Pubkey::new_unique();
        operator.update_secondary_admin(&old_admin, &new_admin);

        assert_eq!(operator.ncn_admin, new_admin);
        assert_eq!(operator.vault_admin, new_admin);
        assert_eq!(operator.voter, new_admin);
        assert_eq!(operator.withdrawal_admin, new_admin);
        assert_eq!(operator.withdrawal_fee_wallet, new_admin);
    }
}
