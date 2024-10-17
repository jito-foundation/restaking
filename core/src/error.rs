use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JsmCoreError {
    #[error("JsmCoreArithmeticOverflow")]
    JsmCoreArithmeticOverflow,
    #[error("JsmCoreArithmeticUnderflow")]
    JsmCoreArithmeticUnderflow,
    #[error("JsmCoreDivisionByZero")]
    JsmCoreDivisionByZero,
}

impl<T> DecodeError<T> for JsmCoreError {
    fn type_of() -> &'static str {
        "jito::jsm-core"
    }
}

impl From<JsmCoreError> for ProgramError {
    fn from(e: JsmCoreError) -> Self {
        Self::Custom(e as u32)
    }
}

impl From<JsmCoreError> for u64 {
    fn from(e: JsmCoreError) -> Self {
        e as Self
    }
}

impl From<JsmCoreError> for u32 {
    fn from(e: JsmCoreError) -> Self {
        e as Self
    }
}
