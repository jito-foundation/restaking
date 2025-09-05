#[allow(deprecated)]
use solana_decode_error::DecodeError;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VaultError {
    #[error("VaultSlashUnderflow")]
    VaultSlashUnderflow = 1000,
    #[error("VaultInitialAmountFailed")]
    VaultInitialAmountFailed,
    #[error("VaultInsufficientFunds")]
    VaultInsufficientFunds,
    #[error("VaultOverflow")]
    VaultOverflow,
    #[error("VaultOperatorAdminInvalid")]
    VaultOperatorAdminInvalid,
    #[error("VaultAdminInvalid")]
    VaultAdminInvalid,
    #[error("VaultCapacityAdminInvalid")]
    VaultCapacityAdminInvalid,
    #[error("VaultMintBurnAdminInvalid")]
    VaultMintBurnAdminInvalid,
    #[error("VaultDelegationAdminInvalid")]
    VaultDelegationAdminInvalid,
    #[error("VaultDelegateAssetAdminInvalid")]
    VaultDelegateAssetAdminInvalid,
    #[error("VaultCapacityExceeded")]
    VaultCapacityExceeded,
    #[error("VaultSlasherAdminInvalid")]
    VaultSlasherAdminInvalid,
    #[error("VaultNcnAdminInvalid")]
    VaultNcnAdminInvalid,
    #[error("VaultFeeAdminInvalid")]
    VaultFeeAdminInvalid,
    #[error("ConfigAdminInvalid")]
    ConfigAdminInvalid,
    #[error("ConfigFeeAdminInvalid")]
    ConfigFeeAdminInvalid,
    #[error("VaultFeeCapExceeded")]
    VaultFeeCapExceeded,
    #[error("VaultFeeChangeTooSoon")]
    VaultFeeChangeTooSoon,
    #[error("VaultFeeBumpTooLarge")]
    VaultFeeBumpTooLarge,
    #[error("VaultUnderflow")]
    VaultUnderflow,
    #[error("VaultUpdateNeeded")]
    VaultUpdateNeeded,
    #[error("VaultIsUpdated")]
    VaultIsUpdated,
    #[error("VaultOperatorDelegationUpdateNeeded")]
    VaultOperatorDelegationUpdateNeeded,
    #[error("VaultOperatorDelegationIsUpdated")]
    VaultOperatorDelegationIsUpdated,
    #[error("VaultUpdateIncorrectIndex")]
    VaultUpdateIncorrectIndex,
    #[error("VaultUpdateStateNotFinishedUpdating")]
    VaultUpdateStateNotFinishedUpdating,
    #[error("VaultSecurityOverflow")]
    VaultSecurityOverflow,
    #[error("VaultSlashIncomplete")]
    VaultSlashIncomplete,
    #[error("VaultSecurityUnderflow")]
    VaultSecurityUnderflow,
    #[error("SlippageError")]
    SlippageError,
    #[error("VaultStakerWithdrawalTicketNotWithdrawable")]
    VaultStakerWithdrawalTicketNotWithdrawable,
    #[error("VaultNcnSlasherTicketFailedCooldown")]
    VaultNcnSlasherTicketFailedCooldown,
    #[error("VaultNcnSlasherTicketFailedWarmup")]
    VaultNcnSlasherTicketFailedWarmup,
    #[error("VaultNcnTicketFailedCooldown")]
    VaultNcnTicketFailedCooldown,
    #[error("VaultNcnTicketFailedWarmup")]
    VaultNcnTicketFailedWarmup,
    #[error("VaultNcnTicketUnslashable")]
    VaultNcnTicketUnslashable,
    #[error("OperatorVaultTicketUnslashable")]
    OperatorVaultTicketUnslashable,
    #[error("NcnOperatorStateUnslashable")]
    NcnOperatorStateUnslashable,
    #[error("VaultNcnSlasherTicketUnslashable")]
    VaultNcnSlasherTicketUnslashable,
    #[error("NcnVaultTicketUnslashable")]
    NcnVaultTicketUnslashable,
    #[error("NcnVaultSlasherTicketUnslashable")]
    NcnVaultSlasherTicketUnslashable,
    #[error("VaultMaxSlashedPerOperatorExceeded")]
    VaultMaxSlashedPerOperatorExceeded,
    #[error("VaultStakerWithdrawalTicketInvalidStaker")]
    VaultStakerWithdrawalTicketInvalidStaker,
    #[error("SlasherOverflow")]
    SlasherOverflow,
    #[error("NcnOverflow")]
    NcnOverflow,
    #[error("OperatorOverflow")]
    OperatorOverflow,
    #[error("VaultDelegationZero")]
    VaultDelegationZero,
    #[error("VaultCooldownZero")]
    VaultCooldownZero,
    #[error("VaultBurnZero")]
    VaultBurnZero,
    #[error("VaultEnqueueWithdrawalAmountZero")]
    VaultEnqueueWithdrawalAmountZero,
    #[error("VaultMintZero")]
    VaultMintZero,
    #[error("VaultIsPaused")]
    VaultIsPaused,
    #[error("InvalidDepositor")]
    InvalidDepositor,
    #[error("InvalidDepositTokenAccount")]
    InvalidDepositTokenAccount,
    #[error("NoSupportedMintBalanceChange")]
    NoSupportedMintBalanceChange,
    #[error("InvalidEpochLength")]
    InvalidEpochLength,
    #[error("VaultRewardFeeDeltaTooLarge")]
    VaultRewardFeeDeltaTooLarge,
    #[error("VaultRewardFeeIsZero")]
    VaultRewardFeeIsZero,
    #[error("VrtOutCannotBeZero")]
    VrtOutCannotBeZero,
    #[error("NonZeroAdditionalAssetsNeededForWithdrawalAtEndOfUpdate")]
    NonZeroAdditionalAssetsNeededForWithdrawalAtEndOfUpdate,
    #[error("ArithmeticOverflow")]
    ArithmeticOverflow = 3000,
    #[error("ArithmeticUnderflow")]
    ArithmeticUnderflow,
    #[error("DivisionByZero")]
    DivisionByZero,
}

#[allow(deprecated)]
impl<T> DecodeError<T> for VaultError {
    fn type_of() -> &'static str {
        "jito::vault"
    }
}

impl From<VaultError> for ProgramError {
    fn from(e: VaultError) -> Self {
        Self::Custom(e as u32)
    }
}

impl From<VaultError> for u64 {
    fn from(e: VaultError) -> Self {
        e as Self
    }
}

impl From<VaultError> for u32 {
    fn from(e: VaultError) -> Self {
        e as Self
    }
}
