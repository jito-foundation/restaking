use std::cmp::min;

use bytemuck::{Pod, Zeroable};
use jito_vault_sdk::error::VaultError;
use solana_program::{msg, pubkey::Pubkey};

/// Represents an operator that has opted-in to the vault and any associated stake on this operator
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct OperatorDelegation {
    /// The operator pubkey that has opted-in to the vault
    pub operator: Pubkey,

    /// The amount of stake that is currently active on the operator
    pub staked_amount: u64,

    /// Any stake that was deactivated in the current epoch
    pub enqueued_for_cooldown_amount: u64,

    /// Any stake that was deactivated in the previous epoch,
    /// to be available for re-delegation in the current epoch + 1
    pub cooling_down_amount: u64,

    /// Any stake that was enqueued for withdraw in the current epoch.
    /// These funds are earmarked for withdraw and can't be redelegated once inactive.
    pub enqueued_for_withdraw_amount: u64,

    /// Any stake that was enqueued for withdraw in the previous epoch,
    /// to be available for withdrawal in the current epoch + 1
    pub cooling_down_for_withdraw_amount: u64,

    pub is_used: u8,

    reserved: [u8; 7],
}

impl OperatorDelegation {
    pub const fn new(operator: Pubkey) -> Self {
        Self {
            operator,
            staked_amount: 0,
            enqueued_for_cooldown_amount: 0,
            cooling_down_amount: 0,
            enqueued_for_withdraw_amount: 0,
            cooling_down_for_withdraw_amount: 0,
            reserved: [0; 7],
            is_used: 1,
        }
    }

    pub fn clear(&mut self) {
        self.operator = Pubkey::default();
        self.staked_amount = 0;
        self.enqueued_for_cooldown_amount = 0;
        self.cooling_down_amount = 0;
        self.enqueued_for_withdraw_amount = 0;
        self.cooling_down_for_withdraw_amount = 0;
        self.is_used = 0;
    }

    pub const fn is_empty(&self) -> bool {
        self.staked_amount == 0
            && self.enqueued_for_cooldown_amount == 0
            && self.cooling_down_amount == 0
            && self.enqueued_for_withdraw_amount == 0
            && self.cooling_down_for_withdraw_amount == 0
    }

    /// # Returns
    /// The total amount of stake on the operator that can be applied for security, which includes
    /// the active and any cooling down stake for re-delegation or withdrawal
    pub fn total_security(&self) -> Result<u64, VaultError> {
        self.staked_amount
            .checked_add(self.enqueued_for_cooldown_amount)
            .and_then(|x| x.checked_add(self.cooling_down_amount))
            .and_then(|x| x.checked_add(self.enqueued_for_withdraw_amount))
            .and_then(|x| x.checked_add(self.cooling_down_for_withdraw_amount))
            .ok_or(VaultError::OperatorDelegationTotalSecurityOverflow)
    }

    /// Returns the amount of withdrawable security, which is the sum of the amount actively staked,
    /// the amount enqueued for cooldown, and the cooling down amount.
    pub fn withdrawable_security(&self) -> Result<u64, VaultError> {
        self.staked_amount
            .checked_add(self.enqueued_for_cooldown_amount)
            .and_then(|x| x.checked_add(self.cooling_down_amount))
            .ok_or(VaultError::OperatorDelegationWithdrawableSecurityOverflow)
    }

    #[inline(always)]
    pub fn update(&mut self) -> u64 {
        let available_for_withdraw = self.cooling_down_for_withdraw_amount;
        self.cooling_down_amount = self.enqueued_for_cooldown_amount;
        self.enqueued_for_cooldown_amount = 0;
        self.cooling_down_for_withdraw_amount = self.enqueued_for_withdraw_amount;
        self.enqueued_for_withdraw_amount = 0;

        available_for_withdraw
    }

    pub fn slash(&mut self, slash_amount: u64) -> Result<(), VaultError> {
        let total_security_amount = self.total_security()?;
        if slash_amount > total_security_amount {
            msg!(
                "slash amount exceeds total security (slash_amount: {}, total_security: {})",
                slash_amount,
                total_security_amount
            );
            return Err(VaultError::OperatorDelegationSlashExceedsTotalSecurity);
        }

        let mut remaining_slash = slash_amount;

        // Helper function to calculate and apply slash based on pro-rata share
        let mut apply_slash = |amount: &mut u64| -> Result<(), VaultError> {
            if *amount == 0 || remaining_slash == 0 {
                return Ok(());
            }
            let pro_rata_slash = (*amount as u128)
                .checked_mul(slash_amount as u128)
                .ok_or(VaultError::OperatorDelegationSlashOverflow)?
                .div_ceil(total_security_amount as u128);
            let actual_slash = min(pro_rata_slash as u64, min(*amount, remaining_slash));
            *amount = amount
                .checked_sub(actual_slash)
                .ok_or(VaultError::OperatorDelegationSlashUnderflow)?;
            remaining_slash = remaining_slash
                .checked_sub(actual_slash)
                .ok_or(VaultError::OperatorDelegationSlashUnderflow)?;
            Ok(())
        };

        // Slash from each bucket
        apply_slash(&mut self.staked_amount)?;
        apply_slash(&mut self.enqueued_for_cooldown_amount)?;
        apply_slash(&mut self.cooling_down_amount)?;
        apply_slash(&mut self.enqueued_for_withdraw_amount)?;
        apply_slash(&mut self.cooling_down_for_withdraw_amount)?;

        // Ensure we've slashed the exact amount requested
        if remaining_slash > 0 {
            msg!("slashing incomplete ({} remaining)", remaining_slash);
            return Err(VaultError::OperatorDelegationSlashIncomplete);
        }

        Ok(())
    }

