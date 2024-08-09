use bytemuck::Pod;
pub use jito_account_traits_derive::{AccountDeserialize, ToBytes};
use solana_program::{msg, program_error::ProgramError};

pub trait Discriminator {
    const DISCRIMINATOR: u8;
}

pub trait AccountDeserialize: Sized + Pod + Discriminator {
    fn try_from_slice(data: &[u8]) -> Result<&Self, ProgramError> {
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

    fn try_from_slice_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
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

pub trait ToBytes {
    fn to_bytes(&self) -> &[u8];
}
