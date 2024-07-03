use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    vault::RestakingVault,
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

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct OperatorAvs {
    /// The AVS account
    avs: Pubkey,

    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 256],
}

impl OperatorAvs {
    pub const fn new(avs: Pubkey, slot_added: u64) -> Self {
        Self {
            avs,
            state: SlotToggle::new(slot_added),
            reserved: [0; 256],
        }
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct OperatorAvsList {
    account_type: AccountType,

    operator: Pubkey,

    bump: u8,

    avs: Vec<OperatorAvs>,
}

impl OperatorAvsList {
    pub const fn new(operator: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::OperatorAvsList,
            operator,
            bump,
            avs: vec![],
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub fn avs_list(&self) -> &[OperatorAvs] {
        &self.avs
    }

    pub fn add_avs(&mut self, avs: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_avs = self.avs.iter_mut().find(|a| a.avs() == avs);
        if let Some(avs) = maybe_avs {
            let activated = avs.state.activate(slot);
            if activated {
                Ok(())
            } else {
                Err(RestakingCoreError::AvsFailedToActivate)
            }
        } else {
            self.avs.push(OperatorAvs::new(avs, slot));
            Ok(())
        }
    }

    pub fn remove_avs(&mut self, avs: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_avs = self.avs.iter_mut().find(|a| a.avs() == avs);
        if let Some(avs) = maybe_avs {
            let deactivated = avs.state.deactivate(slot);
            if deactivated {
                Ok(())
            } else {
                Err(RestakingCoreError::AvsFailedToDeactivate)
            }
        } else {
            Err(RestakingCoreError::AvsNotFound)
        }
    }

    pub fn check_avs_active(&self, avs: &Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_avs = self.avs.iter().find(|a| a.avs() == *avs);
        maybe_avs.map_or(Err(RestakingCoreError::AvsNotFound), |avs| {
            if avs.state.is_active(slot) {
                Ok(())
            } else {
                Err(RestakingCoreError::AvsNotActive)
            }
        })
    }

    pub fn seeds(operator: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"operator_avs_list".to_vec(), operator.to_bytes().to_vec()]
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "Operator AVS List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Operator AVS List account is not owned by the program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let operator_avs_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            operator_avs_list.account_type == AccountType::OperatorAvsList,
            ProgramError::InvalidAccountData,
            "Operator AVS List account is not valid",
        )?;
        assert_with_msg(
            operator_avs_list.operator == *operator,
            ProgramError::InvalidAccountData,
            "Operator AVS List account is not for the correct operator",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(operator);
        seeds.push(vec![operator_avs_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Operator AVS List account is not at the correct PDA",
        )?;

        Ok(operator_avs_list)
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct OperatorVaultList {
    account_type: AccountType,

    operator: Pubkey,

    bump: u8,

    vaults: Vec<RestakingVault>,
}

impl OperatorVaultList {
    pub const fn new(operator: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::OperatorVaultList,
            operator,
            bump,
            vaults: vec![],
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub fn vault_list(&self) -> &[RestakingVault] {
        &self.vaults
    }

    pub fn check_active_vault(&self, vault: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vaults.iter().find(|v| v.vault() == vault);
        maybe_vault.map_or(Err(RestakingCoreError::VaultNotFound), |vault| {
            if vault.state().is_active(slot) {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultNotActive)
            }
        })
    }

    pub fn add_vault(&mut self, vault: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vaults.iter_mut().find(|v| v.vault() == vault);
        if let Some(vault) = maybe_vault {
            let activated = vault.state_mut().activate(slot);
            if activated {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultFailedToActivate)
            }
        } else {
            self.vaults.push(RestakingVault::new(vault, slot));
            Ok(())
        }
    }

    pub fn remove_vault(&mut self, vault: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vaults.iter_mut().find(|v| v.vault() == vault);
        maybe_vault.map_or(Err(RestakingCoreError::VaultNotFound), |vault| {
            let deactivated = vault.state_mut().deactivate(slot);
            if deactivated {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultFailedToDeactivate)
            }
        })
    }

    pub fn seeds(operator: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            b"operator_vault_list".to_vec(),
            operator.to_bytes().to_vec(),
        ]
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "Operator Vault List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Operator Vault List account is not owned by the program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let operator_vault_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            operator_vault_list.account_type == AccountType::OperatorVaultList,
            ProgramError::InvalidAccountData,
            "Operator Vault List account is not valid",
        )?;
        assert_with_msg(
            operator_vault_list.operator == *operator,
            ProgramError::InvalidAccountData,
            "Operator Vault List account is not for the correct operator",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(operator);
        seeds.push(vec![operator_vault_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Operator Vault List account is not at the correct PDA",
        )?;

        Ok(operator_vault_list)
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

pub struct SanitizedOperatorAvsList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator_avs_list: OperatorAvsList,
}

impl<'a, 'info> SanitizedOperatorAvsList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Operator AVS List account is not writable",
            )?;
        }

        let operator_avs_list =
            OperatorAvsList::deserialize_checked(program_id, account, operator)?;

        Ok(Self {
            account,
            operator_avs_list,
        })
    }

    pub const fn account(&self) -> &'a AccountInfo<'info> {
        self.account
    }

    pub const fn operator_avs_list(&self) -> &OperatorAvsList {
        &self.operator_avs_list
    }

    pub fn operator_avs_list_mut(&mut self) -> &mut OperatorAvsList {
        &mut self.operator_avs_list
    }

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.operator_avs_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.operator_avs_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}

pub struct SanitizedOperatorVaultList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator_vault_list: OperatorVaultList,
}

impl<'a, 'info> SanitizedOperatorVaultList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Operator Vault List account is not writable",
            )?;
        }

        let operator_vault_list =
            OperatorVaultList::deserialize_checked(program_id, account, operator)?;

        Ok(Self {
            account,
            operator_vault_list,
        })
    }

    pub const fn account(&self) -> &'a AccountInfo<'info> {
        self.account
    }

    pub const fn operator_vault_list(&self) -> &OperatorVaultList {
        &self.operator_vault_list
    }

    pub fn operator_vault_list_mut(&mut self) -> &mut OperatorVaultList {
        &mut self.operator_vault_list
    }

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.operator_vault_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.operator_vault_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
