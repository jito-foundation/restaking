use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct Operator {
    /// The account type
    account_type: AccountType,

    /// The base pubkey used as a seed for the PDA
    base: Pubkey,

    /// The admin pubkey
    admin: Pubkey,

    avs_admin: Pubkey,

    vault_admin: Pubkey,

    /// The voter pubkey
    voter: Pubkey,

    /// The operator index
    index: u64,

    avs_count: u64,

    vault_count: u64,

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
            avs_admin: admin,
            vault_admin: admin,
            voter,
            index,
            avs_count: 0,
            vault_count: 0,
            reserved_space: [0; 1024],
            bump,
        }
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn avs_count(&self) -> u64 {
        self.avs_count
    }

    pub fn increment_avs_count(&mut self) -> RestakingCoreResult<()> {
        self.avs_count = self
            .avs_count
            .checked_add(1)
            .ok_or(RestakingCoreError::OperatorAvsCountOverflow)?;
        Ok(())
    }

    pub const fn vault_count(&self) -> u64 {
        self.vault_count
    }

    pub fn increment_vault_count(&mut self) -> RestakingCoreResult<()> {
        self.vault_count = self
            .vault_count
            .checked_add(1)
            .ok_or(RestakingCoreError::OperatorVaultCountOverflow)?;
        Ok(())
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

    pub const fn avs_admin(&self) -> Pubkey {
        self.avs_admin
    }

    pub fn check_avs_admin(&self, avs_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.avs_admin != *avs_admin {
            return Err(RestakingCoreError::OperatorInvalidAvsAdmin);
        }
        Ok(())
    }

    pub fn set_avs_admin(&mut self, avs_admin: Pubkey) {
        self.avs_admin = avs_admin;
    }

    pub const fn vault_admin(&self) -> Pubkey {
        self.vault_admin
    }

    pub fn check_vault_admin(&self, vault_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.vault_admin != *vault_admin {
            return Err(RestakingCoreError::OperatorInvalidVaultAdmin);
        }
        Ok(())
    }

    pub fn set_vault_admin(&mut self, vault_admin: Pubkey) {
        self.vault_admin = vault_admin;
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
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::OperatorDataEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::OperatorInvalidOwner);
        }

        // The AvsState shall be properly deserialized and valid struct
        let operator = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::OperatorInvalidData(e.to_string()))?;
        if operator.account_type != AccountType::Operator {
            return Err(RestakingCoreError::OperatorInvalidAccountType);
        }

        let mut seeds = Self::seeds(&operator.base);
        seeds.push(vec![operator.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::OperatorInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::OperatorInvalidPda);
        }

        Ok(operator)
    }
}

pub struct SanitizedOperator<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator: Box<Operator>,
}

impl<'a, 'info> SanitizedOperator<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::OperatorNotWritable);
        }

        let operator = Box::new(Operator::deserialize_checked(program_id, account)?);

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
