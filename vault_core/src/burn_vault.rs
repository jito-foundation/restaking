use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

/// Uninitiatilized, no-data account used to hold SOL for ClaimStatus rent
/// Must be empty and uninitialized to be used as a payer or `transfer` instructions fail
pub struct BurnVault {}

impl BurnVault {
    // ------------------------------------------
    // Serialization & Deserialization
    // ------------------------------------------

    /// Returns the seeds for the PDA
    ///
    /// # Returns
    /// * `Vec<Vec<u8>>` - containing the seed vectors
    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"burn_vault".as_ref().to_vec(), base.to_bytes().to_vec()]
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

    pub fn load(
        program_id: &Pubkey,
        base: &Pubkey,
        account: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if account.owner.ne(&solana_program::system_program::ID) {
            msg!("Burn Vault account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }

        if expect_writable && !account.is_writable {
            msg!("Burn Vault account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }

        if account
            .key
            .ne(&Self::find_program_address(program_id, base).0)
        {
            msg!("Burn Vault account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use solana_program::{account_info::AccountInfo, rent::Rent, system_program};

    use super::*;

    #[test]
    fn test_seeds_generation() {
        let base = Pubkey::new_unique();
        let seeds = BurnVault::seeds(&base);

        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0], b"burn_vault".as_ref());
        assert_eq!(seeds[1], base.to_bytes().to_vec());
    }

    #[test]
    fn test_find_program_address() {
        let program_id = Pubkey::new_unique();
        let base = Pubkey::new_unique();

        let (pda, bump, seeds) = BurnVault::find_program_address(&program_id, &base);

        // Verify seeds are correct
        assert_eq!(seeds, BurnVault::seeds(&base));

        // Verify PDA derivation
        let seeds_with_bump = seeds.iter().map(|s| s.as_slice()).collect::<Vec<_>>();
        let (expected_pda, expected_bump) =
            Pubkey::find_program_address(&seeds_with_bump, &program_id);

        assert_eq!(pda, expected_pda);
        assert_eq!(bump, expected_bump);
    }

    #[test]
    fn test_load_valid_account() {
        let program_id = Pubkey::new_unique();
        let base = Pubkey::new_unique();
        let (pda, _bump, _) = BurnVault::find_program_address(&program_id, &base);

        // Create a valid account at the PDA
        let mut lamports = 1000000;
        let mut data = vec![];
        let account = AccountInfo::new(
            &pda,
            false,
            true,
            &mut lamports,
            &mut data,
            &system_program::ID,
            false,
            Rent::default().minimum_balance(0),
        );

        // Test load with writable=false
        let result = BurnVault::load(&program_id, &base, &account, false);
        assert!(result.is_ok());

        // Test load with writable=true
        let result = BurnVault::load(&program_id, &base, &account, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_invalid_owner() {
        let program_id = Pubkey::new_unique();
        let base = Pubkey::new_unique();
        let (pda, _bump, _) = BurnVault::find_program_address(&program_id, &base);

        // Create account with invalid owner
        let mut lamports = 1000000;
        let mut data = vec![];
        let invalid_owner = Pubkey::new_unique();
        let account = AccountInfo::new(
            &pda,
            false,
            true,
            &mut lamports,
            &mut data,
            &invalid_owner,
            false,
            Rent::default().minimum_balance(0),
        );

        let result = BurnVault::load(&program_id, &base, &account, false);
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountOwner);
    }

    #[test]
    fn test_load_invalid_pda() {
        let program_id = Pubkey::new_unique();
        let base = Pubkey::new_unique();
        let invalid_pda = Pubkey::new_unique();

        // Create account at wrong address
        let mut lamports = 1000000;
        let mut data = vec![];
        let account = AccountInfo::new(
            &invalid_pda,
            false,
            true,
            &mut lamports,
            &mut data,
            &system_program::ID,
            false,
            Rent::default().minimum_balance(0),
        );

        let result = BurnVault::load(&program_id, &base, &account, false);
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_load_not_writable() {
        let program_id = Pubkey::new_unique();
        let base = Pubkey::new_unique();
        let (pda, _bump, _) = BurnVault::find_program_address(&program_id, &base);

        // Create non-writable account
        let mut lamports = 1000000;
        let mut data = vec![];
        let account = AccountInfo::new(
            &pda,
            false,
            false, // not writable
            &mut lamports,
            &mut data,
            &system_program::ID,
            false,
            Rent::default().minimum_balance(0),
        );

        let result = BurnVault::load(&program_id, &base, &account, true);
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);
    }
}
