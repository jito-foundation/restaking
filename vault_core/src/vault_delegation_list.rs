use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::realloc;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::{
    operator_delegation::OperatorDelegation,
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

pub enum UndelegateForWithdrawMethod {
    /// Withdraws from each operator's delegated amount in proportion to the total delegated amount
    ProRata,
}

/// Represents the operators which have opted-in to this vault
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultDelegationList {
    /// The account type
    account_type: AccountType,

    /// The vault this operator list is associated with
    vault: Pubkey,

    /// the list of delegations
    delegations: Vec<OperatorDelegation>,

    /// The reserve for withdrawable tokens
    withdrawable_reserve_amount: u64,

    /// The last slot the operator list was updated.
    /// Delegation information here is out of date if the last update epoch < current epoch
    last_slot_updated: u64,

    /// Reserved space
    reserved: [u8; 128],

    /// The bump seed for the PDA
    bump: u8,
}

impl VaultDelegationList {
    pub const fn new(vault: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::VaultDelegationList,
            vault,
            delegations: vec![],
            withdrawable_reserve_amount: 0,
            last_slot_updated: 0,
            reserved: [0; 128],
            bump,
        }
    }

    /// # Returns
    /// The vault pubkey
    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    /// # Returns
    /// The list of delegations
    pub fn delegations(&self) -> &[OperatorDelegation] {
        &self.delegations
    }

    pub const fn withdrawable_reserve_amount(&self) -> u64 {
        self.withdrawable_reserve_amount
    }

    pub fn decrement_withdrawable_reserve_amount(&mut self, amount: u64) -> VaultCoreResult<()> {
        self.withdrawable_reserve_amount = self
            .withdrawable_reserve_amount
            .checked_sub(amount)
            .ok_or(VaultCoreError::VaultDelegationListAmountWithdrawableUnderflow)?;
        Ok(())
    }

    /// Returns the total security in the delegation list
    pub fn all_security(&self) -> VaultCoreResult<u64> {
        let mut total: u64 = 0;
        for operator in self.delegations.iter() {
            total = total
                .checked_add(operator.total_security()?)
                .ok_or(VaultCoreError::VaultDelegationListTotalDelegationOverflow)?;
        }
        Ok(total)
    }

    /// The amount of security available for withdrawal from the delegation list. Includes
    /// staked and assets cooling down that aren't set aside for the withdrawal reserve
    pub fn withdrawable_security(&self) -> VaultCoreResult<u64> {
        let mut total: u64 = 0;
        for operator in self.delegations.iter() {
            total = total
                .checked_add(operator.withdrawable_security()?)
                .ok_or(VaultCoreError::VaultDelegationListTotalDelegationOverflow)?;
        }
        Ok(total)
    }

    /// Checks to see if the vault needs updating, which is defined as the epoch of the last update
    /// slot being less than the current epoch.
    ///
    /// # Returns
    /// true if the vault delegation list needs updating, false if not.
    #[inline(always)]
    pub fn is_update_needed(&self, slot: u64, epoch_length: u64) -> bool {
        let last_updated_epoch = self.last_slot_updated.checked_div(epoch_length).unwrap();
        let current_epoch = slot.checked_div(epoch_length).unwrap();
        last_updated_epoch < current_epoch
    }

    #[inline(always)]
    pub fn check_update_needed(&self, slot: u64, epoch_length: u64) -> VaultCoreResult<()> {
        if self.is_update_needed(slot, epoch_length) {
            Err(VaultCoreError::VaultDelegationListUpdateRequired)
        } else {
            Ok(())
        }
    }

    /// Updates the delegation list for the current epoch if needed.
    #[inline(always)]
    pub fn update(&mut self, slot: u64, epoch_length: u64) -> VaultCoreResult<bool> {
        let last_epoch_update = self.last_slot_updated.checked_div(epoch_length).unwrap();
        let current_epoch = slot.checked_div(epoch_length).unwrap();

        // time should only move forward, unwrap is safe
        let epoch_diff = current_epoch.checked_sub(last_epoch_update).unwrap();
        match epoch_diff {
            0 => return Ok(false),
            1 => {
                // enqueued -> cooling down, enqueued wiped
                for operator in self.delegations.iter_mut() {
                    self.withdrawable_reserve_amount = self
                        .withdrawable_reserve_amount
                        .checked_add(operator.update())
                        .ok_or(VaultCoreError::VaultDelegationListUpdateOverflow)?;
                }
            }
            _ => {
                // max updates required are two (enqueued -> cooling down -> wiped out),
                // so this only needs to be done twice even if >2 epochs have passed
                for operator in self.delegations.iter_mut() {
                    let amount_withdrawal_1 = operator.update();
                    let amount_withdrawal_2 = operator.update();

                    self.withdrawable_reserve_amount = self
                        .withdrawable_reserve_amount
                        .checked_add(amount_withdrawal_1)
                        .and_then(|x| x.checked_add(amount_withdrawal_2))
                        .ok_or(VaultCoreError::VaultDelegationListUpdateOverflow)?;
                }
            }
        }
        Ok(true)
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
    ) -> VaultCoreResult<()> {
        let delegated_security = self.all_security()?;

        // Ensure the amount delegated doesn't exceed the total deposited
        let security_available_for_delegation = total_deposited
            .checked_sub(delegated_security)
            .and_then(|x| x.checked_sub(self.withdrawable_reserve_amount))
            .ok_or(VaultCoreError::VaultDelegationListInsufficientSecurity)?;

        if amount > security_available_for_delegation {
            msg!("Insufficient security available for delegation");
            return Err(VaultCoreError::VaultDelegationListInsufficientSecurity);
        }

        if let Some(operator) = self
            .delegations
            .iter_mut()
            .find(|d| d.operator() == operator)
        {
            operator.delegate(amount)?;
        } else {
            let mut delegation = OperatorDelegation::new(operator);
            delegation.delegate(amount)?;
            self.delegations.push(delegation);
        }

        Ok(())
    }

    /// Undelegates an amount of stake from the vault for withdrawal
    pub fn undelegate_for_withdrawal(
        &mut self,
        amount: u64,
        method: UndelegateForWithdrawMethod,
    ) -> VaultCoreResult<()> {
        match method {
            UndelegateForWithdrawMethod::ProRata => self.undelegate_for_withdrawal_pro_rata(amount),
        }
    }

    /// Un-delegates `amount` staked assets from all the operators pro-rata based on the withdrawable
    /// security on each one.
    fn undelegate_for_withdrawal_pro_rata(&mut self, amount: u64) -> VaultCoreResult<()> {
        let withdrawable_assets = self.withdrawable_security()?;

        if amount > withdrawable_assets || withdrawable_assets == 0 {
            return Err(VaultCoreError::WithdrawAmountExceedsDelegatedFunds);
        }

        let mut remaining_to_undelegate = amount;

        for delegation in self.delegations.iter_mut() {
            // TODO (LB): instead of pro-rata for all stake, should be pro-rata for withdrawable stake
            let delegated_security = delegation.withdrawable_security()?;
            let undelegate_amount = (delegated_security as u128)
                .checked_mul(amount as u128)
                .and_then(|product| product.checked_div(withdrawable_assets as u128))
                .and_then(|result| result.try_into().ok())
                .ok_or(VaultCoreError::ArithmeticOverflow)?;

            if undelegate_amount > 0 {
                delegation.undelegate_for_withdraw(undelegate_amount)?;
                remaining_to_undelegate = remaining_to_undelegate
                    .checked_sub(undelegate_amount)
                    .ok_or(VaultCoreError::ArithmeticUnderflow)?;
            }
        }

        // Handle any remaining dust due to rounding
        if remaining_to_undelegate > 0 {
            for delegation in self.delegations.iter_mut() {
                if delegation.staked_amount() >= remaining_to_undelegate {
                    delegation.undelegate_for_withdraw(remaining_to_undelegate)?;
                    remaining_to_undelegate = 0;
                    break;
                }
            }
        }

        if remaining_to_undelegate > 0 {
            return Err(VaultCoreError::UndelegationIncomplete);
        }

        Ok(())
    }

    /// Undelegates an amount of stake from an operator
    ///
    /// # Arguments
    /// * `operator` - The operator pubkey to undelegate from
    /// * `amount` - The amount of stake to undelegate
    pub fn undelegate(&mut self, operator: Pubkey, amount: u64) -> Result<(), ProgramError> {
        if let Some(operator) = self
            .delegations
            .iter_mut()
            .find(|d| d.operator() == operator)
        {
            operator.undelegate(amount)?;
        } else {
            msg!("Delegation not found");
            return Err(ProgramError::InvalidArgument);
        }

        Ok(())
    }

    pub fn slash(&mut self, operator: &Pubkey, slash_amount: u64) -> VaultCoreResult<()> {
        self.delegations
            .iter_mut()
            .find(|x| x.operator() == *operator)
            .ok_or(VaultCoreError::VaultOperatorNotFound)?
            .slash(slash_amount)
    }

    pub fn seeds(vault: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"vault_delegation_list".to_vec(), vault.to_bytes().to_vec()]
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
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultDelegationListDataEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultDelegationListInvalidProgramOwner);
        }

        let state = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| VaultCoreError::VaultDelegationListInvalidData(e.to_string()))?;
        if state.account_type != AccountType::VaultDelegationList {
            return Err(VaultCoreError::VaultDelegationListInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault);
        seeds.push(vec![state.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultDelegationListInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultDelegationListInvalidPda);
        }

        Ok(state)
    }
}

