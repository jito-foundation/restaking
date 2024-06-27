use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::assert_with_msg;
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::AccountType;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Avs {
    /// The account type
    account_type: AccountType,

    /// The base account used as a PDA seed
    base: Pubkey,

    /// The admin of the AVS
    admin: Pubkey,

    /// The index of the AVS
    avs_index: u64,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl Avs {
    pub const fn new(base: Pubkey, admin: Pubkey, avs_index: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::Avs,
            base,
            admin,
            avs_index,
            reserved: [0; 1024],
            bump,
        }
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub const fn avs_index(&self) -> u64 {
        self.avs_index
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"avs".to_vec(), base.as_ref().to_vec()])
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
            "AVS account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "AVS account not owned by the correct program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let avs_state = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            avs_state.account_type == AccountType::Avs,
            ProgramError::InvalidAccountData,
            "AVS account is invalid",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(&avs_state.base());
        seeds.push(vec![avs_state.bump()]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "AVS account is not at the correct PDA",
        )?;

        Ok(avs_state)
    }
}

pub struct SanitizedAvs<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs: Avs,
}

impl<'a, 'info> SanitizedAvs<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> Result<SanitizedAvs<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for AVS",
            )?;
        }
        let avs = Avs::deserialize_checked(program_id, account)?;

        Ok(SanitizedAvs { account, avs })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs(&self) -> &Avs {
        &self.avs
    }

    pub fn avs_mut(&mut self) -> &mut Avs {
        &mut self.avs
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.avs)?;
        Ok(())
    }
}
