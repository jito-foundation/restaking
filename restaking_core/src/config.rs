//! Global configuration account for the restaking program
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::{clock::DEFAULT_SLOTS_PER_EPOCH, msg, pubkey::Pubkey};

/// The discriminator for the global configuration account
impl Discriminator for Config {
    const DISCRIMINATOR: u8 = 1;
}

/// The global configuration account for the restaking program. Manages
/// program-wide settings and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Config {
    /// The configuration admin
    pub admin: Pubkey,

    /// The vault program
    pub vault_program: Pubkey,

    /// The number of NCN managed by the program
    pub ncn_count: u64,

    /// The number of operators managed by the program
    pub operator_count: u64,

    /// The length of an epoch in slots
    pub epoch_length: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved_1: [u8; 7],
}

impl Config {
    pub const fn new(admin: Pubkey, vault_program: Pubkey, bump: u8) -> Self {
        Self {
            admin,
            vault_program,
            epoch_length: DEFAULT_SLOTS_PER_EPOCH,
            ncn_count: 0,
            operator_count: 0,
            bump,
            reserved_1: [0; 7],
        }
    }

    /// Returns the seeds for the PDA
    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"config".to_vec()]
    }

    /// Find the program address for the global configuration account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// # Returns
    /// * `Pubkey` - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn find_program_address(program_id: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds();
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Attempts to load the account as [`Config`], returning an error if it's not valid.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load the configuration from
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
            msg!("Config account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("Config account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("Config account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Config::DISCRIMINATOR) {
            msg!("Config account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.key.ne(&Config::find_program_address(program_id).0) {
            msg!("Config account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