pub struct SanitizedVaultDelegationList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_delegation_list: Box<VaultDelegationList>,
}

impl<'a, 'info> SanitizedVaultDelegationList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
    ) -> VaultCoreResult<SanitizedVaultDelegationList<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultDelegationListExpectedWritable);
        }
        let vault_delegation_list = Box::new(VaultDelegationList::deserialize_checked(
            program_id, account, vault,
        )?);

        Ok(SanitizedVaultDelegationList {
            account,
            vault_delegation_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_delegation_list(&self) -> &VaultDelegationList {
        &self.vault_delegation_list
    }

    pub fn vault_delegation_list_mut(&mut self) -> &mut VaultDelegationList {
        &mut self.vault_delegation_list
    }

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.vault_delegation_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.vault_delegation_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use super::*;

    fn setup_vault_delegation_list() -> VaultDelegationList {
        VaultDelegationList::new(Pubkey::new_unique(), 255)
    }

    #[test]
    fn test_delegate_new_operator() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();

        assert!(list.delegate(operator, 100, 1_000).is_ok());
        assert_eq!(list.all_security().unwrap(), 100);

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.operator(), operator);
        assert_eq!(delegation.staked_amount(), 100);
        assert_eq!(delegation.total_security().unwrap(), 100);
    }

    #[test]
    fn test_delegate_existing_operator() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();
        list.delegate(operator, 100, 1_000).unwrap();
        list.delegate(operator, 50, 1_000).unwrap();
        assert_eq!(list.all_security().unwrap(), 150);

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.operator(), operator);
        assert_eq!(delegation.staked_amount(), 150);
    }

    #[test]
    fn test_undelegate() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();

        list.delegate(operator, 100, 1_000).unwrap();
        list.undelegate(operator, 30).unwrap();

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount(), 70);
        assert_eq!(delegation.enqueued_for_cooldown_amount(), 30);
        assert_eq!(delegation.total_security().unwrap(), 100);
    }

    #[test]
    fn test_slash() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();

        list.delegate(operator, 100, 1_000).unwrap();
        list.slash(&operator, 20).unwrap();

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount(), 80);
        assert_eq!(delegation.total_security().unwrap(), 80);
    }

    #[test]
    fn test_update() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();
        let amount = 100;
        let total_deposited = 1000;
        let epoch_length = 100;

        list.delegate(operator, amount, total_deposited).unwrap();
        list.undelegate(operator, 30).unwrap();

        // Simulate passing of one epoch
        assert!(list.update(epoch_length, epoch_length).unwrap());

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount(), 70);
        assert_eq!(delegation.cooling_down_amount(), 30);
        assert_eq!(delegation.enqueued_for_cooldown_amount(), 0);

        // Simulate passing of another epoch
        assert!(list.update(epoch_length * 2, epoch_length).unwrap());

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount(), 70);
        assert_eq!(delegation.cooling_down_amount(), 0);
        assert_eq!(list.withdrawable_reserve_amount(), 0);
    }

    #[test]
    fn test_undelegate_for_withdraw_and_over_delegate() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();
        let total_deposited = 1000;
        let initial_delegation = 500;
        let undelegate_amount = 200;
        let over_delegation_attempt = 600;

        list.delegate(operator, initial_delegation, total_deposited)
            .unwrap();
        list.undelegate_for_withdrawal(undelegate_amount, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(
            delegation.staked_amount(),
            initial_delegation - undelegate_amount
        );
        assert_eq!(delegation.enqueued_for_withdraw_amount(), undelegate_amount);
        assert_eq!(list.withdrawable_reserve_amount(), 0);

        assert!(list.update(100, 100).unwrap());
        assert!(list.update(200, 100).unwrap());
        assert_eq!(list.withdrawable_reserve_amount(), undelegate_amount);
        assert_eq!(
            list.all_security().unwrap(),
            initial_delegation - undelegate_amount
        );

        // 1000 total deposits, 300 delegated, 200 enqueued for withdraw
        // if try to delegate 600, should fail because some assets are set aside for withdraw
        assert_eq!(
            list.delegate(operator, over_delegation_attempt, total_deposited),
            Err(VaultCoreError::VaultDelegationListInsufficientSecurity)
        );
    }

    #[test]
    fn test_undelegate_for_withdraw_pro_rata_single_operator() {
        let mut list = VaultDelegationList::new(Pubkey::new_unique(), 255);
        let operator = Pubkey::new_unique();
        list.delegate(operator, 1000, 1000).unwrap();

        list.undelegate_for_withdrawal(500, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount(), 500);
        assert_eq!(delegation.enqueued_for_withdraw_amount(), 500);
    }

    #[test]
    fn test_undelegate_for_withdraw_pro_rata_multiple_operators() {
        let mut list = setup_vault_delegation_list();
        let operator1 = Pubkey::new_unique();
        let operator2 = Pubkey::new_unique();
        let operator3 = Pubkey::new_unique();

        list.delegate(operator1, 1000, 3000).unwrap();
        list.delegate(operator2, 1500, 3000).unwrap();
        list.delegate(operator3, 500, 3000).unwrap();

        let total_delegated_before_undelegation = list.all_security().unwrap();

        list.undelegate_for_withdrawal(600, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        assert_eq!(
            total_delegated_before_undelegation,
            list.all_security().unwrap()
        );

        // 3000 total staked, 600 withdrawn
        let delegations = list.delegations();
        assert_eq!(delegations[0].staked_amount(), 800);
        assert_eq!(delegations[0].enqueued_for_withdraw_amount(), 200);
        assert_eq!(delegations[1].staked_amount(), 1200);
        assert_eq!(delegations[1].enqueued_for_withdraw_amount(), 300);
        assert_eq!(delegations[2].staked_amount(), 400);
        assert_eq!(delegations[2].enqueued_for_withdraw_amount(), 100);
    }

    #[test]
    fn test_undelegate_for_withdraw_pro_rata_rounding() {
        let mut list = setup_vault_delegation_list();
        let operator1 = Pubkey::new_unique();
        let operator2 = Pubkey::new_unique();
        let operator3 = Pubkey::new_unique();

        list.delegate(operator1, 100, 301).unwrap();
        list.delegate(operator2, 100, 301).unwrap();
        list.delegate(operator3, 101, 301).unwrap();

        list.undelegate_for_withdrawal(100, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        let delegations = list.delegations();
        assert_eq!(delegations[0].enqueued_for_withdraw_amount(), 34);
        assert_eq!(delegations[1].enqueued_for_withdraw_amount(), 33);
        assert_eq!(delegations[2].enqueued_for_withdraw_amount(), 33);
    }

    #[test]
    fn test_undelegate_for_withdraw_pro_rata_insufficient_funds() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();
        list.delegate(operator, 100, 100).unwrap();

        let result = list.undelegate_for_withdrawal(101, UndelegateForWithdrawMethod::ProRata);
        assert!(matches!(
            result,
            Err(VaultCoreError::WithdrawAmountExceedsDelegatedFunds)
        ));
    }

    #[test]
    fn test_undelegate_for_withdraw_pro_rata_no_delegations() {
        let mut list = setup_vault_delegation_list();

        let result = list.undelegate_for_withdrawal(100, UndelegateForWithdrawMethod::ProRata);
        assert!(matches!(
            result,
            Err(VaultCoreError::WithdrawAmountExceedsDelegatedFunds)
        ));
    }

    #[test]
    fn test_undelegate_for_withdraw_pro_rata_zero_amount() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();
        list.delegate(operator, 100, 100).unwrap();

        list.undelegate_for_withdrawal(0, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount(), 100);
        assert_eq!(delegation.enqueued_for_withdraw_amount(), 0);
    }

    // ensures cooling down assets are handled correctly when undelegating
    #[test]
    fn test_undelegate_for_withdraw_with_cooling_down_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 10_000, total_deposited).unwrap();

        let operator_2 = Pubkey::new_unique();
        list.delegate(operator_2, 60_000, total_deposited).unwrap();

        let operator_3 = Pubkey::new_unique();
        list.delegate(operator_3, 30_000, total_deposited).unwrap();

        list.undelegate(operator_2, 30_000).unwrap();

        assert_eq!(list.all_security().unwrap(), total_deposited);

        let delegation_1 = list.delegations().get(0).unwrap();
        assert_eq!(delegation_1.operator(), operator_1);
        assert_eq!(delegation_1.withdrawable_security().unwrap(), 10_000);

        let delegation_2 = list.delegations().get(1).unwrap();
        assert_eq!(delegation_2.operator(), operator_2);
        assert_eq!(delegation_2.withdrawable_security().unwrap(), 60_000);
        assert_eq!(delegation_2.staked_amount(), 30_000);
        assert_eq!(delegation_2.enqueued_for_cooldown_amount(), 30_000);

        let delegation_3 = list.delegations().get(2).unwrap();
        assert_eq!(delegation_3.operator(), operator_3);
        assert_eq!(delegation_3.withdrawable_security().unwrap(), 30_000);

        list.undelegate_for_withdrawal(50_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        // 10% of assets staked -> 10% of withdraw
        let delegation_1 = list.delegations().get(0).unwrap();
        assert_eq!(delegation_1.operator(), operator_1);
        assert_eq!(delegation_1.total_security().unwrap(), 10_000);
        assert_eq!(delegation_1.enqueued_for_withdraw_amount(), 5_000);

        // 30k was staked, 30k was cooling down
        // 60% of assets staked -> 60% of withdraw -> 30,000 withdrawn
        let delegation_2 = list.delegations().get(1).unwrap();
        assert_eq!(delegation_2.total_security().unwrap(), 60_000);
        assert_eq!(delegation_2.operator(), operator_2);
        assert_eq!(delegation_2.staked_amount(), 0);
        assert_eq!(delegation_2.enqueued_for_withdraw_amount(), 30_000);
        assert_eq!(delegation_2.enqueued_for_cooldown_amount(), 30_000);

        // 30% of assets staked -> 30% of withdraw
        let delegation_3 = list.delegations().get(2).unwrap();
        assert_eq!(delegation_3.total_security().unwrap(), 30_000);
        assert_eq!(delegation_3.operator(), operator_3);
        assert_eq!(delegation_3.enqueued_for_withdraw_amount(), 15_000);
    }

    #[test]
    fn test_undelegate_for_withdraw_pull_from_enqueued_for_cooling_down() {}

    /// ensures that assets cooling down for withdraw are handled correctly when undelegating
    #[test]
    fn test_undelegate_for_withdraw_with_cooling_down_for_withdrawal_assets() {}
}
