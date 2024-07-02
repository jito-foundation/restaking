use solana_program::program_error::ProgramError;

pub type VaultCoreResult<T> = Result<T, VaultCoreError>;

#[derive(Debug, PartialEq, Eq)]
pub enum VaultCoreError {
    DepositOverflow,
    DepositExceedsCapacity,
    FeeCalculationOverflow,
}

impl From<VaultCoreError> for ProgramError {
    fn from(value: VaultCoreError) -> Self {
        match value {
            VaultCoreError::DepositOverflow => Self::Custom(1000),
            VaultCoreError::DepositExceedsCapacity => Self::Custom(1001),
            VaultCoreError::FeeCalculationOverflow => Self::Custom(1002),
        }
    }
}
