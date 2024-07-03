use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, clock::DEFAULT_SLOTS_PER_EPOCH,
    entrypoint_deprecated::ProgramResult, pubkey::Pubkey,
};
use VaultCoreError::ConfigInvalidPda;

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

pub const MAX_RESTAKING_PROGRAMS: usize = 8;

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Config {
    /// The account type
    account_type: AccountType,

    /// The configuration admin
    admin: Pubkey,

    /// The approved restaking program for this vault
    restaking_program: Pubkey,

    /// The length of an epoch in slots
    epoch_length: u64,

    /// The number of vaults managed by the program
    num_vaults: u64,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl Config {
    pub const fn new(admin: Pubkey, restaking_program: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::Config,
            admin,
            restaking_program,
            epoch_length: DEFAULT_SLOTS_PER_EPOCH,
            num_vaults: 0,
            reserved: [0; 1024],
            bump,
        }
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub const fn restaking_program(&self) -> Pubkey {
        self.restaking_program
    }

    pub const fn epoch_length(&self) -> u64 {
        self.epoch_length
    }

    pub fn increment_vaults(&mut self) -> Option<u64> {
        self.num_vaults = self.num_vaults.checked_add(1)?;
        Some(self.num_vaults)
    }

    pub const fn vaults_count(&self) -> u64 {
        self.num_vaults
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn is_struct_valid(&self) -> bool {
        self.account_type == AccountType::Config
    }

    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"config".to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds();
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::ConfigDataEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::ConfigInvalidProgramOwner);
        }

        let state = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| VaultCoreError::ConfigInvalidData(e.to_string()))?;
        if state.account_type != AccountType::Config {
            return Err(VaultCoreError::ConfigInvalidAccountType);
        }
        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds();
        seeds.push(vec![state.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| ConfigInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(ConfigInvalidPda);
        }

        Ok(state)
    }
}

pub struct SanitizedConfig<'a, 'info> {
    account: &'a AccountInfo<'info>,
    config: Config,
}

impl<'a, 'info> SanitizedConfig<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> VaultCoreResult<SanitizedConfig<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::ConfigExpectedWritable);
        }
        let config = Config::deserialize_checked(program_id, account)?;

        Ok(SanitizedConfig { account, config })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.config)?;
        Ok(())
    }
}
