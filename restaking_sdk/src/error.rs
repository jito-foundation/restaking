use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RestakingError {
    #[error("NcnOperatorAdminInvalid")]
    NcnOperatorAdminInvalid = 1000,
    #[error("NcnCooldownOperatorFailed")]
    NcnCooldownOperatorFailed,
    #[error("NcnSlasherAdminInvalid")]
    NcnSlasherAdminInvalid,
    #[error("NcnVaultAdminInvalid")]
    NcnVaultAdminInvalid,
    #[error("NcnAdminInvalid")]
    NcnAdminInvalid,
    #[error("NcnDelegateAdminInvalid")]
    NcnDelegateAdminInvalid,
    #[error("NcnVaultSlasherTicketFailedCooldown")]
    NcnVaultSlasherTicketFailedCooldown,
    #[error("NcnVaultTicketFailedCooldown")]
    NcnVaultTicketFailedCooldown,
    #[error("NcnWarmupOperatorFailed")]
    NcnWarmupOperatorFailed,
    #[error("NcnVaultSlasherTicketFailedWarmup")]
    NcnVaultSlasherTicketFailedWarmup,
    #[error("NcnVaultTicketFailedWarmup")]
    NcnVaultTicketFailedWarmup,

    #[error("OperatorNcnAdminInvalid")]
    OperatorNcnAdminInvalid = 2000,
    #[error("OperatorVaultAdminInvalid")]
    OperatorVaultAdminInvalid,
    #[error("OperatorAdminInvalid")]
    OperatorAdminInvalid,
    #[error("OperatorDelegateAdminInvalid")]
    OperatorDelegateAdminInvalid,
    #[error("OperatorCooldownNcnFailed")]
    OperatorCooldownNcnFailed,
    #[error("OperatorVaultTicketFailedCooldown")]
    OperatorVaultTicketFailedCooldown,
    #[error("OperatorVaultTicketFailedWarmup")]
    OperatorVaultTicketFailedWarmup,
    #[error("OperatorWarmupNcnFailed")]
    OperatorWarmupNcnFailed,
    #[error("OperatorFeeCapExceeded")]
    OperatorFeeCapExceeded,
    #[error("NcnOverflow")]
    NcnOverflow,
    #[error("OperatorOverflow")]
    OperatorOverflow,
    #[error("VaultOverflow")]
    VaultOverflow,
    #[error("SlasherOverflow")]
    SlasherOverflow,
    #[error("InvalidEpochLength")]
    InvalidEpochLength,

    #[error("ArithmeticOverflow")]
    ArithmeticOverflow = 3000,
    #[error("ArithmeticUnderflow")]
    ArithmeticUnderflow,
    #[error("DivisionByZero")]
    DivisionByZero,
}

impl<T> DecodeError<T> for RestakingError {
    fn type_of() -> &'static str {
        "jito::restaking"
    }
}

impl From<RestakingError> for ProgramError {
    fn from(e: RestakingError) -> Self {
        Self::Custom(e as u32)
    }
}

impl From<RestakingError> for u64 {
    fn from(e: RestakingError) -> Self {
        e as Self
    }
}
