//! Trait that can be used when working with Solana structs that are used as accounts.

pub mod types;

use bytemuck::Pod;
pub use jito_account_traits_derive::AccountDeserialize;
use solana_program::{msg, program_error::ProgramError};

pub trait Discriminator {
    const DISCRIMINATOR: u8;
}

pub trait AccountDeserialize: Sized + Pod + Discriminator {
    /// Deserialize the account data into a struct.
    /// It assumes the first byte is the discriminator and the next seven bytes are reserved.
    /// The rest of the data is deserialized into the struct.
    ///
    /// # Arguments
    /// * `data` - The account data to deserialize
    ///
    /// # Returns
    /// * `Result<&Self, ProgramError>` - The deserialized struct as a reference or an error
    fn try_from_slice_unchecked(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.first() != Some(&Self::DISCRIMINATOR) {
            msg!(
                "Discriminator is invalid; expected {}, got {}",
                Self::DISCRIMINATOR,
                data.first().unwrap()
            );
            return Err(ProgramError::InvalidAccountData);
        }
        bytemuck::try_from_bytes(&data[8..]).map_err(|_| ProgramError::InvalidAccountData)
    }

    /// Deserialize the account data into a mutable struct.
    /// It assumes the first byte is the discriminator and the next seven bytes are reserved.
    /// The rest of the data is deserialized into the struct.
    ///
    /// # Arguments
    /// * `data` - The account data to deserialize
    ///
    /// # Returns
    /// * `Result<&mut Self, ProgramError>` - The deserialized struct as a reference or an error
    fn try_from_slice_unchecked_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.first() != Some(&Self::DISCRIMINATOR) {
            msg!(
                "Discriminator is invalid; expected {}, got {}",
                Self::DISCRIMINATOR,
                data.first().unwrap()
            );
            return Err(ProgramError::InvalidAccountData);
        }
        bytemuck::try_from_bytes_mut(&mut data[8..]).map_err(|_| ProgramError::InvalidAccountData)
    }
}
