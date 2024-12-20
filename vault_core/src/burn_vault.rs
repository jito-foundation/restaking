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
