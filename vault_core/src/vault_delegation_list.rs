use crate::{
    operator_delegation::OperatorDelegation,
    result::{VaultCoreError, VaultCoreResult},
};
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

pub enum UndelegateForWithdrawMethod {
    /// Withdraws from each operator's delegated amount in proportion to the total delegated amount
    ProRata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultDelegationUpdateSummary {
    NotUpdated,
    Updated { amount_reserved_for_withdraw: u64 },
}

const MAX_DELEGATIONS: usize = 128; // TODO (LB): make bigger

impl Discriminator for VaultDelegationList {
    const DISCRIMINATOR: u8 = 8;
}

/// Represents the operators which have opted-in to this vault
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultDelegationList {
    /// The vault this operator list is associated with
    vault: Pubkey,

    /// the list of delegations
    delegations: [OperatorDelegation; MAX_DELEGATIONS],

    /// The last slot the operator list was updated.
    /// Delegation information here is out of date if the last update epoch < current epoch
    last_slot_updated: u64,

    /// The bump seed for the PDA
    bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultDelegationList {
    pub fn new(vault: Pubkey, bump: u8) -> Self {
        Self {
            vault,
            delegations: [OperatorDelegation::default(); MAX_DELEGATIONS],
            last_slot_updated: 0,
            bump,
            reserved: [0; 7],
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

    /// Returns the total security in the delegation list
    pub fn total_security(&self) -> VaultCoreResult<u64> {
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
            msg!("Vault delegation list update required");
            Err(VaultCoreError::VaultDelegationListUpdateRequired)
        } else {
            Ok(())
        }
    }

    /// Updates the delegation list for the current epoch if needed.
    #[inline(always)]
    pub fn update(
        &mut self,
        slot: u64,
        epoch_length: u64,
    ) -> VaultCoreResult<VaultDelegationUpdateSummary> {
        let last_epoch_update = self.last_slot_updated.checked_div(epoch_length).unwrap();
        let current_epoch = slot.checked_div(epoch_length).unwrap();

        let mut amount_reserved_for_withdraw: u64 = 0;

        // time should only move forward, unwrap is safe
        let epoch_diff = current_epoch.checked_sub(last_epoch_update).unwrap();
        match epoch_diff {
            0 => return Ok(VaultDelegationUpdateSummary::NotUpdated),
            1 => {
                // enqueued -> cooling down, enqueued wiped
                for operator in self.delegations.iter_mut() {
                    amount_reserved_for_withdraw = amount_reserved_for_withdraw
                        .checked_add(operator.update())
                        .ok_or(VaultCoreError::VaultDelegationListUpdateOverflow)?;

                    if operator.is_empty() {
                        *operator = OperatorDelegation::default();
                    }
                }
            }
            _ => {
                // max updates required are two (enqueued -> cooling down -> wiped out),
                // so this only needs to be done twice even if >2 epochs have passed
                for operator in self.delegations.iter_mut() {
                    let amount_withdrawal_1 = operator.update();
                    let amount_withdrawal_2 = operator.update();

                    amount_reserved_for_withdraw = amount_reserved_for_withdraw
                        .checked_add(amount_withdrawal_1)
                        .and_then(|x| x.checked_add(amount_withdrawal_2))
                        .ok_or(VaultCoreError::VaultDelegationListUpdateOverflow)?;

                    if operator.is_empty() {
                        *operator = OperatorDelegation::default();
                    }
                }
            }
        }

        self.last_slot_updated = slot;

        Ok(VaultDelegationUpdateSummary::Updated {
            amount_reserved_for_withdraw,
        })
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
        max_delegation_amount: u64,
    ) -> VaultCoreResult<()> {
        let delegated_security = self.total_security()?;

        // Ensure the amount delegated doesn't exceed the total deposited
        let security_available_for_delegation = max_delegation_amount
            .checked_sub(delegated_security)
            .ok_or(VaultCoreError::VaultDelegationListInsufficientSecurity)?;

        if amount > security_available_for_delegation {
            msg!("Insufficient security available for delegation");
            return Err(VaultCoreError::VaultDelegationListInsufficientSecurity);
        }

        if let Some(operator) = self.delegations.iter_mut().find(|d| d.operator == operator) {
            operator.delegate(amount)?;
        } else {
            let mut delegation = OperatorDelegation::new(operator);
            delegation.delegate(amount)?;
            let mut first_spot = self
                .delegations
                .iter_mut()
                .find(|d| d.operator == Pubkey::default());
            if let Some(spot) = first_spot {
                *spot = delegation;
            } else {
                return Err(VaultCoreError::VaultDelegationListFull);
            }
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

        let mut total_undelegated: u64 = 0;

        for delegation in self.delegations.iter_mut() {
            let delegated_security = delegation.withdrawable_security()?;

            // Calculate un-delegate amount, rounding up
            let undelegate_amount = delegated_security
                .checked_mul(amount)
                .ok_or(VaultCoreError::ArithmeticOverflow)?
                .div_ceil(withdrawable_assets);

            if undelegate_amount > 0 {
                //  don't un-delegate too much
                let actual_undelegate =
                    std::cmp::min(undelegate_amount, amount.saturating_sub(total_undelegated));

                delegation.undelegate_for_withdraw(actual_undelegate)?;

                total_undelegated = total_undelegated
                    .checked_add(actual_undelegate)
                    .ok_or(VaultCoreError::ArithmeticOverflow)?;

                if total_undelegated == amount {
                    break;
                }
            }
        }

        if total_undelegated != amount {
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
        if let Some(operator) = self.delegations.iter_mut().find(|d| d.operator == operator) {
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
            .find(|operator_delegation| operator_delegation.operator == *operator)
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
        assert_eq!(list.total_security().unwrap(), 100);

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.operator, operator);
        assert_eq!(delegation.staked_amount, 100);
        assert_eq!(delegation.total_security().unwrap(), 100);
    }

    #[test]
    fn test_delegate_existing_operator() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();
        list.delegate(operator, 100, 1_000).unwrap();
        list.delegate(operator, 50, 1_000).unwrap();
        assert_eq!(list.total_security().unwrap(), 150);

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.operator, operator);
        assert_eq!(delegation.staked_amount, 150);
    }

    #[test]
    fn test_undelegate() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();

        list.delegate(operator, 100, 1_000).unwrap();
        list.undelegate(operator, 30).unwrap();

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount, 70);
        assert_eq!(delegation.enqueued_for_cooldown_amount, 30);
        assert_eq!(delegation.total_security().unwrap(), 100);
    }

    #[test]
    fn test_slash() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();

        list.delegate(operator, 100, 1_000).unwrap();
        list.slash(&operator, 20).unwrap();

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount, 80);
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
        assert_eq!(
            list.update(epoch_length, epoch_length).unwrap(),
            VaultDelegationUpdateSummary::Updated {
                amount_reserved_for_withdraw: 0
            }
        );

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount, 70);
        assert_eq!(delegation.cooling_down_amount, 30);
        assert_eq!(delegation.enqueued_for_cooldown_amount, 0);

        // Simulate passing of another epoch
        assert_eq!(
            list.update(epoch_length * 2, epoch_length).unwrap(),
            VaultDelegationUpdateSummary::Updated {
                amount_reserved_for_withdraw: 0
            }
        );

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount, 70);
        assert_eq!(delegation.cooling_down_amount, 0);
    }

    #[test]
    fn test_undelegate_for_withdraw_and_over_delegate() {
        let mut list = setup_vault_delegation_list();
        let operator = Pubkey::new_unique();

        list.delegate(operator, 500, 1000).unwrap();
        list.undelegate_for_withdrawal(200, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        assert_eq!(list.delegations().len(), 1);
        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount, 300);
        assert_eq!(delegation.enqueued_for_withdraw_amount, 200);

        assert_eq!(
            list.update(100, 100).unwrap(),
            VaultDelegationUpdateSummary::Updated {
                amount_reserved_for_withdraw: 0
            }
        );
        assert_eq!(
            list.update(200, 100).unwrap(),
            VaultDelegationUpdateSummary::Updated {
                amount_reserved_for_withdraw: 200
            }
        );
        assert_eq!(list.total_security().unwrap(), 300);

        assert_eq!(
            list.delegate(operator, 701, 1000),
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
        assert_eq!(delegation.staked_amount, 500);
        assert_eq!(delegation.enqueued_for_withdraw_amount, 500);
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

        let total_delegated_before_undelegation = list.total_security().unwrap();

        list.undelegate_for_withdrawal(600, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        assert_eq!(
            total_delegated_before_undelegation,
            list.total_security().unwrap()
        );

        // 3000 total staked, 600 withdrawn
        let delegations = list.delegations();
        assert_eq!(delegations[0].staked_amount, 800);
        assert_eq!(delegations[0].enqueued_for_withdraw_amount, 200);
        assert_eq!(delegations[1].staked_amount, 1200);
        assert_eq!(delegations[1].enqueued_for_withdraw_amount, 300);
        assert_eq!(delegations[2].staked_amount, 400);
        assert_eq!(delegations[2].enqueued_for_withdraw_amount, 100);
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
        assert_eq!(delegations[0].enqueued_for_withdraw_amount, 34);
        assert_eq!(delegations[1].enqueued_for_withdraw_amount, 34);
        assert_eq!(delegations[2].enqueued_for_withdraw_amount, 32);
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
        assert_eq!(delegation.staked_amount, 100);
        assert_eq!(delegation.enqueued_for_withdraw_amount, 0);
    }

    // ensures cooling down assets are handled correctly when undelegating
    #[test]
    fn test_undelegate_for_withdraw_with_enqueued_for_cooling_down_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 10_000, total_deposited).unwrap();

        let operator_2 = Pubkey::new_unique();
        list.delegate(operator_2, 60_000, total_deposited).unwrap();

        let operator_3 = Pubkey::new_unique();
        list.delegate(operator_3, 30_000, total_deposited).unwrap();

        list.undelegate(operator_2, 30_000).unwrap();

        assert_eq!(list.total_security().unwrap(), total_deposited);

        let delegation_1 = list.delegations().get(0).unwrap();
        assert_eq!(delegation_1.operator, operator_1);
        assert_eq!(delegation_1.withdrawable_security().unwrap(), 10_000);

        let delegation_2 = list.delegations().get(1).unwrap();
        assert_eq!(delegation_2.operator, operator_2);
        assert_eq!(delegation_2.withdrawable_security().unwrap(), 60_000);
        assert_eq!(delegation_2.staked_amount, 30_000);
        assert_eq!(delegation_2.enqueued_for_cooldown_amount, 30_000);

        let delegation_3 = list.delegations().get(2).unwrap();
        assert_eq!(delegation_3.operator, operator_3);
        assert_eq!(delegation_3.withdrawable_security().unwrap(), 30_000);

        list.undelegate_for_withdrawal(50_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        // 10% of assets staked -> 10% of withdraw
        let delegation_1 = list.delegations().get(0).unwrap();
        assert_eq!(delegation_1.operator, operator_1);
        assert_eq!(delegation_1.total_security().unwrap(), 10_000);
        assert_eq!(delegation_1.enqueued_for_withdraw_amount, 5_000);

        // 30k was staked, 30k was cooling down
        // 60% of assets staked -> 60% of withdraw -> 30,000 withdrawn
        let delegation_2 = list.delegations().get(1).unwrap();
        assert_eq!(delegation_2.total_security().unwrap(), 60_000);
        assert_eq!(delegation_2.operator, operator_2);
        assert_eq!(delegation_2.staked_amount, 0);
        assert_eq!(delegation_2.enqueued_for_withdraw_amount, 30_000);
        assert_eq!(delegation_2.enqueued_for_cooldown_amount, 30_000);

        // 30% of assets staked -> 30% of withdraw
        let delegation_3 = list.delegations().get(2).unwrap();
        assert_eq!(delegation_3.total_security().unwrap(), 30_000);
        assert_eq!(delegation_3.operator, operator_3);
        assert_eq!(delegation_3.enqueued_for_withdraw_amount, 15_000);
    }

    /// ensures that assets cooling down for withdraw are handled correctly when undelegating
    #[test]
    fn test_undelegate_for_withdraw_with_cooling_down_for_withdrawal_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 10_000, total_deposited).unwrap();

        let operator_2 = Pubkey::new_unique();
        list.delegate(operator_2, 60_000, total_deposited).unwrap();

        let operator_3 = Pubkey::new_unique();
        list.delegate(operator_3, 25_000, total_deposited).unwrap();

        list.undelegate_for_withdrawal(30_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        // 10000 * 30000 /_ceil 95000
        let delegation_1 = list.delegations.get(0).unwrap();
        assert_eq!(delegation_1.enqueued_for_withdraw_amount, 3_158);

        // 60000 * 30000 /_ceil 95000
        let delegation_2 = list.delegations.get(1).unwrap();
        assert_eq!(delegation_2.enqueued_for_withdraw_amount, 18_948);

        // min(25000 * 30000 /_ceil 95000, 30000 - 18948 - 3158)
        let delegation_3 = list.delegations.get(2).unwrap();
        assert_eq!(delegation_3.enqueued_for_withdraw_amount, 7_894);

        // send 5k more to operator 3
        list.delegate(operator_3, 5_000, total_deposited).unwrap();

        list.undelegate_for_withdrawal(20_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        // 3158 + ((10000 - 3158) * 20000 /_ceil 70000)
        let delegation_1 = list.delegations.get(0).unwrap();
        assert_eq!(delegation_1.enqueued_for_withdraw_amount, 5113);

        // 18948 + ((60000 - 18948) * 20000 /_ceil 70000)
        let delegation_2 = list.delegations.get(1).unwrap();
        assert_eq!(delegation_2.enqueued_for_withdraw_amount, 30678);

        // minimum of 7894 + ((30000 -  7894) * 20000 /_ceil 70000) and whatever is left to get to 50000 total
        // undelegated
        let delegation_3 = list.delegations.get(2).unwrap();
        assert_eq!(delegation_3.enqueued_for_withdraw_amount, 14209);
    }

    #[test]
    fn test_undelegate_for_withdraw_ceiling_ok() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 1, total_deposited).unwrap();

        let operator_2 = Pubkey::new_unique();
        list.delegate(operator_2, 99_999, total_deposited).unwrap();

        list.undelegate_for_withdrawal(99_999, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        // 1 * 99999 / 100000
        let delegation_1 = list.delegations().get(0).unwrap();
        assert_eq!(delegation_1.enqueued_for_withdraw_amount, 1);

        let delegation_2 = list.delegations().get(1).unwrap();
        assert_eq!(delegation_2.enqueued_for_withdraw_amount, 99_998);
    }

    #[test]
    fn test_slash_with_enqueued_for_cooldown_down_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 100_000, total_deposited).unwrap();
        list.undelegate(operator_1, 25_000).unwrap();

        list.slash(&operator_1, 10_000).unwrap();

        let delegation = list.delegations().get(0).unwrap();

        // 100k staked, 25k enqueued + 10k slashed
        // 25% of staked assets was enqueued for cooldown -> 25% of slashed funds -> 2500
        assert_eq!(delegation.staked_amount, 67500);
        assert_eq!(delegation.enqueued_for_cooldown_amount, 22500);
        assert_eq!(delegation.total_security().unwrap(), 90_000);
    }

    #[test]
    fn test_slash_with_cooling_down_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 100_000, total_deposited).unwrap();
        list.undelegate(operator_1, 25_000).unwrap();

        list.update(100, 100).unwrap();

        list.slash(&operator_1, 10_000).unwrap();

        let delegation = list.delegations().get(0).unwrap();

        // 100k staked, 25k cooldown + 10k slashed
        // 25% of staked assets were cooling down -> 25% of slashed funds -> 2500
        assert_eq!(delegation.staked_amount, 67500);
        assert_eq!(delegation.cooling_down_amount, 22500);
        assert_eq!(delegation.total_security().unwrap(), 90_000);
    }

    #[test]
    fn test_slash_with_enqueued_for_cooldown_and_cooling_down_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 100_000, total_deposited).unwrap();

        list.undelegate(operator_1, 12500).unwrap();
        list.update(100, 100).unwrap();
        list.undelegate(operator_1, 12500).unwrap();

        list.slash(&operator_1, 10_000).unwrap();

        let delegation = list.delegations().get(0).unwrap();

        assert_eq!(delegation.staked_amount, 67500);
        assert_eq!(delegation.cooling_down_amount, 11250);
        assert_eq!(delegation.enqueued_for_cooldown_amount, 11250);
        assert_eq!(delegation.total_security().unwrap(), 90_000);
    }

    #[test]
    fn test_slash_with_enqueued_for_withdraw_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 100_000, total_deposited).unwrap();
        list.undelegate_for_withdrawal(25_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        list.slash(&operator_1, 10_000).unwrap();

        let delegation = list.delegations().get(0).unwrap();

        assert_eq!(delegation.staked_amount, 67500);
        assert_eq!(delegation.enqueued_for_withdraw_amount, 22500);
        assert_eq!(delegation.total_security().unwrap(), 90_000);
    }

    #[test]
    fn test_slash_with_cooling_down_for_withdraw_assets() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 100_000, total_deposited).unwrap();

        list.undelegate_for_withdrawal(25_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();
        list.slash(&operator_1, 10_000).unwrap();
        list.update(100, 100).unwrap();

        let delegation = list.delegations().get(0).unwrap();

        assert_eq!(delegation.staked_amount, 67500);
        assert_eq!(delegation.cooling_down_for_withdraw_amount, 22500);
        assert_eq!(delegation.total_security().unwrap(), 90_000);
    }

    #[test]
    fn test_slash_with_enqueued_for_withdraw_and_cooling_down_for_withdraw() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 100_000, total_deposited).unwrap();

        list.undelegate_for_withdrawal(12500, UndelegateForWithdrawMethod::ProRata)
            .unwrap();
        list.update(100, 100).unwrap();
        list.undelegate_for_withdrawal(12500, UndelegateForWithdrawMethod::ProRata)
            .unwrap();

        list.slash(&operator_1, 10_000).unwrap();

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount, 67500);
        assert_eq!(delegation.cooling_down_for_withdraw_amount, 11250);
        assert_eq!(delegation.enqueued_for_withdraw_amount, 11250);
        assert_eq!(delegation.total_security().unwrap(), 90_000);
    }

    #[test]
    fn test_slash_with_withdraw_reserves() {
        let mut list = setup_vault_delegation_list();
        let total_deposited = 100_000;

        let operator_1 = Pubkey::new_unique();
        list.delegate(operator_1, 100_000, total_deposited).unwrap();

        list.undelegate_for_withdrawal(25_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();
        list.update(100, 100).unwrap();
        list.undelegate_for_withdrawal(25_000, UndelegateForWithdrawMethod::ProRata)
            .unwrap();
        assert_eq!(
            list.update(200, 100).unwrap(),
            VaultDelegationUpdateSummary::Updated {
                amount_reserved_for_withdraw: 25_000
            }
        );

        let delegation = list.delegations().get(0).unwrap();
        assert_eq!(delegation.staked_amount, 50000);
        assert_eq!(delegation.cooling_down_for_withdraw_amount, 25000);
        assert_eq!(delegation.total_security().unwrap(), 75000);

        list.slash(&operator_1, 10_000).unwrap();
        let delegation = list.delegations().get(0).unwrap();

        // 2/3 staked -> 2/3 of slashed
        assert_eq!(delegation.staked_amount, 43333);
        assert_eq!(delegation.cooling_down_for_withdraw_amount, 21667);
        assert_eq!(delegation.total_security().unwrap(), 65000);
    }
}
