use solana_program::program_error::ProgramError;

pub type VaultCoreResult<T> = Result<T, VaultCoreError>;

#[derive(Debug, PartialEq, Eq)]
pub enum VaultCoreError {
    VaultDepositOverflow,
    VaultDepositExceedsCapacity,
    VaultFeeCalculationOverflow,
    VaultDataEmpty,
    VaultInvalidProgramOwner,
    VaultInvalidData,
    VaultInvalidPda,
    VaultExpectedWritable,
    VaultSerializationFailed(String),
    VaultNcnAlreadyActive,
    VaultInvalidAdmin,

    ConfigExpectedWritable,
    VaultNcnListExpectedWritable,
    VaultDelegationListExpectedWritable,
    VaultSlasherListExpectedWritable,
    VaultNcnListDataEmpty,
    VaultNcnListInvalidProgramOwner,
    VaultNcnListInvalidData(String),
    VaultNcnListInvalidAccountType,
    VaultNcnListInvalidPda,
    VaultDelegationListDataEmpty,
    VaultDelegationListInvalidProgramOwner,
    VaultDelegationListInvalidData(String),
    VaultDelegationListInvalidAccountType,
    VaultDelegationListInvalidPda,
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
    VaultNcnNotSupported,
    VaultNcnAlreadyInactive,
    VaultDelegationListOperatorAlreadyAdded,
    VaultDelegationListOperatorAlreadyRemoved,
    VaultDelegationListOperatorNotAdded,
    VaultInvalidDelegationAdmin,
    VaultSlasherNotFound,
    VaultSlasherNotActive,
    VaultNcnNotActive,
    VaultOperatorNotFound,
    VaultOperatorNotActive,
    VaultSlashingOverflow,
    VaultSlashingUnderflow,
    VaultNcnTicketEmpty,
    VaultNcnTicketInvalidOwner,
    VaultNcnTicketInvalidAccountType,
    VaultNcnTicketInvalidData(String),
    VaultNcnTicketInvalidPda,
    VaultOperatorTicketEmpty,
    VaultOperatorTicketInvalidOwner,
    VaultOperatorTicketInvalidData(String),
    VaultOperatorTicketInvalidAccountType,
    VaultOperatorTicketInvalidPda,
    VaultSlasherTicketEmpty,
    VaultSlasherTicketInvalidOwner,
    VaultSlasherTicketInvalidData(String),
    VaultSlasherTicketInvalidAccountType,
    VaultSlasherTicketInvalidPda,
    VaultInvalidNcnAdmin,
    VaultInvalidOperatorAdmin,
    VaultNcnTicektNotWritable,
    VaultOperatorTicketNotWritable,
    VaultSlasherTicketNotWritable,
    VaultOperatorTicketAlreadyDeactivated,
    VaultNcnTicketInactive,
    VaultOperatorTicketInactive,
    VaultNcnSlasherTicketInactive,
    VaultInvalidSlasherAdmin,
    VaultNcnOverflow,
    VaultSlasherOverflow,
    VaultOperatorOverflow,
    VaultNcnSlasherOperatorDataEmpty,
    VaultNcnSlasherOperatorInvalidOwner,
    VaultNcnSlasherOperatorInvalidData(String),
    VaultNcnSlasherOperatorInvalidAccountType,
    VaultNcnSlasherOperatorInvalidPda,
    VaultNcnSlasherOperatorNotWritable,
    VaultNcnSlasherOperatorOverflow,
    VaultNcnSlasherOperatorMaxSlashableExceeded,
    VaultStakerWithdrawalTicketEmpty,
    VaultStakerWithdrawalTicketEmptyInvalidOwner,
    VaultStakerWithdrawalTicketEmptyInvalidData(String),
    VaultStakerWithdrawalTicketEmptyInvalidAccountType,
    VaultStakerWithdrawalTicketEmptyInvalidPda,
    VaultStakerWithdrawalTicketInvalidProgramOwner,
    VaultStakerWithdrawalTicketNotWritable,
    VaultOperatorActiveStakeOverflow,
    VaultDelegationListUpdateOverflow,
    VaultDelegationListTotalDelegationOverflow,
    VaultDelegationUnderflow,
    VaultDelegationOverflow,
    VaultSlashingDivisionByZero,
    VaultSlashingIncomplete,
    VaultWithdrawOverflow,
    VaultEmpty,
    VaultInsufficientFunds,
    VaultDelegationListInsufficientSecurity,
    WithdrawAmountExceedsDelegatedFunds,
    ArithmeticOverflow,
    ArithmeticUnderflow,
    UndelegationIncomplete,
    VaultDelegationListAmountWithdrawableUnderflow,
    VaultDelegationListUpdateRequired,
    VaultStakerWithdrawalTicketOverflow,
    VaultStakerWithdrawalTicketNotWithdrawable,
    VaultUndelegationUnderflow,
    VaultDepositUnderflow,
    VaultDelegationListFull,
}

impl From<VaultCoreError> for ProgramError {
    fn from(_value: VaultCoreError) -> Self {
        Self::Custom(0)
    }
}