    /// Undelegates assets from the operator, pulling from the staked assets.
    pub fn undelegate(&mut self, amount: u64) -> Result<(), VaultError> {
        self.staked_amount = self
            .staked_amount
            .checked_sub(amount)
            .ok_or(VaultError::OperatorDelegationUndelegateUnderflow)?;
        self.enqueued_for_cooldown_amount =
            self.enqueued_for_cooldown_amount
                .checked_add(amount)
                .ok_or(VaultError::OperatorDelegationUndelegateOverflow)?;

        Ok(())
    }

    /// Un-delegates assets for withdraw from the operator. If the total amount to withdraw is
    /// greater than the staked amount, it pulls from the enqueued_for_cooldown_amount.
    /// If there is still excess, it pulls from the cooling_down_amount.
    ///
    /// Funds that are cooling down are likely meant to be re-delegated by the delegation manager.
    /// The function first withdraws from staked assets, falling back to cooling down assets
    /// to avoid blocking the delegation manager from redelegating.
    pub fn undelegate_for_withdraw(&mut self, amount: u64) -> Result<(), VaultError> {
        if amount > self.withdrawable_security()? {
            return Err(VaultError::OperatorDelegationUndelegateUnderflow);
        }

        let mut amount_left = amount;

        let staked_amount_withdraw = min(self.staked_amount, amount_left);
        self.staked_amount = self
            .staked_amount
            .checked_sub(staked_amount_withdraw)
            .ok_or(VaultError::OperatorDelegationUndelegateUnderflow)?;
        amount_left = amount_left
            .checked_sub(staked_amount_withdraw)
            .ok_or(VaultError::OperatorDelegationUndelegateUnderflow)?;

        let enqueued_for_cooldown_amount_withdraw =
            min(self.enqueued_for_cooldown_amount, amount_left);
        self.enqueued_for_cooldown_amount = self
            .enqueued_for_cooldown_amount
            .checked_sub(enqueued_for_cooldown_amount_withdraw)
            .ok_or(VaultError::OperatorDelegationUndelegateUnderflow)?;
        amount_left = amount_left
            .checked_sub(enqueued_for_cooldown_amount_withdraw)
            .ok_or(VaultError::OperatorDelegationUndelegateUnderflow)?;

        let cooldown_amount_withdraw = min(self.cooling_down_amount, amount_left);
        self.cooling_down_amount = self
            .cooling_down_amount
            .checked_sub(cooldown_amount_withdraw)
            .ok_or(VaultError::OperatorDelegationUndelegateUnderflow)?;

        self.enqueued_for_withdraw_amount =
            self.enqueued_for_withdraw_amount
                .checked_add(amount)
                .ok_or(VaultError::OperatorDelegationUndelegateUnderflow)?;

        Ok(())
    }

