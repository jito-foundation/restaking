use solana_program::program_error::ProgramError;

pub type RestakingCoreResult<T> = Result<T, RestakingCoreError>;

#[derive(Debug, PartialEq, Eq)]
pub enum RestakingCoreError {
    AvsInvalidAdmin,
    VaultFailedToActivate,
    VaultFailedToDeactivate,
    VaultNotFound,
    OperatorInvalidAdmin,
    VaultNotActive,
    AvsInvalidOperatorAdmin,
    AvsInvalidVaultAdmin,
    OperatorAlreadyAdded,
    AvsNotActive,
    AvsNotFound,
    AvsInvalidSlasherAdmin,
    VaultNotActiveOrCoolingDown,
    VaultSlasherAlreadyExists,
    VaultSlasherNotActive,
    VaultSlasherNotFound,
    OperatorAlreadyRemoved,
    OperatorNotFound,
    AvsInvalidWithdrawAdmin,
    AvsFailedToActivate,
    AvsFailedToDeactivate,
    SlasherNotActive,
    OperatorNotActive,
}

impl From<RestakingCoreError> for ProgramError {
    fn from(_value: RestakingCoreError) -> Self {
        Self::Custom(0)
    }
}
