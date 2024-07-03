use solana_program::program_error::ProgramError;

pub type VaultCoreResult<T> = Result<T, VaultCoreError>;

#[derive(Debug, PartialEq, Eq)]
pub enum VaultCoreError {
    VaultDepositOverflow,
    VaultDepositExceedsCapacity,
    VaultFeeCalculationOverflow,
    VaultDataEmpty,
    VaultInvalidProgramOwner,
    VaultInvalidData(String),
    VaultInvalidPda,
    VaultExpectedWritable,
    VaultSerializationFailed(String),
    VaultAvsAlreadyActive,
    VaultInvalidAdmin,

    ConfigExpectedWritable,
    VaultAvsListExpectedWritable,
    VaultOperatorListExpectedWritable,
    VaultSlasherListExpectedWritable,
    VaultAvsListDataEmpty,
    VaultAvsListInvalidProgramOwner,
    VaultAvsListInvalidData(String),
    VaultAvsListInvalidAccountType,
    VaultAvsListInvalidPda,
    VaultOperatorListDataEmpty,
    VaultOperatorListInvalidProgramOwner,
    VaultOperatorListInvalidData(String),
    VaultOperatorListInvalidAccountType,
    VaultOperatorListInvalidPda,
    VaultSlasherListInvalidPda,
    VaultSlasherListInvalidAccountType,
    VaultSlasherListInvalidData(String),
    VaultSlasherListInvalidProgramOwner,
    VaultSlasherListDataEmpty,
    ConfigInvalidPda,
    ConfigInvalidAccountType,
    ConfigInvalidData(String),
    ConfigInvalidProgramOwner,
    ConfigDataEmpty,
    VaultAvsNotSupported,
    VaultAvsAlreadyInactive,
    VaultOperatorListOperatorAlreadyAdded,
    VaultOperatorListOperatorAlreadyRemoved,
    VaultOperatorListOperatorNotAdded,
    VaultInvalidDelegationAdmin,
}

impl From<VaultCoreError> for ProgramError {
    fn from(_value: VaultCoreError) -> Self {
        Self::Custom(0)
        // match value {
        //     VaultCoreError::VaultDepositOverflow => ProgramError::Custom(1000),
        //     VaultCoreError::VaultDepositExceedsCapacity => ProgramError::Custom(1001),
        //     VaultCoreError::VaultFeeCalculationOverflow => ProgramError::Custom(1002),
        //     VaultCoreError::VaultDataEmpty => ProgramError::Custom(1003),
        //     VaultCoreError::VaultInvalidProgramOwner => ProgramError::Custom(1004),
        //     VaultCoreError::VaultInvalidData(_) => ProgramError::Custom(1005),
        //     VaultCoreError::VaultInvalidPda => ProgramError::Custom(1006),
        //     VaultCoreError::VaultNotWritable => ProgramError::Custom(1007),
        //     VaultCoreError::VaultSerializationFailed(_) => ProgramError::Custom(1008),
        //     VaultCoreError::VaultAvsAlreadyActive => ProgramError::Custom(1009),
        //     VaultCoreError::VaultInvalidAdmin => ProgramError::Custom(1010),
        // }
    }
}
