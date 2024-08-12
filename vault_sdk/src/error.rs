use solana_program::program_error::ProgramError;

#[derive(Debug, PartialEq, Eq)]
pub enum VaultError {
    OperatorDelegationTotalSecurityOverflow = 1000,
    OperatorDelegationWithdrawableSecurityOverflow = 1001,
    OperatorDelegationSlashExceedsTotalSecurity = 1002,
    OperatorDelegationSlashOverflow = 1003,
    OperatorDelegationSlashUnderflow = 1004,
    OperatorDelegationSlashIncomplete = 1005,
    OperatorDelegationUndelegateUnderflow = 1006,
    OperatorDelegationUndelegateOverflow = 1007,
    OperatorDelegationDelegateOverflow = 1008,

    VaultMaxDelegationOverflow = 2000,
    VaultLrtEmpty = 2001,
    VaultInsufficientFunds = 2002,
    VaultAssetsReturnedOverflow = 2003,
    VaultOverflow = 2004,

    VaultDelegationListOverflow = 3000,
    VaultDelegationListUnderflow = 3001,
    VaultDelegationListInsufficientSecurity = 3002,
    VaultDelegationListFull = 3003,
    VaultDelegationListUndelegateIncomplete = 3004,
    VaultDelegationListOperatorNotFound = 3005,
}

impl From<VaultError> for ProgramError {
    fn from(e: VaultError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<VaultError> for u64 {
    fn from(e: VaultError) -> Self {
        e as u64
    }
}
