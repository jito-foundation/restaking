//! The vault configuration account

use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{
    types::{PodU16, PodU64},
    AccountDeserialize, Discriminator,
};
use jito_vault_sdk::error::VaultError;
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, epoch_schedule::DEFAULT_SLOTS_PER_EPOCH, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::MAX_BPS;

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
    epoch_length: PodU64,

    /// The number of vaults managed by the program
    num_vaults: PodU64,

    /// The fee cap in basis points ( withdraw and deposit )
    deposit_withdrawal_fee_cap_bps: PodU16,

    /// The maximum amount a fee can increase per epoch in basis points
    fee_rate_of_change_bps: PodU16,

    /// The amount a fee can increase above the rate of change in basis points
    fee_bump_bps: PodU16,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 263],
}

impl Config {
    /// Maximum fee cap in basis points
    pub const DEFAULT_FEES_CAP_BPS: u16 = 2_000; // 20%
    /// Maximum rate of change in fee basis pointer per epoch
    pub const DEFAULT_FEE_RATE_OF_CHANGE_BPS: u16 = 2_500; // 25%
    /// Maximum bump in fee change above the rate of change
    pub const DEFAULT_FEE_BUMP_BPS: u16 = 10; // 0.1%

    pub fn new(admin: Pubkey, restaking_program: Pubkey, bump: u8) -> Self {
        Self {
            admin,
            restaking_program,
            epoch_length: PodU64::from(DEFAULT_SLOTS_PER_EPOCH),
            num_vaults: PodU64::from(0),
            // Cannot be higher than 100%
            deposit_withdrawal_fee_cap_bps: PodU16::from(Self::DEFAULT_FEES_CAP_BPS),
            fee_rate_of_change_bps: PodU16::from(Self::DEFAULT_FEE_RATE_OF_CHANGE_BPS),
            fee_bump_bps: PodU16::from(Self::DEFAULT_FEE_BUMP_BPS),
            bump,
            reserved: [0; 263],
        }
    }

    pub fn epoch_length(&self) -> u64 {
        self.epoch_length.into()
    }

    pub fn num_vaults(&self) -> u64 {
        self.num_vaults.into()
    }

    pub fn deposit_withdrawal_fee_cap_bps(&self) -> u16 {
        u16::from(self.deposit_withdrawal_fee_cap_bps)
    }

    pub fn fee_rate_of_change_bps(&self) -> u16 {
        u16::from(self.fee_rate_of_change_bps)
    }

    pub fn fee_bump_bps(&self) -> u16 {
        u16::from(self.fee_bump_bps)
    }

    pub fn set_deposit_withdrawal_fee_cap_bps(
        &mut self,
        fee_cap_bps: u16,
    ) -> Result<(), VaultError> {
        if fee_cap_bps > MAX_BPS {
            return Err(VaultError::VaultFeeCapExceeded);
        }
        self.deposit_withdrawal_fee_cap_bps = PodU16::from(fee_cap_bps);
        Ok(())
    }

    pub fn set_fee_rate_of_change_bps(
        &mut self,
        fee_rate_of_change_bps: u16,
    ) -> Result<(), VaultError> {
        if fee_rate_of_change_bps > MAX_BPS {
            return Err(VaultError::VaultFeeCapExceeded);
        }
        self.fee_rate_of_change_bps = PodU16::from(fee_rate_of_change_bps);
        Ok(())
    }

    pub fn set_fee_bump_bps(&mut self, fee_bump_bps: u16) -> Result<(), VaultError> {
        if fee_bump_bps > MAX_BPS {
            return Err(VaultError::VaultFeeCapExceeded);
        }
        self.fee_bump_bps = PodU16::from(fee_bump_bps);
        Ok(())
    }

    pub fn increment_num_vaults(&mut self) -> Result<(), VaultError> {
        let mut num_vaults: u64 = self.num_vaults.into();
        num_vaults = num_vaults.checked_add(1).ok_or(VaultError::VaultOverflow)?;
        self.num_vaults = PodU64::from(num_vaults);
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_no_padding() {
        let config_size = std::mem::size_of::<Config>();
        let sum_of_fields = std::mem::size_of::<Pubkey>() + // admin
            std::mem::size_of::<Pubkey>() + // restaking_program
            std::mem::size_of::<PodU64>() + // epoch_length
            std::mem::size_of::<PodU64>() + // num_vaults
            std::mem::size_of::<PodU16>() + // fee_cap_bps
            std::mem::size_of::<PodU16>() + // fee_rate_of_change_bps
            std::mem::size_of::<PodU16>() + // fee_bump_bps
            std::mem::size_of::<u8>() + // bump
            263; // reserved
        assert_eq!(config_size, sum_of_fields);
    }

    #[test]
    fn test_bps_are_within_bounds() {
        assert!(Config::DEFAULT_FEES_CAP_BPS <= MAX_BPS);
        assert!(Config::DEFAULT_FEE_RATE_OF_CHANGE_BPS <= MAX_BPS);
        assert!(Config::DEFAULT_FEE_BUMP_BPS <= MAX_BPS);
    }

    #[test]
    fn test_set_fee_cap_bps() {
        let mut config = Config::new(Pubkey::new_unique(), Pubkey::new_unique(), 0);
        assert_eq!(config.set_deposit_withdrawal_fee_cap_bps(1), Ok(()));
        assert_eq!(config.set_deposit_withdrawal_fee_cap_bps(MAX_BPS), Ok(()));
        assert_eq!(
            config.set_deposit_withdrawal_fee_cap_bps(MAX_BPS + 1),
            Err(VaultError::VaultFeeCapExceeded)
        );
    }

    #[test]
    fn test_set_fee_rate_of_change_bps() {
        let mut config = Config::new(Pubkey::new_unique(), Pubkey::new_unique(), 0);
        assert_eq!(config.set_fee_rate_of_change_bps(1), Ok(()));
        assert_eq!(config.set_fee_rate_of_change_bps(MAX_BPS), Ok(()));
        assert_eq!(
            config.set_fee_rate_of_change_bps(MAX_BPS + 1),
            Err(VaultError::VaultFeeCapExceeded)
        );
    }

    #[test]
    fn test_set_fee_bump_bps() {
        let mut config = Config::new(Pubkey::new_unique(), Pubkey::new_unique(), 0);
        assert_eq!(config.set_fee_bump_bps(1), Ok(()));
        assert_eq!(config.set_fee_bump_bps(MAX_BPS), Ok(()));
        assert_eq!(
            config.set_fee_bump_bps(MAX_BPS + 1),
            Err(VaultError::VaultFeeCapExceeded)
        );
    }
}
