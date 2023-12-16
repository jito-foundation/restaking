use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
};

use crate::AccountType;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultOperator {
    operator: Pubkey,

    slot_added: u64,
    slot_removed: u64,

    active_amount: u64,
    cooling_down_amount: u64,
}

impl VaultOperator {
    pub const fn new(operator: Pubkey, slot_added: u64) -> Self {
        Self {
            operator,
            slot_added,
            slot_removed: 0,
            active_amount: 0,
            cooling_down_amount: 0,
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn slot_added(&self) -> u64 {
        self.slot_added
    }

    pub const fn slot_removed(&self) -> u64 {
        self.slot_removed
    }

    pub const fn active_amount(&self) -> u64 {
        self.active_amount
    }

    pub const fn cooling_down_amount(&self) -> u64 {
        self.cooling_down_amount
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultOperatorList {
    account_type: AccountType,

    vault: Pubkey,

    operator_list: Vec<VaultOperator>,

    reserved: [u8; 256],

    bump: u8,
}

impl VaultOperatorList {
    pub const fn new(vault: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::VaultOperatorList,
            vault,
            operator_list: vec![],
            reserved: [0; 256],
            bump,
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub fn operator_list(&self) -> &[VaultOperator] {
        &self.operator_list
    }

    pub fn delegate(
        &mut self,
        operator: Pubkey,
        amount: u64,
        total_deposited: u64,
    ) -> Result<(), ProgramError> {
        // calculate the total delegations
        let total_delegations = self.total_delegation();
        assert_with_msg(
            total_delegations.is_some(),
            ProgramError::InvalidArgument,
            "Total delegation overflow",
        )?;

        // ensure the new deposit doesn't overallocate the vault
        let delegated_after_deposit = total_delegations.unwrap().checked_add(amount);
        assert_with_msg(
            delegated_after_deposit.is_some(),
            ProgramError::InvalidArgument,
            "Total delegation overflow",
        )?;
        assert_with_msg(
            delegated_after_deposit.unwrap() <= total_deposited,
            ProgramError::InvalidArgument,
            "overdelegated amount",
        )?;

        let operator = self
            .operator_list
            .iter_mut()
            .find(|d| d.operator == operator);
        assert_with_msg(
            operator.is_some(),
            ProgramError::InvalidArgument,
            "Operator not found",
        )?;
        let operator = operator.unwrap();

        operator.active_amount = operator.active_amount.checked_add(amount).ok_or_else(|| {
            msg!("Delegation overflow");
            ProgramError::InvalidArgument
        })?;

        Ok(())
    }

    pub fn undelegate(&mut self, operator: Pubkey, amount: u64) -> Result<(), ProgramError> {
        if let Some(operator) = self
            .operator_list
            .iter_mut()
            .find(|d| d.operator == operator)
        {
            operator.active_amount =
                operator.active_amount.checked_sub(amount).ok_or_else(|| {
                    msg!("Delegation underflow");
                    ProgramError::InvalidArgument
                })?;
            operator.cooling_down_amount = operator
                .cooling_down_amount
                .checked_add(amount)
                .ok_or_else(|| {
                    msg!("Delegation overflow");
                    ProgramError::InvalidArgument
                })?;
        } else {
            msg!("Delegation not found");
            return Err(ProgramError::InvalidArgument);
        }

        Ok(())
    }

    /// Returns the total active + cooling down delegations
    pub fn total_delegation(&self) -> Option<u64> {
        let mut total: u64 = 0;
        for operator in self.operator_list.iter() {
            total = total
                .checked_add(operator.active_amount)?
                .checked_add(operator.cooling_down_amount)?;
        }
        Some(total)
    }

    pub fn add_operator(&mut self, operator: Pubkey, slot: u64) -> bool {
        if let Some(operator) = self
            .operator_list
            .iter_mut()
            .find(|x| x.operator == operator)
        {
            operator.slot_added = slot;
        } else {
            self.operator_list.push(VaultOperator::new(operator, slot));
        }
        true
    }

    pub fn remove_operator(&mut self, operator: Pubkey, slot: u64) -> bool {
        if let Some(operator) = self
            .operator_list
            .iter_mut()
            .find(|x| x.operator == operator)
        {
            operator.slot_removed = slot;
            true
        } else {
            false
        }
    }

    pub fn seeds(vault: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            b"vault_supported_operators".to_vec(),
            vault.to_bytes().to_vec(),
        ]
    }

    pub fn find_program_address(program_id: &Pubkey, vault: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "VaultOperatorList account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "VaultOperatorList account not owned by the correct program",
        )?;

        let state = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            state.account_type == AccountType::VaultOperatorList,
            ProgramError::InvalidAccountData,
            "VaultOperatorList account is invalid",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(vault);
        seeds.push(vec![state.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "VaultOperatorList account is not at the correct PDA",
        )?;

        Ok(state)
    }
}

pub struct SanitizedVaultOperatorList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_operator_list: VaultOperatorList,
}

impl<'a, 'info> SanitizedVaultOperatorList<'a, 'info> {
    /// Sanitizes the AvsAccount so it can be used in a safe context
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
    ) -> Result<SanitizedVaultOperatorList<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for LRT",
            )?;
        }
        let vault_operator_list =
            VaultOperatorList::deserialize_checked(program_id, account, vault)?;

        Ok(SanitizedVaultOperatorList {
            account,
            vault_operator_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_operator_list(&self) -> &VaultOperatorList {
        &self.vault_operator_list
    }

    pub fn vault_operator_list_mut(&mut self) -> &mut VaultOperatorList {
        &mut self.vault_operator_list
    }

    pub fn save(
        &self,
        rent: &Rent,
        payer: &'a AccountInfo<'info>,
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
        let serialized = self.vault_operator_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