    pub fn delegate(&mut self, amount: u64) -> Result<(), VaultError> {
        self.staked_amount = self
            .staked_amount
            .checked_add(amount)
            .ok_or(VaultError::OperatorDelegationDelegateOverflow)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use crate::operator_delegation::OperatorDelegation;

    #[test]
    fn test_delegate() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());
        operator_delegation.delegate(100).unwrap();
        assert_eq!(operator_delegation.staked_amount, 100);
        assert_eq!(operator_delegation.total_security().unwrap(), 100);
    }

    #[test]
    fn test_undelegate_with_updates() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());
        operator_delegation.delegate(100).unwrap();
        operator_delegation.undelegate(50).unwrap();

        assert_eq!(operator_delegation.staked_amount, 50);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 50);
        assert_eq!(operator_delegation.cooling_down_amount, 0);
        assert_eq!(operator_delegation.total_security().unwrap(), 100);

        assert_eq!(operator_delegation.update(), 0);

        assert_eq!(operator_delegation.staked_amount, 50);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(operator_delegation.cooling_down_amount, 50);
        assert_eq!(operator_delegation.total_security().unwrap(), 100);

        assert_eq!(operator_delegation.update(), 0);

        assert_eq!(operator_delegation.staked_amount, 50);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(operator_delegation.cooling_down_amount, 0);
        assert_eq!(operator_delegation.total_security().unwrap(), 50);
    }

    #[test]
    fn test_undelegate_for_withdraw_with_updates() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());
        operator_delegation.delegate(100).unwrap();
        operator_delegation.undelegate_for_withdraw(50).unwrap();

        assert_eq!(operator_delegation.staked_amount, 50);
        assert_eq!(operator_delegation.enqueued_for_withdraw_amount, 50);
        assert_eq!(operator_delegation.cooling_down_for_withdraw_amount, 0);
        assert_eq!(operator_delegation.total_security().unwrap(), 100);

        assert_eq!(operator_delegation.update(), 0);

        assert_eq!(operator_delegation.staked_amount, 50);
        assert_eq!(operator_delegation.enqueued_for_withdraw_amount, 0);
        assert_eq!(operator_delegation.cooling_down_for_withdraw_amount, 50);
        assert_eq!(operator_delegation.total_security().unwrap(), 100);

        assert_eq!(operator_delegation.update(), 50);

        assert_eq!(operator_delegation.staked_amount, 50);
        assert_eq!(operator_delegation.enqueued_for_withdraw_amount, 0);
        assert_eq!(operator_delegation.cooling_down_for_withdraw_amount, 0);
        assert_eq!(operator_delegation.total_security().unwrap(), 50);
    }

    #[test]
    fn test_slashing_simple() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());
        operator_delegation.delegate(100_000).unwrap();
        operator_delegation.undelegate(10_000).unwrap();
        operator_delegation.slash(5_000).unwrap();

        assert_eq!(operator_delegation.total_security().unwrap(), 95_000);
        assert_eq!(operator_delegation.staked_amount, 85_500);
    }

    #[test]
    fn test_undelegate_for_withdraw_with_cooling_down() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());
        operator_delegation.delegate(100_000).unwrap();
        assert_eq!(operator_delegation.staked_amount, 100_000);

        operator_delegation.undelegate(10_000).unwrap();
        assert_eq!(operator_delegation.staked_amount, 90_000);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 10_000);

        operator_delegation.undelegate_for_withdraw(95_000).unwrap();
        assert_eq!(operator_delegation.staked_amount, 0);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 5_000);
        assert_eq!(operator_delegation.enqueued_for_withdraw_amount, 95_000);
    }

    #[test]
    fn test_undelegate_for_withdraw_not_enough_security() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());
        operator_delegation.delegate(100_000).unwrap();

        operator_delegation
            .undelegate_for_withdraw(100_001)
            .unwrap_err();

        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());
        operator_delegation.delegate(100_000).unwrap();
        operator_delegation.undelegate_for_withdraw(50_000).unwrap();
        operator_delegation
            .undelegate_for_withdraw(50_001)
            .unwrap_err();
    }

    /// Test pulling assets from enqueued for cooling down after staked assets are exhausted
    #[test]
    fn test_undelegate_for_withdraw_pull_from_enqueued_for_cooling_down() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());

        operator_delegation.delegate(100_000).unwrap();
        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);

        operator_delegation.undelegate(50_000).unwrap();
        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(operator_delegation.staked_amount, 50_000);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 50_000);

        // shall pull 50,000 from the staked and 10,000 from the undelegated
        operator_delegation.undelegate_for_withdraw(60_000).unwrap();

        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(operator_delegation.staked_amount, 0);
        assert_eq!(operator_delegation.enqueued_for_withdraw_amount, 60_000);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 40_000);
    }

    /// Test pulling assets from cooling down after staked assets are exhausted
    #[test]
    fn test_undelegate_for_withdraw_pull_from_cooling_down() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());

        operator_delegation.delegate(100_000).unwrap();
        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);

        operator_delegation.undelegate(50_000).unwrap();
        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(operator_delegation.staked_amount, 50_000);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 50_000);

        assert_eq!(operator_delegation.update(), 0);

        // shall pull 50,000 from the staked and 10,000 from the undelegated
        operator_delegation.undelegate_for_withdraw(60_000).unwrap();

        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(operator_delegation.staked_amount, 0);
        assert_eq!(operator_delegation.enqueued_for_withdraw_amount, 60_000);
        assert_eq!(operator_delegation.cooling_down_amount, 40_000);
    }

    #[test]
    fn test_undelegate_for_withdraw_pull_from_enqueued_for_cooling_down_and_cooling_down() {
        let mut operator_delegation = OperatorDelegation::new(Pubkey::new_unique());

        operator_delegation.delegate(100_000).unwrap();
        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);

        operator_delegation.undelegate(50_000).unwrap();
        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(operator_delegation.staked_amount, 50_000);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 50_000);

        assert_eq!(operator_delegation.update(), 0);

        operator_delegation.undelegate(10_000).unwrap();

        // 100k total security, 40k staked, 10k in enqueued for cooling down, 50k in cooling down

        operator_delegation.undelegate_for_withdraw(60_000).unwrap();
        // shall pull 40,000 from the staked, 10k from the enqueued for cooling down, and 10k from cooling down

        assert_eq!(operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(operator_delegation.staked_amount, 0);
        assert_eq!(operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(operator_delegation.cooling_down_amount, 40_000);
        assert_eq!(operator_delegation.enqueued_for_withdraw_amount, 60_000);
    }
}
