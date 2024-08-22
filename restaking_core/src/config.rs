//! Global configuration account for the restaking program

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use jito_restaking_sdk::error::RestakingError;
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, clock::DEFAULT_SLOTS_PER_EPOCH, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// The discriminator for the global configuration account
impl Discriminator for Config {
    const DISCRIMINATOR: u8 = 1;
}

/// The global configuration account for the restaking program. Manages
/// program-wide settings and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Config {
    /// The configuration admin
    pub admin: Pubkey,

    /// The vault program
    pub vault_program: Pubkey,

    /// The number of NCN managed by the program
    ncn_count: PodU64,

    /// The number of operators managed by the program
    operator_count: PodU64,

    /// The length of an epoch in slots
    epoch_length: PodU64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved_1: [u8; 7],
}

impl Config {
    /// Creates a new [`Config`].
    ///
    /// # Arguments
    /// * `admin` - A `Pubkey` representing the admin's public key, which will be responsible for managing this config.
    /// * `vault_program` - A `Pubkey` representing the jito-vault-program program id.
    /// * `bump` - A `u8` value used for the program-derived address (PDA) bump seed, ensuring the correct PDA is generated.
    ///
    /// # Returns
    /// * `Self` - Returns a new instance of the config, initialized with the provided arguments and default values.
    pub fn new(admin: Pubkey, vault_program: Pubkey, bump: u8) -> Self {
        Self {
            admin,
            vault_program,
            epoch_length: PodU64::from(DEFAULT_SLOTS_PER_EPOCH),
            ncn_count: PodU64::from(0),
            operator_count: PodU64::from(0),
            bump,
            reserved_1: [0; 7],
        }
    }

    /// Returns the length of an epoch in slots.
    ///
    /// # Returns
    /// * `u64` - The length of the epoch in slots, represented as a `u64` value.
    pub fn epoch_length(&self) -> u64 {
        self.epoch_length.into()
    }

    /// Returns the number of NCNs managed by the program.
    ///
    /// # Returns
    /// * `u64` - The number of NCNs managed by the program, represented as a `u64` value.
    pub fn ncn_count(&self) -> u64 {
        self.ncn_count.into()
    }

    /// Returns the number of operators managed by the program.
    ///
    /// # Returns
    /// * `u64` - The number of operators managed by the program, represented as a `u64` value.
    pub fn operator_count(&self) -> u64 {
        self.operator_count.into()
    }

    /// Increments the count of NCNs by one.
    ///
    /// # Returns
    /// * `Result<(), RestakingError>` - Returns `Ok(())` if the increment is successful.
    ///   If the increment operation causes an overflow, it returns a `RestakingError::NcnOverflow` error.
    ///
    /// # Errors
    /// This function will return a `RestakingError::NcnOverflow` error in the following case:
    /// * The `ncn_count` increment causes an overflow, meaning the count has reached the maximum value
    ///   representable by a `u64` and cannot be safely incremented further.
    pub fn increment_ncn_count(&mut self) -> Result<(), RestakingError> {
        let ncn_count = self
            .ncn_count()
            .checked_add(1)
            .ok_or(RestakingError::NcnOverflow)?;
        self.ncn_count = PodU64::from(ncn_count);
        Ok(())
    }

    /// Increments the count of operators by one.
    ///
    /// # Returns
    /// * `Result<(), RestakingError>` - Returns `Ok(())` if the increment is successful.
    ///   If the increment operation causes an overflow, it returns a `RestakingError::OperatorOverflow` error.
    ///
    /// # Errors
    /// This function will return a `RestakingError::OperatorOverflow` error in the following case:
    /// * The `operator_count` increment causes an overflow, meaning the count has reached the maximum value
    ///   representable by a `u64` and cannot be safely incremented further.
    pub fn increment_operator_count(&mut self) -> Result<(), RestakingError> {
        let operator_count = self
            .operator_count()
            .checked_add(1)
            .ok_or(RestakingError::OperatorOverflow)?;
        self.operator_count = PodU64::from(operator_count);
        Ok(())
    }

    /// Returns the seeds for the PDA
    ///
    /// # Returns
    /// * `Vec<Vec<u8>>` - A vector of byte vectors containing the seeds used for deriving the PDA.
    fn seeds() -> Vec<Vec<u8>> {
        vec![b"config".to_vec()]
    }

    /// Find the program address for the global configuration account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    ///
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
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Config account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.key.ne(&Self::find_program_address(program_id).0) {
            msg!("Config account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
