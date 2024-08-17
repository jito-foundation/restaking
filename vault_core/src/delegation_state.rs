use std::cmp::min;

use bytemuck::{Pod, Zeroable};
use jito_vault_sdk::error::VaultError;
use solana_program::msg;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct DelegationState {
    /// The amount of stake that is currently active on the operator
    pub staked_amount: u64,

    /// Any stake that was deactivated in the current epoch
    pub enqueued_for_cooldown_amount: u64,

    /// Any stake that was deactivated in the previous epoch,
    /// to be available for re-delegation in the current epoch + 1
    pub cooling_down_amount: u64,

    /// Any stake that was enqueued for withdraw in the current epoch.
    /// These funds are earmarked for withdrawal and are in the last bucket of slashed
    /// assets.
    pub enqueued_for_withdraw_amount: u64,

    /// Any stake that was enqueued for withdraw in the previous epoch.
    /// These funds are earmarked for withdrawal and are in the last bucket of slashed
    /// assets.
    pub cooling_down_for_withdraw_amount: u64,
}

impl DelegationState {
    pub fn undo(&mut self, other: &Self) -> Result<(), VaultError> {
        self.staked_amount = self
            .staked_amount
            .checked_sub(other.staked_amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        self.enqueued_for_cooldown_amount = self
            .enqueued_for_cooldown_amount
            .checked_sub(other.enqueued_for_cooldown_amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        self.cooling_down_amount = self
            .cooling_down_amount
            .checked_sub(other.cooling_down_amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        self.enqueued_for_withdraw_amount = self
            .enqueued_for_withdraw_amount
            .checked_sub(other.enqueued_for_withdraw_amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        self.cooling_down_for_withdraw_amount = self
            .cooling_down_for_withdraw_amount
            .checked_sub(other.cooling_down_for_withdraw_amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        Ok(())
    }

    /// Used to accumulate the state of other into the state of self
    pub fn accumulate(&mut self, other: &Self) -> Result<(), VaultError> {
        self.staked_amount = self
            .staked_amount
            .checked_add(other.staked_amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;
        self.enqueued_for_cooldown_amount = self
            .enqueued_for_cooldown_amount
            .checked_add(other.enqueued_for_cooldown_amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;
        self.cooling_down_amount = self
            .cooling_down_amount
            .checked_add(other.cooling_down_amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;
        self.enqueued_for_withdraw_amount = self
            .enqueued_for_withdraw_amount
            .checked_add(other.enqueued_for_withdraw_amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;
        self.cooling_down_for_withdraw_amount = self
            .cooling_down_for_withdraw_amount
            .checked_add(other.cooling_down_for_withdraw_amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;
        Ok(())
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
            .ok_or(VaultError::VaultSecurityOverflow)
    }

    /// Returns the amount of withdrawable security, which is the sum of the amount actively staked,
    /// the amount enqueued for cooldown, and the cooling down amount.
    pub fn withdrawable_security(&self) -> Result<u64, VaultError> {
        self.staked_amount
            .checked_add(self.enqueued_for_cooldown_amount)
            .and_then(|x| x.checked_add(self.cooling_down_amount))
            .ok_or(VaultError::VaultSecurityOverflow)
    }

    #[inline(always)]
    pub fn update(&mut self) {
        self.cooling_down_amount = self.enqueued_for_cooldown_amount;
        self.enqueued_for_cooldown_amount = 0;
        self.cooling_down_for_withdraw_amount = self.enqueued_for_withdraw_amount;
        self.enqueued_for_withdraw_amount = 0;
    }

    /// Slashes the operator delegation by the given amount.
    ///
    /// Slashes are applied in the following order:
    /// 1. Staked amount
    /// 2. Enqueued for cooldown amount
    /// 3. Cooling down amount
    /// 4. Enqueued for withdraw amount
    /// 5. Cooling down for withdraw amount
    ///
    /// The reason for this is that withdrawals are the most important to ensure that the funds are
    /// available for withdrawal when a user's ticket matures. If any withdrawal funds are slashed,
    /// the vault delegation manager needs to move funds around to ensure that the funds are available
    /// for withdrawal.
    ///
    /// # Arguments
    /// * `slash_amount` - The amount to slash
    ///
    /// # Returns
    /// * `Ok(())` if the slash was successful
    /// * `Err(VaultError)` if the slash failed
    pub fn slash(&mut self, slash_amount: u64) -> Result<(), VaultError> {
        let total_security_amount = self.total_security()?;
        if slash_amount > total_security_amount {
            msg!(
                "slash amount exceeds total security (slash_amount: {}, total_security: {})",
                slash_amount,
                total_security_amount
            );
            return Err(VaultError::VaultSlashUnderflow);
        }

        let mut remaining_slash = slash_amount;

        // Slash as much as possible from the given amount
        let mut apply_slash = |amount: &mut u64| -> Result<(), VaultError> {
            if *amount == 0 || remaining_slash == 0 {
                return Ok(());
            }
            let slash_amount = min(*amount, remaining_slash);
            *amount = amount
                .checked_sub(slash_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
            remaining_slash = remaining_slash
                .checked_sub(slash_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
            Ok(())
        };

        apply_slash(&mut self.staked_amount)?;
        apply_slash(&mut self.enqueued_for_cooldown_amount)?;
        apply_slash(&mut self.cooling_down_amount)?;
        apply_slash(&mut self.enqueued_for_withdraw_amount)?;
        apply_slash(&mut self.cooling_down_for_withdraw_amount)?;

        // Ensure we've slashed the exact amount requested
        if remaining_slash > 0 {
            msg!("slashing incomplete ({} remaining)", remaining_slash);
            return Err(VaultError::VaultSlashIncomplete);
        }

        Ok(())
    }

    /// Cools down stake by subtracting it from the staked amount and adding it to the enqueued
    /// cooldown amount
    pub fn cooldown(&mut self, amount: u64) -> Result<(), VaultError> {
        self.staked_amount = self
            .staked_amount
            .checked_sub(amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        self.enqueued_for_cooldown_amount = self
            .enqueued_for_cooldown_amount
            .checked_add(amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;

        Ok(())
    }

    /// Un-delegates assets for withdraw from the operator
    ///
    /// Opts to pull from buckets in the following order for speed of withdrawal, taking
    /// care to not break the invariant of cooling down assets:
    /// 1. cooling_down_amount -> cooling_down_for_withdraw_amount (moving assets laterally)
    /// 2. enqueued_for_cooldown_amount -> enqueued_for_withdraw_amount (moving assets laterally)
    /// 3. staked_amount -> enqueued_for_withdraw_amount (cooling down)
    pub fn cooldown_for_withdrawal(&mut self, amount: u64) -> Result<(), VaultError> {
        if amount > self.withdrawable_security()? {
            msg!("Attempting to withdraw too much from the vault");
            return Err(VaultError::VaultSecurityUnderflow);
        }

        let mut remaining_amount = amount;

        // cooling_down_amount -> cooling_down_for_withdraw_amount (moving assets laterally)
        if self.cooling_down_amount > 0 {
            let pull_amount = min(self.cooling_down_amount, remaining_amount);
            self.cooling_down_amount = self
                .cooling_down_amount
                .checked_sub(pull_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
            self.cooling_down_for_withdraw_amount = self
                .cooling_down_for_withdraw_amount
                .checked_add(pull_amount)
                .ok_or(VaultError::VaultSecurityOverflow)?;
            remaining_amount = remaining_amount
                .checked_sub(pull_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
        }
        // enqueued_for_cooldown_amount -> enqueued_for_withdraw_amount (moving assets laterally)
        if self.enqueued_for_cooldown_amount > 0 && remaining_amount > 0 {
            let pull_amount = min(self.enqueued_for_cooldown_amount, remaining_amount);
            self.enqueued_for_cooldown_amount = self
                .enqueued_for_cooldown_amount
                .checked_sub(pull_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
            self.enqueued_for_withdraw_amount = self
                .enqueued_for_withdraw_amount
                .checked_add(pull_amount)
                .ok_or(VaultError::VaultSecurityOverflow)?;
            remaining_amount = remaining_amount
                .checked_sub(pull_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
        }
        // staked_amount -> enqueued_for_withdraw_amount (cooling down)
        if self.staked_amount > 0 && remaining_amount > 0 {
            let pull_amount = min(self.staked_amount, remaining_amount);
            self.staked_amount = self
                .staked_amount
                .checked_sub(pull_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
            self.enqueued_for_withdraw_amount = self
                .enqueued_for_withdraw_amount
                .checked_add(pull_amount)
                .ok_or(VaultError::VaultSecurityOverflow)?;
            remaining_amount = remaining_amount
                .checked_sub(pull_amount)
                .ok_or(VaultError::VaultSecurityUnderflow)?;
        }

        if remaining_amount > 0 {
            msg!("Failed to withdraw all the requested amount");
            return Err(VaultError::VaultSecurityUnderflow);
        }

        Ok(())
    }

    /// Delegates assets to the operator
    pub fn delegate(&mut self, amount: u64) -> Result<(), VaultError> {
        self.staked_amount = self
            .staked_amount
            .checked_add(amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::delegation_state::DelegationState;

    #[test]
    fn test_undo_self_zeroes() {
        let mut delegation_state = DelegationState {
            staked_amount: 1,
            enqueued_for_cooldown_amount: 2,
            cooling_down_amount: 3,
            enqueued_for_withdraw_amount: 4,
            cooling_down_for_withdraw_amount: 5,
        };
        let copy = delegation_state.clone();
        delegation_state.undo(&copy).unwrap();
        assert_eq!(delegation_state, DelegationState::default());
    }

    #[test]
    fn test_undo_complex() {
        let mut delegation_state_1 = DelegationState {
            staked_amount: 10,
            enqueued_for_cooldown_amount: 20,
            cooling_down_amount: 30,
            enqueued_for_withdraw_amount: 40,
            cooling_down_for_withdraw_amount: 50,
        };
        let delegation_state_2 = DelegationState {
            staked_amount: 5,
            enqueued_for_cooldown_amount: 10,
            cooling_down_amount: 15,
            enqueued_for_withdraw_amount: 20,
            cooling_down_for_withdraw_amount: 25,
        };
        delegation_state_1.undo(&delegation_state_2).unwrap();
        assert_eq!(delegation_state_1.staked_amount, 5);
        assert_eq!(delegation_state_1.enqueued_for_cooldown_amount, 10);
        assert_eq!(delegation_state_1.cooling_down_amount, 15);
        assert_eq!(delegation_state_1.enqueued_for_withdraw_amount, 20);
        assert_eq!(delegation_state_1.cooling_down_for_withdraw_amount, 25);
    }

    #[test]
    fn test_delegate() {
        let mut delegation_state = DelegationState::default();
        delegation_state.delegate(100).unwrap();
        assert_eq!(delegation_state.staked_amount, 100);
        assert_eq!(delegation_state.total_security().unwrap(), 100);
    }

    #[test]
    fn test_delegate_cooling_down() {
        let mut delegation_state = DelegationState::default();

        delegation_state.delegate(100).unwrap();

        delegation_state.cooldown(50).unwrap();
        assert_eq!(delegation_state.staked_amount, 50);
        assert_eq!(delegation_state.enqueued_for_cooldown_amount, 50);
        assert_eq!(delegation_state.total_security().unwrap(), 100);

        delegation_state.update();
        assert_eq!(delegation_state.staked_amount, 50);
        assert_eq!(delegation_state.enqueued_for_cooldown_amount, 0);
        assert_eq!(delegation_state.cooling_down_amount, 50);
        assert_eq!(delegation_state.total_security().unwrap(), 100);

        delegation_state.update();
        assert_eq!(delegation_state.staked_amount, 50);
        assert_eq!(delegation_state.enqueued_for_cooldown_amount, 0);
        assert_eq!(delegation_state.cooling_down_amount, 0);
        assert_eq!(delegation_state.total_security().unwrap(), 50);
    }

    #[test]
    fn test_delegate_cooling_down_for_withdraw() {
        let mut delegation_state = DelegationState::default();

        delegation_state.delegate(100).unwrap();

        delegation_state.cooldown_for_withdrawal(50).unwrap();
        assert_eq!(delegation_state.staked_amount, 50);
        assert_eq!(delegation_state.enqueued_for_withdraw_amount, 50);
        assert_eq!(delegation_state.total_security().unwrap(), 100);

        delegation_state.update();
        assert_eq!(delegation_state.staked_amount, 50);
        assert_eq!(delegation_state.enqueued_for_withdraw_amount, 0);
        assert_eq!(delegation_state.cooling_down_for_withdraw_amount, 50);
        assert_eq!(delegation_state.total_security().unwrap(), 100);

        delegation_state.update();
        assert_eq!(delegation_state.staked_amount, 50);
        assert_eq!(delegation_state.enqueued_for_withdraw_amount, 0);
        assert_eq!(delegation_state.cooling_down_for_withdraw_amount, 0);
        assert_eq!(delegation_state.total_security().unwrap(), 50);
    }

    #[test]
    fn test_delegate_cooling_down_for_withdraw_fund_transfer() {
        let mut delegation_state = DelegationState {
            staked_amount: 100,
            enqueued_for_cooldown_amount: 50,
            cooling_down_amount: 25,
            enqueued_for_withdraw_amount: 0,
            cooling_down_for_withdraw_amount: 0,
        };

        delegation_state.cooldown_for_withdrawal(150).unwrap();

        assert_eq!(
            delegation_state,
            DelegationState {
                staked_amount: 25,
                enqueued_for_cooldown_amount: 0,
                cooling_down_amount: 0,
                enqueued_for_withdraw_amount: 125,
                cooling_down_for_withdraw_amount: 25,
            }
        );
    }
}
