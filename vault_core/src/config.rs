//! The vault configuration account
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, epoch_schedule::DEFAULT_SLOTS_PER_EPOCH, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

impl Discriminator for Config {
    const DISCRIMINATOR: u8 = 1;
}

/// The vault configuration account for the vault program.
/// Manages program-wide settings and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Config {
    /// The configuration admin
    pub admin: Pubkey,

    /// The approved restaking program for this vault
    pub restaking_program: Pubkey,

    /// The length of an epoch in slots
    pub epoch_length: u64,

    /// The number of vaults managed by the program
    pub num_vaults: u64,

    /// The fee cap in basis points ( withdraw and deposit )
    pub fee_cap_bps: u16,

    /// The maximum amount a fee can increase per epoch in basis points
    pub fee_rate_of_change_bps: u16,

    /// The amount a fee can increase above the rate of change in basis points
    pub fee_bump_bps: u16,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 17],
}

impl Config {
    /// Maximum fee cap in basis points
    pub const DEFAULT_FEES_CAP_BPS: u16 = 2_000; // 2%
    /// Maximum rate of change in fee baisis pointer per epoch
    pub const DEFAULT_FEE_RATE_OF_CHANGE_BPS: u16 = 2_500; // 25%
    /// Maximum bump in fee change above the rate of change
    pub const DEFAULT_FEE_BUMP_BPS: u16 = 10; // 0.1%
    /// 100% in basis points
    pub const MAX_BPS: u16 = 10_000;

    pub const fn new(admin: Pubkey, restaking_program: Pubkey, bump: u8) -> Self {
        Self {
            admin,
            restaking_program,
            epoch_length: DEFAULT_SLOTS_PER_EPOCH,
            num_vaults: 0,
            // Cannot be higher than 100%
            fee_cap_bps: Self::DEFAULT_FEES_CAP_BPS,
            fee_rate_of_change_bps: Self::DEFAULT_FEE_RATE_OF_CHANGE_BPS,
            fee_bump_bps: Self::DEFAULT_FEE_BUMP_BPS,
            bump,
            reserved: [0; 17],
        }
    }

    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"config".to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds();
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the vault [`Config`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load
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
