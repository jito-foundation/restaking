use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{
    account_info::AccountInfo, clock::DEFAULT_SLOTS_PER_EPOCH, entrypoint::ProgramResult, msg,
    pubkey::Pubkey,
};

use crate::result::{RestakingCoreError, RestakingCoreResult};

impl Discriminator for Config {
    const DISCRIMINATOR: u8 = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Config {
    /// The configuration admin
    pub admin: Pubkey,

    /// The vault program
    pub vault_program: Pubkey,

    /// The number of NCN managed by the program
    pub ncn_count: u64,

    /// The number of operators managed by the program
    pub operator_count: u64,

    /// The length of an epoch in slots
    pub epoch_length: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved_1: [u8; 7],
}

impl Config {
    pub const fn new(admin: Pubkey, vault_program: Pubkey, bump: u8) -> Self {
        Self {
            admin,
            vault_program,
            epoch_length: DEFAULT_SLOTS_PER_EPOCH,
            ncn_count: 0,
            operator_count: 0,
            bump,
            reserved_1: [0; 7],
        }
    }

    pub fn increment_ncn(&mut self) -> RestakingCoreResult<()> {
        self.ncn_count = self
            .ncn_count
            .checked_add(1)
            .ok_or(RestakingCoreError::NcnOverflow)?;
        Ok(())
    }

    pub fn increment_operators(&mut self) -> RestakingCoreResult<()> {
        self.operator_count = self
            .operator_count
            .checked_add(1)
            .ok_or(RestakingCoreError::OperatorOverflow)?;
        Ok(())
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
        let config = Box::new(Config::new(Pubkey::default(), Pubkey::default(), 0));

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
        // borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.config)?;
        Ok(())
    }
}
