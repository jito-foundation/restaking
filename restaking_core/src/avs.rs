use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
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
    index: u64,

    /// Number of operator accounts associated with the AVS
    operator_count: u64,

    /// Number of vault accounts associated with the AVS
    vault_count: u64,

    /// Number of slasher accounts associated with the AVS
    slasher_count: u64,

    /// Reserved space
    reserved: [u8; 128],

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
            index: avs_index,
            operator_count: 0,
            vault_count: 0,
            slasher_count: 0,
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub const fn operator_admin(&self) -> Pubkey {
        self.operator_admin
    }

    pub const fn vault_admin(&self) -> Pubkey {
        self.vault_admin
    }

    pub const fn slasher_admin(&self) -> Pubkey {
        self.slasher_admin
    }

    pub const fn withdraw_admin(&self) -> Pubkey {
        self.withdraw_admin
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub const fn operator_count(&self) -> u64 {
        self.operator_count
    }

    pub fn increment_operator_count(&mut self) -> RestakingCoreResult<()> {
        self.operator_count = self
            .operator_count
            .checked_add(1)
            .ok_or(RestakingCoreError::AvsOperatorCountOverflow)?;
        Ok(())
    }

    pub const fn vault_count(&self) -> u64 {
        self.vault_count
    }

    pub fn increment_vault_count(&mut self) -> RestakingCoreResult<()> {
        self.vault_count = self
            .vault_count
            .checked_add(1)
            .ok_or(RestakingCoreError::AvsVaultCountOverflow)?;
        Ok(())
    }

    pub const fn slasher_count(&self) -> u64 {
        self.slasher_count
    }

    pub fn increment_slasher_count(&mut self) -> RestakingCoreResult<()> {
        self.slasher_count = self
            .slasher_count
            .checked_add(1)
            .ok_or(RestakingCoreError::AvsSlasherCountOverflow)?;
        Ok(())
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
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::AvsEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::AvsInvalidOwner);
        }

        let avs_state = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::AvsInvalidData(e.to_string()))?;
        if avs_state.account_type != AccountType::Avs {
            return Err(RestakingCoreError::AvsInvalidAccountType);
        }

        let mut seeds = Self::seeds(&avs_state.base());
        seeds.push(vec![avs_state.bump()]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::AvsInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::AvsInvalidPda);
        }
        Ok(avs_state)
    }
}

pub struct SanitizedAvs<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs: Box<Avs>,
}

impl<'a, 'info> SanitizedAvs<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> RestakingCoreResult<SanitizedAvs<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::AvsNotWritable);
        }
        let avs = Box::new(Avs::deserialize_checked(program_id, account)?);

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
