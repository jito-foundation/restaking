use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::assert_with_msg;
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Avs {
    /// The account type
    account_type: AccountType,

    /// The base account used as a PDA seed
    base: Pubkey,

    /// The admin of the AVS
    admin: Pubkey,

    /// The operator admin of the AVS
    operator_admin: Pubkey,

    /// The vault admin of the AVS
    vault_admin: Pubkey,

    /// The slasher admin of the AVS
    slasher_admin: Pubkey,

    /// The withdraw admin of the AVS
    withdraw_admin: Pubkey,

    /// The index of the AVS
    avs_index: u64,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl Avs {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        base: Pubkey,
        admin: Pubkey,
        operator_admin: Pubkey,
        vault_admin: Pubkey,
        slasher_admin: Pubkey,
        withdraw_admin: Pubkey,
        avs_index: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::Avs,
            base,
            admin,
            operator_admin,
            vault_admin,
            slasher_admin,
            withdraw_admin,
            avs_index,
            reserved: [0; 1024],
            bump,
        }
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    pub const fn avs_index(&self) -> u64 {
        self.avs_index
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub fn set_admin(&mut self, admin: Pubkey) {
        self.admin = admin;
    }

    /// Check if the provided pubkey is the admin of the AVS
    pub fn check_admin(&self, admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.admin != *admin {
            return Err(RestakingCoreError::AvsInvalidAdmin);
        }
        Ok(())
    }

    pub const fn operator_admin(&self) -> Pubkey {
        self.operator_admin
    }

    pub fn set_operator_admin(&mut self, operator_admin: Pubkey) {
        self.operator_admin = operator_admin;
    }

    /// Check if the provided pubkey is the operator admin of the AVS
    pub fn check_operator_admin(&self, operator_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.operator_admin != *operator_admin {
            return Err(RestakingCoreError::AvsInvalidOperatorAdmin);
        }
        Ok(())
    }

    pub const fn vault_admin(&self) -> Pubkey {
        self.vault_admin
    }

    pub fn set_vault_admin(&mut self, vault_admin: Pubkey) {
        self.vault_admin = vault_admin;
    }

    /// Check if the provided pubkey is the vault admin of the AVS
    pub fn check_vault_admin(&self, vault_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.vault_admin != *vault_admin {
            return Err(RestakingCoreError::AvsInvalidVaultAdmin);
        }
        Ok(())
    }

    pub const fn slasher_admin(&self) -> Pubkey {
        self.slasher_admin
    }

    pub fn set_slasher_admin(&mut self, slasher_admin: Pubkey) {
        self.slasher_admin = slasher_admin;
    }

    /// Check if the provided pubkey is the slasher admin of the AVS
    pub fn check_slasher_admin(&self, slasher_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.slasher_admin != *slasher_admin {
            return Err(RestakingCoreError::AvsInvalidSlasherAdmin);
        }
        Ok(())
    }

    pub const fn withdraw_admin(&self) -> Pubkey {
        self.withdraw_admin
    }

    pub fn set_withdraw_admin(&mut self, withdraw_admin: Pubkey) {
        self.withdraw_admin = withdraw_admin;
    }

    /// Check if the provided pubkey is the withdraw admin of the AVS
    pub fn check_withdraw_admin(&self, withdraw_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.withdraw_admin != *withdraw_admin {
            return Err(RestakingCoreError::AvsInvalidWithdrawAdmin);
        }
        Ok(())
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
