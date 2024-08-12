use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum RestakingError {
    NcnOperatorAdminInvalid = 1000,
    NcnOperatorTicketFailedCooldown = 1001,
    NcnSlasherAdminInvalid = 1002,
    NcnVaultAdminInvalid = 1003,
    NcnAdminInvalid = 1004,
    NcnWithdrawAdminInvalid = 1005,
    NcnVaultSlasherTicketFailedCooldown = 1006,
    NcnVaultTicketFailedCooldown = 1007,
    NcnOperatorTicketFailedWarmup = 1008,
    NcnVaultSlasherTicketFailedWarmup = 1009,
    NcnVaultTicketFailedWarmup = 1010,
    NcnVaultTicketNotActive = 1011,

    OperatorNcnAdminInvalid = 2000,
    OperatorVaultAdminInvalid = 2001,
    OperatorAdminInvalid = 2002,
    OperatorWithdrawAdminInvalid = 2003,
    OperatorNcnTicketFailedCooldown = 2004,
    OperatorVaultTicketFailedCooldown = 2005,
    OperatorNcnTicketFailedWarmup = 2006,
    OperatorVaultTicketFailedWarmup = 2007,
    OperatorNcnTicketNotActive = 2008,
}

impl From<RestakingError> for ProgramError {
    fn from(e: RestakingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<RestakingError> for u64 {
    fn from(e: RestakingError) -> Self {
        e as u64
    }
}
