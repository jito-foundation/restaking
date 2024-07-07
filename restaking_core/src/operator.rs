use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::assert_with_msg;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Operator {
    /// The account type
    account_type: AccountType,

    /// The base pubkey used as a seed for the PDA
    base: Pubkey,

    /// The admin pubkey
    admin: Pubkey,

    /// The voter pubkey
    voter: Pubkey,

    /// The operator index
    index: u64,

    /// Reserved space
    reserved_space: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl Operator {
    pub const fn new(base: Pubkey, admin: Pubkey, voter: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::Operator,
            base,
            admin,
            voter,
            index,
            reserved_space: [0; 1024],
            bump,
        }
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub fn check_admin(&self, admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.admin != *admin {
            return Err(RestakingCoreError::OperatorInvalidAdmin);
        }
        Ok(())
    }

    pub fn set_admin(&mut self, admin: Pubkey) {
        self.admin = admin;
    }

    pub const fn voter(&self) -> Pubkey {
        self.voter
    }

    pub fn set_voter(&mut self, voter: Pubkey) {
        self.voter = voter;
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"operator".to_vec(), base.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
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
            "Operator account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Operator account is not owned by the program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let operator = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            operator.account_type == AccountType::Operator,
            ProgramError::InvalidAccountData,
            "Operator account is not valid",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(&operator.base);
        seeds.push(vec![operator.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Operator account is not at the correct PDA",
        )?;

        Ok(operator)
    }
}

pub struct SanitizedOperator<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator: Operator,
}

impl<'a, 'info> SanitizedOperator<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> Result<Self, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Operator account is not writable",
            )?;
        }

        let operator = Operator::deserialize_checked(program_id, account)?;

        Ok(Self { account, operator })
    }

    pub const fn account(&self) -> &'a AccountInfo<'info> {
        self.account
    }

    pub const fn operator(&self) -> &Operator {
        &self.operator
    }

    pub fn operator_mut(&mut self) -> &mut Operator {
        &mut self.operator
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.operator)?;
        Ok(())
    }
}
