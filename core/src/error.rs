use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("Bad epoch length")]
    BadEpochLength,
}

impl<T> DecodeError<T> for CoreError {
    fn type_of() -> &'static str {
        "jito::core"
    }
}

impl From<CoreError> for ProgramError {
    fn from(e: CoreError) -> Self {
        Self::Custom(e as u32)
    }
}

impl From<CoreError> for u64 {
    fn from(e: CoreError) -> Self {
        e as Self
    }
}

impl From<CoreError> for u32 {
    fn from(e: CoreError) -> Self {
        e as Self
    }
}
