use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::AccountType;

/// Represents an operator that has opted-in to the vault and any associated stake on this operator
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultOperator {
    /// The operator pubkey that has opted-in to the vault
    operator: Pubkey,

    /// The state of the operator being opted-in to the vault
    state: SlotToggle,

    /// The amount of stake that is currently active on the operator
    active_amount: u64,

    /// The amount of stake that is currently cooling down on the operator
    cooling_down_amount: u64,
}

impl VaultOperator {
    pub const fn new(operator: Pubkey, slot_added: u64) -> Self {
        Self {
            operator,
            state: SlotToggle::new(slot_added),
            active_amount: 0,
            cooling_down_amount: 0,
        }
    }

    /// # Returns
    /// The operator pubkey
    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    /// # Returns
    /// The state of the operator
    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    /// # Returns
    /// The active amount of stake on the operator
    pub const fn active_amount(&self) -> u64 {
        self.active_amount
    }

    /// # Returns
    /// The cooling down amount of stake on the operator
    pub const fn cooling_down_amount(&self) -> u64 {
        self.cooling_down_amount
    }
}

/// Represents the operators which have opted-in to this vault
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultOperatorList {
    /// The account type
    account_type: AccountType,

    /// The vault this operator list is associated with
    vault: Pubkey,

    /// The list of operators that have opted-in to this vault
    operator_list: Vec<VaultOperator>,

    /// The last slot the operator list was updated.
    /// Delegation information here is out of date if the last update epoch < current epoch
    last_slot_updated: u64,

    /// Reserved space
    reserved: [u8; 256],

    /// The bump seed for the PDA
    bump: u8,
}

impl VaultOperatorList {
    pub const fn new(vault: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::VaultOperatorList,
            vault,
            operator_list: vec![],
            last_slot_updated: 0,
            reserved: [0; 256],
            bump,
        }
    }

    /// # Returns
    /// The vault pubkey
    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    /// # Returns
    /// The list of operators that have opted-in to this vault
    pub fn operator_list(&self) -> &[VaultOperator] {
        &self.operator_list
    }

    pub fn needs_update(&self, slot: u64, epoch_length: u64) -> bool {
        self.last_slot_updated.checked_div(epoch_length).unwrap()
            < slot.checked_div(epoch_length).unwrap()
    }

    pub fn update_delegations(&mut self, slot: u64, epoch_length: u64) -> bool {
        let last_epoch_update = self.last_slot_updated.checked_div(epoch_length).unwrap();
        let current_epoch = slot.checked_div(epoch_length).unwrap();

        if last_epoch_update < current_epoch {
            for operator in self.operator_list.iter_mut() {
                operator.cooling_down_amount = 0;
            }
            self.last_slot_updated = slot;
            true
        } else {
            false
        }
    }

    /// Delegates an amount of stake to an operator and ensures the amount delegated doesn't
    /// exceed the total deposited.
    ///
    /// # Arguments
    /// * `operator` - The operator pubkey to delegate to
    /// * `amount` - The amount of stake to delegate
    /// * `total_deposited` - The total amount of stake deposited in the vault
    ///
    /// # Returns
    /// Ok(()) if the delegation was successful, otherwise an error
    pub fn delegate(
        &mut self,
        operator: Pubkey,
        amount: u64,
        total_deposited: u64,
    ) -> Result<(), ProgramError> {
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

    /// Undelegates an amount of stake from an operator
    ///
    /// # Arguments
    /// * `operator` - The operator pubkey to undelegate from
    /// * `amount` - The amount of stake to undelegate
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
            operator.state.activate(slot)
        } else {
            self.operator_list.push(VaultOperator::new(operator, slot));
            true
        }
    }

    /// TODO (LB): should stake be deactivated when the operator is removed?
    pub fn remove_operator(&mut self, operator: Pubkey, slot: u64) -> bool {
        if let Some(operator) = self
            .operator_list
            .iter_mut()
            .find(|x| x.operator == operator)
        {
            operator.state.deactivate(slot)
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
                "Invalid writable flag for vault operator list",
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

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.vault_operator_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.vault_operator_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
