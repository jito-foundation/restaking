use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct Ncn {
    /// The account type
    account_type: AccountType,

    /// The base account used as a PDA seed
    base: Pubkey,

    /// The admin of the NCN
    admin: Pubkey,

    /// The operator admin of the NCN
    operator_admin: Pubkey,

    /// The vault admin of the NCN
    vault_admin: Pubkey,

    /// The slasher admin of the NCN
    slasher_admin: Pubkey,

    /// The withdraw admin of the NCN
    withdraw_admin: Pubkey,

    /// The index of the NCN
    index: u64,

    /// Number of operator accounts associated with the NCN
    operator_count: u64,

    /// Number of vault accounts associated with the NCN
    vault_count: u64,

    /// Number of slasher accounts associated with the NCN
    slasher_count: u64,

    /// Reserved space
    reserved: [u8; 128],

    /// The bump seed for the PDA
    bump: u8,
}

impl Ncn {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        base: Pubkey,
        admin: Pubkey,
        operator_admin: Pubkey,
        vault_admin: Pubkey,
        slasher_admin: Pubkey,
        withdraw_admin: Pubkey,
        ncn_index: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::Ncn,
            base,
            admin,
            operator_admin,
            vault_admin,
            slasher_admin,
            withdraw_admin,
            index: ncn_index,
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
            .ok_or(RestakingCoreError::NcnOperatorCountOverflow)?;
        Ok(())
    }

    pub const fn vault_count(&self) -> u64 {
        self.vault_count
    }

    pub fn increment_vault_count(&mut self) -> RestakingCoreResult<()> {
        self.vault_count = self
            .vault_count
            .checked_add(1)
            .ok_or(RestakingCoreError::NcnVaultCountOverflow)?;
        Ok(())
    }

    pub const fn slasher_count(&self) -> u64 {
        self.slasher_count
    }

    pub fn increment_slasher_count(&mut self) -> RestakingCoreResult<()> {
        self.slasher_count = self
            .slasher_count
            .checked_add(1)
            .ok_or(RestakingCoreError::NcnSlasherCountOverflow)?;
        Ok(())
    }

    pub fn set_admin(&mut self, admin: Pubkey) {
        self.admin = admin;
    }

    /// Check if the provided pubkey is the admin of the NCN
    pub fn check_admin(&self, admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.admin != *admin {
            return Err(RestakingCoreError::NcnInvalidAdmin);
        }
        Ok(())
    }

    pub fn set_operator_admin(&mut self, operator_admin: Pubkey) {
        self.operator_admin = operator_admin;
    }

    /// Check if the provided pubkey is the operator admin of the NCN
    pub fn check_operator_admin(&self, operator_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.operator_admin != *operator_admin {
            return Err(RestakingCoreError::NcnInvalidOperatorAdmin);
        }
        Ok(())
    }

    pub fn set_vault_admin(&mut self, vault_admin: Pubkey) {
        self.vault_admin = vault_admin;
    }

    /// Check if the provided pubkey is the vault admin of the NCN
    pub fn check_vault_admin(&self, vault_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.vault_admin != *vault_admin {
            return Err(RestakingCoreError::NcnInvalidVaultAdmin);
        }
        Ok(())
    }

    pub fn set_slasher_admin(&mut self, slasher_admin: Pubkey) {
        self.slasher_admin = slasher_admin;
    }

    /// Check if the provided pubkey is the slasher admin of the NCN
    pub fn check_slasher_admin(&self, slasher_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.slasher_admin != *slasher_admin {
            return Err(RestakingCoreError::NcnInvalidSlasherAdmin);
        }
        Ok(())
    }

    pub fn set_withdraw_admin(&mut self, withdraw_admin: Pubkey) {
        self.withdraw_admin = withdraw_admin;
    }

    /// Check if the provided pubkey is the withdraw admin of the NCN
    pub fn check_withdraw_admin(&self, withdraw_admin: &Pubkey) -> RestakingCoreResult<()> {
        if self.withdraw_admin != *withdraw_admin {
            return Err(RestakingCoreError::NcnInvalidWithdrawAdmin);
        }
        Ok(())
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"ncn".to_vec(), base.as_ref().to_vec()])
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
            return Err(RestakingCoreError::NcnEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::NcnInvalidOwner);
        }

        let ncn_state = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::NcnInvalidData(e.to_string()))?;
        if ncn_state.account_type != AccountType::Ncn {
            return Err(RestakingCoreError::NcnInvalidAccountType);
        }

        let mut seeds = Self::seeds(&ncn_state.base());
        seeds.push(vec![ncn_state.bump()]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::NcnInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::NcnInvalidPda);
        }
        Ok(ncn_state)
    }
}

pub struct SanitizedNcn<'a, 'info> {
    account: &'a AccountInfo<'info>,
    ncn: Box<Ncn>,
}

impl<'a, 'info> SanitizedNcn<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> RestakingCoreResult<SanitizedNcn<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::NcnNotWritable);
        }
        let ncn = Box::new(Ncn::deserialize_checked(program_id, account)?);

        Ok(SanitizedNcn { account, ncn })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn ncn(&self) -> &Ncn {
        &self.ncn
    }

    pub fn ncn_mut(&mut self) -> &mut Ncn {
        &mut self.ncn
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.ncn)?;
        Ok(())
    }
}
