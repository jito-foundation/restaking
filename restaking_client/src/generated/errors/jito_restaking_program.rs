//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum JitoRestakingProgramError {
    /// 1000 - NcnOperatorAdminInvalid
    #[error("NcnOperatorAdminInvalid")]
    NcnOperatorAdminInvalid = 0x3E8,
    /// 1001 - NcnCooldownOperatorFailed
    #[error("NcnCooldownOperatorFailed")]
    NcnCooldownOperatorFailed = 0x3E9,
    /// 1002 - NcnSlasherAdminInvalid
    #[error("NcnSlasherAdminInvalid")]
    NcnSlasherAdminInvalid = 0x3EA,
    /// 1003 - NcnVaultAdminInvalid
    #[error("NcnVaultAdminInvalid")]
    NcnVaultAdminInvalid = 0x3EB,
    /// 1004 - NcnAdminInvalid
    #[error("NcnAdminInvalid")]
    NcnAdminInvalid = 0x3EC,
    /// 1005 - NcnWithdrawAdminInvalid
    #[error("NcnWithdrawAdminInvalid")]
    NcnWithdrawAdminInvalid = 0x3ED,
    /// 1006 - NcnVaultSlasherTicketFailedCooldown
    #[error("NcnVaultSlasherTicketFailedCooldown")]
    NcnVaultSlasherTicketFailedCooldown = 0x3EE,
    /// 1007 - NcnVaultTicketFailedCooldown
    #[error("NcnVaultTicketFailedCooldown")]
    NcnVaultTicketFailedCooldown = 0x3EF,
    /// 1008 - NcnWarmupOperatorFailed
    #[error("NcnWarmupOperatorFailed")]
    NcnWarmupOperatorFailed = 0x3F0,
    /// 1009 - NcnVaultSlasherTicketFailedWarmup
    #[error("NcnVaultSlasherTicketFailedWarmup")]
    NcnVaultSlasherTicketFailedWarmup = 0x3F1,
    /// 1010 - NcnVaultTicketFailedWarmup
    #[error("NcnVaultTicketFailedWarmup")]
    NcnVaultTicketFailedWarmup = 0x3F2,
    /// 2000 - OperatorNcnAdminInvalid
    #[error("OperatorNcnAdminInvalid")]
    OperatorNcnAdminInvalid = 0x7D0,
    /// 2001 - OperatorVaultAdminInvalid
    #[error("OperatorVaultAdminInvalid")]
    OperatorVaultAdminInvalid = 0x7D1,
    /// 2002 - OperatorAdminInvalid
    #[error("OperatorAdminInvalid")]
    OperatorAdminInvalid = 0x7D2,
    /// 2003 - OperatorWithdrawAdminInvalid
    #[error("OperatorWithdrawAdminInvalid")]
    OperatorWithdrawAdminInvalid = 0x7D3,
    /// 2004 - OperatorCooldownNcnFailed
    #[error("OperatorCooldownNcnFailed")]
    OperatorCooldownNcnFailed = 0x7D4,
    /// 2005 - OperatorVaultTicketFailedCooldown
    #[error("OperatorVaultTicketFailedCooldown")]
    OperatorVaultTicketFailedCooldown = 0x7D5,
    /// 2006 - OperatorVaultTicketFailedWarmup
    #[error("OperatorVaultTicketFailedWarmup")]
    OperatorVaultTicketFailedWarmup = 0x7D6,
    /// 2007 - OperatorWarmupNcnFailed
    #[error("OperatorWarmupNcnFailed")]
    OperatorWarmupNcnFailed = 0x7D7,
}

impl solana_program::program_error::PrintProgramError for JitoRestakingProgramError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
