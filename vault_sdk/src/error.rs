use solana_program::program_error::ProgramError;

#[derive(Debug, PartialEq, Eq)]
pub enum VaultError {
    VaultSlashUnderflow = 1002,

    VaultMaxDelegationOverflow = 2000,
    VaultVrtEmpty = 2001,
    VaultInsufficientFunds = 2002,
    VaultAssetsReturnedOverflow = 2003,
    VaultOverflow = 2004,
    VaultOperatorAdminInvalid = 2005,
    VaultAdminInvalid = 2006,
    VaultCapacityAdminInvalid = 2007,
    VaultMintBurnAdminInvalid = 2008,
    VaultDelegationAdminInvalid = 2009,
    VaultCapacityExceeded = 2010,
    VaultSlasherAdminInvalid = 2011,
    VaultNcnAdminInvalid = 2012,
    VaultFeeAdminInvalid = 2013,
    VaultFeeCapExceeded = 2014,
    VaultFeeChangeTooSoon = 2015,
    VaultFeeBumpTooLarge = 2016,
    VaultUnderflow = 2017,
    VaultUpdateNeeded = 2018,
    VaultIsUpdated = 2019,
    VaultUpdateIncorrectIndex = 2020,
    VaultUpdateStateNotFinishedUpdating = 2021,
    VaultSecurityOverflow = 2022,
    VaultSlashIncomplete = 2023,
    VaultSecurityUnderflow = 2024,
    SlippageError = 2025,

    VaultDelegationListOverflow = 3000,
    VaultDelegationListUnderflow = 3001,
    VaultDelegationListInsufficientSecurity = 3002,
    VaultDelegationListFull = 3003,
    VaultDelegationListUndelegateIncomplete = 3004,
    VaultDelegationListOperatorNotFound = 3005,
    VaultDelegationListUpdateNeeded = 3006,

    VaultStakerWithdrawalTicketNotWithdrawable = 4000,
    VaultNcnSlasherTicketFailedCooldown = 4001,
    VaultNcnSlasherTicketFailedWarmup = 4002,
    VaultNcnTicketFailedCooldown = 4003,
    VaultNcnTicketFailedWarmup = 4004,
    VaultOperatorDelegationNotActive = 4005,
    VaultOperatorDelegationFailedCooldown = 4006,
    VaultOperatorDelegationFailedWarmup = 4007,
    NcnVaultSlasherTicketNotActive = 4008,
    NcnVaultTicketNotActive = 4009,
    OperatorVaultTicketNotActive = 4010,
    VaultNcnTicketUnslashable = 4011,
    OperatorVaultTicketUnslashable = 4012,
    VaultOperatorDelegationUnslashable = 4013,
    NcnOperatorStateUnslashable = 4014,
    OperatorNcnTicketUnslashable = 4015,
    VaultNcnSlasherTicketUnslashable = 4016,
    NcnVaultTicketUnslashable = 4017,
    NcnVaultSlasherTicketUnslashable = 4018,
    VaultMaxSlashedPerOperatorExceeded = 4019,

    VaultDelegationUpdateOverflow = 5000,
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
