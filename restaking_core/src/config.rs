use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::assert_with_msg;
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::AccountType;

pub const DEFAULT_RESTAKING_EPOCH_DURATION: u64 = 864_000;

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Config {
    /// The account type
    account_type: AccountType,

    /// The configuration admin
    admin: Pubkey,

    /// The vault program
    vault_program: Pubkey,

    /// The number of AVS managed by the program
    num_avs: u64,

    /// The number of operators managed by the program
    num_operators: u64,

    /// The duration of an epoch in slots
    epoch_duration: u64,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl Config {
    pub const fn new(admin: Pubkey, vault_program: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::Config,
            admin,
            vault_program,
            num_avs: 0,
            num_operators: 0,
            epoch_duration: DEFAULT_RESTAKING_EPOCH_DURATION,
            reserved: [0; 1024],
            bump,
        }
    }

    pub fn increment_avs(&mut self) -> Option<u64> {
        self.num_avs = self.num_avs.checked_add(1)?;
        Some(self.num_avs)
    }

    pub const fn avs_count(&self) -> u64 {
        self.num_avs
    }

    pub fn increment_operators(&mut self) -> Option<u64> {
        self.num_operators = self.num_operators.checked_add(1)?;
        Some(self.num_operators)
    }

    pub const fn operators_count(&self) -> u64 {
        self.num_operators
    }

    pub const fn epoch_duration(&self) -> u64 {
        self.epoch_duration
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
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "Config account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Config account not owned by the correct program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let config = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            config.account_type == AccountType::Config,
            ProgramError::InvalidAccountData,
            "AVS account is invalid",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds();
        seeds.push(vec![config.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Config account is not at the correct PDA",
        )?;

        Ok(config)
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
    ) -> Result<SanitizedConfig<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for Config",
            )?;
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
