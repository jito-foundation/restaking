//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum JitoVaultProgramError {
    /// 1000 - VaultSlashUnderflow
    #[error("VaultSlashUnderflow")]
    VaultSlashUnderflow = 0x3E8,
    /// 1001 - VaultInsufficientFunds
    #[error("VaultInsufficientFunds")]
    VaultInsufficientFunds = 0x3E9,
    /// 1002 - VaultOverflow
    #[error("VaultOverflow")]
    VaultOverflow = 0x3EA,
    /// 1003 - VaultOperatorAdminInvalid
    #[error("VaultOperatorAdminInvalid")]
    VaultOperatorAdminInvalid = 0x3EB,
    /// 1004 - VaultAdminInvalid
    #[error("VaultAdminInvalid")]
    VaultAdminInvalid = 0x3EC,
    /// 1005 - VaultCapacityAdminInvalid
    #[error("VaultCapacityAdminInvalid")]
    VaultCapacityAdminInvalid = 0x3ED,
    /// 1006 - VaultMintBurnAdminInvalid
    #[error("VaultMintBurnAdminInvalid")]
    VaultMintBurnAdminInvalid = 0x3EE,
    /// 1007 - VaultDelegationAdminInvalid
    #[error("VaultDelegationAdminInvalid")]
    VaultDelegationAdminInvalid = 0x3EF,
    /// 1008 - VaultCapacityExceeded
    #[error("VaultCapacityExceeded")]
    VaultCapacityExceeded = 0x3F0,
    /// 1009 - VaultSlasherAdminInvalid
    #[error("VaultSlasherAdminInvalid")]
    VaultSlasherAdminInvalid = 0x3F1,
    /// 1010 - VaultNcnAdminInvalid
    #[error("VaultNcnAdminInvalid")]
    VaultNcnAdminInvalid = 0x3F2,
    /// 1011 - VaultFeeAdminInvalid
    #[error("VaultFeeAdminInvalid")]
    VaultFeeAdminInvalid = 0x3F3,
    /// 1012 - VaultFeeCapExceeded
    #[error("VaultFeeCapExceeded")]
    VaultFeeCapExceeded = 0x3F4,
    /// 1013 - VaultFeeChangeTooSoon
    #[error("VaultFeeChangeTooSoon")]
    VaultFeeChangeTooSoon = 0x3F5,
    /// 1014 - VaultFeeBumpTooLarge
    #[error("VaultFeeBumpTooLarge")]
    VaultFeeBumpTooLarge = 0x3F6,
    /// 1015 - VaultUnderflow
    #[error("VaultUnderflow")]
    VaultUnderflow = 0x3F7,
    /// 1016 - VaultUpdateNeeded
    #[error("VaultUpdateNeeded")]
    VaultUpdateNeeded = 0x3F8,
    /// 1017 - VaultIsUpdated
    #[error("VaultIsUpdated")]
    VaultIsUpdated = 0x3F9,
    /// 1018 - VaultUpdateIncorrectIndex
    #[error("VaultUpdateIncorrectIndex")]
    VaultUpdateIncorrectIndex = 0x3FA,
    /// 1019 - VaultUpdateStateNotFinishedUpdating
    #[error("VaultUpdateStateNotFinishedUpdating")]
    VaultUpdateStateNotFinishedUpdating = 0x3FB,
    /// 1020 - VaultSecurityOverflow
    #[error("VaultSecurityOverflow")]
    VaultSecurityOverflow = 0x3FC,
    /// 1021 - VaultSlashIncomplete
    #[error("VaultSlashIncomplete")]
    VaultSlashIncomplete = 0x3FD,
    /// 1022 - VaultSecurityUnderflow
    #[error("VaultSecurityUnderflow")]
    VaultSecurityUnderflow = 0x3FE,
    /// 1023 - SlippageError
    #[error("SlippageError")]
    SlippageError = 0x3FF,
    /// 1024 - VaultStakerWithdrawalTicketNotWithdrawable
    #[error("VaultStakerWithdrawalTicketNotWithdrawable")]
    VaultStakerWithdrawalTicketNotWithdrawable = 0x400,
    /// 1025 - VaultNcnSlasherTicketFailedCooldown
    #[error("VaultNcnSlasherTicketFailedCooldown")]
    VaultNcnSlasherTicketFailedCooldown = 0x401,
    /// 1026 - VaultNcnSlasherTicketFailedWarmup
    #[error("VaultNcnSlasherTicketFailedWarmup")]
    VaultNcnSlasherTicketFailedWarmup = 0x402,
    /// 1027 - VaultNcnTicketFailedCooldown
    #[error("VaultNcnTicketFailedCooldown")]
    VaultNcnTicketFailedCooldown = 0x403,
    /// 1028 - VaultNcnTicketFailedWarmup
    #[error("VaultNcnTicketFailedWarmup")]
    VaultNcnTicketFailedWarmup = 0x404,
    /// 1029 - VaultNcnTicketUnslashable
    #[error("VaultNcnTicketUnslashable")]
    VaultNcnTicketUnslashable = 0x405,
    /// 1030 - OperatorVaultTicketUnslashable
    #[error("OperatorVaultTicketUnslashable")]
    OperatorVaultTicketUnslashable = 0x406,
    /// 1031 - NcnOperatorStateUnslashable
    #[error("NcnOperatorStateUnslashable")]
    NcnOperatorStateUnslashable = 0x407,
    /// 1032 - VaultNcnSlasherTicketUnslashable
    #[error("VaultNcnSlasherTicketUnslashable")]
    VaultNcnSlasherTicketUnslashable = 0x408,
    /// 1033 - NcnVaultTicketUnslashable
    #[error("NcnVaultTicketUnslashable")]
    NcnVaultTicketUnslashable = 0x409,
    /// 1034 - NcnVaultSlasherTicketUnslashable
    #[error("NcnVaultSlasherTicketUnslashable")]
    NcnVaultSlasherTicketUnslashable = 0x40A,
    /// 1035 - VaultMaxSlashedPerOperatorExceeded
    #[error("VaultMaxSlashedPerOperatorExceeded")]
    VaultMaxSlashedPerOperatorExceeded = 0x40B,
    /// 1036 - VaultStakerWithdrawalTicketInvalidStaker
    #[error("VaultStakerWithdrawalTicketInvalidStaker")]
    VaultStakerWithdrawalTicketInvalidStaker = 0x40C,
    /// 1037 - SlasherOverflow
    #[error("SlasherOverflow")]
    SlasherOverflow = 0x40D,
    /// 1038 - NcnOverflow
    #[error("NcnOverflow")]
    NcnOverflow = 0x40E,
    /// 1039 - OperatorOverflow
    #[error("OperatorOverflow")]
    OperatorOverflow = 0x40F,
}

impl solana_program::program_error::PrintProgramError for JitoVaultProgramError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
