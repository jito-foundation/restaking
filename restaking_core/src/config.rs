use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
#[repr(C)]
pub struct Config {
    /// The account type
    account_type: AccountType,

    /// The configuration admin
    admin: Pubkey,

    /// The vault program
    vault_program: Pubkey,

    /// The number of AVS managed by the program
    avs_count: u64,

    /// The number of operators managed by the program
    operator_count: u64,

    /// Reserved space
    reserved: [u8; 128],

    /// The bump seed for the PDA
    bump: u8,
}

impl Config {
    pub const fn new(admin: Pubkey, vault_program: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::Config,
            admin,
            vault_program,
            avs_count: 0,
            operator_count: 0,
            reserved: [0; 128],
            bump,
        }
    }

    pub fn increment_avs(&mut self) -> RestakingCoreResult<()> {
        self.avs_count = self
            .avs_count
            .checked_add(1)
            .ok_or(RestakingCoreError::AvsOverflow)?;
        Ok(())
    }

    pub const fn avs_count(&self) -> u64 {
        self.avs_count
    }

    pub fn increment_operators(&mut self) -> RestakingCoreResult<()> {
        self.operator_count = self
            .operator_count
            .checked_add(1)
            .ok_or(RestakingCoreError::OperatorOverflow)?;
        Ok(())
    }

    pub const fn operators_count(&self) -> u64 {
        self.operator_count
    }

    pub const fn vault_program(&self) -> Pubkey {
        self.vault_program
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub const fn bump(&self) -> u8 {
        self.bump
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
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::ConfigEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::ConfigInvalidOwner);
        }

        // The AvsState shall be properly deserialized and valid struct
        let config = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::ConfigInvalidData(e.to_string()))?;
        if config.account_type != AccountType::Config {
            return Err(RestakingCoreError::ConfigInvalidAccountType);
        }

        let mut seeds = Self::seeds();
        seeds.push(vec![config.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::ConfigInvalidPda)?;

        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::ConfigInvalidPda);
        }

        Ok(config)
    }
}

pub struct SanitizedConfig<'a, 'info> {
    account: &'a AccountInfo<'info>,
    config: Box<Config>,
}

impl<'a, 'info> SanitizedConfig<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> RestakingCoreResult<SanitizedConfig<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::ConfigNotWritable);
        }
        let config = Box::new(Config::deserialize_checked(program_id, account)?);

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
