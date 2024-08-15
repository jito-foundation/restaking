use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum RestakingError {
    NcnOperatorAdminInvalid = 1000,
    NcnCooldownOperatorFailed = 1001,
    NcnSlasherAdminInvalid = 1002,
    NcnVaultAdminInvalid = 1003,
    NcnAdminInvalid = 1004,
    NcnWithdrawAdminInvalid = 1005,
    NcnVaultSlasherTicketFailedCooldown = 1006,
    NcnVaultTicketFailedCooldown = 1007,
    NcnWarmupOperatorFailed = 1008,
    NcnVaultSlasherTicketFailedWarmup = 1009,
    NcnVaultTicketFailedWarmup = 1010,
    NcnVaultTicketNotActive = 1011,

    OperatorNcnAdminInvalid = 2000,
    OperatorVaultAdminInvalid = 2001,
    OperatorAdminInvalid = 2002,
    OperatorWithdrawAdminInvalid = 2003,
    OperatorCooldownNcnFailed = 2004,
    OperatorVaultTicketFailedCooldown = 2005,
    OperatorVaultTicketFailedWarmup = 2006,
    OperatorNcnTicketNotActive = 2007,
    OperatorWarmupNcnFailed = 2008,
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
