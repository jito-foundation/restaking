use std::{cmp::min, fmt::Debug};

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::types::PodU64;
use jito_vault_sdk::error::VaultError;
use shank::ShankType;
use solana_program::msg;

const RESERVED_SPACE_LEN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, ShankType)]
#[repr(C)]
pub struct DelegationState {
    /// The amount of stake that is currently active on the operator
    staked_amount: PodU64,

    /// Any stake that was deactivated in the current epoch
    enqueued_for_cooldown_amount: PodU64,

    /// Any stake that was deactivated in the previous epoch,
    /// to be available for re-delegation in the current epoch + 1
    cooling_down_amount: PodU64,

    reserved: [u8; 256],
}

impl Default for DelegationState {
    fn default() -> Self {
        Self {
            staked_amount: PodU64::from(0),
            enqueued_for_cooldown_amount: PodU64::from(0),
            cooling_down_amount: PodU64::from(0),
            reserved: [0; RESERVED_SPACE_LEN],
        }
    }
}

impl DelegationState {
    pub fn new(
        staked_amount: u64,
        enqueued_for_cooldown_amount: u64,
        cooling_down_amount: u64,
    ) -> Self {
        Self {
            staked_amount: PodU64::from(staked_amount),
            enqueued_for_cooldown_amount: PodU64::from(enqueued_for_cooldown_amount),
            cooling_down_amount: PodU64::from(cooling_down_amount),
            reserved: [0; RESERVED_SPACE_LEN],
        }
    }

    pub fn staked_amount(&self) -> u64 {
        self.staked_amount.into()
    }

    pub fn enqueued_for_cooldown_amount(&self) -> u64 {
        self.enqueued_for_cooldown_amount.into()
    }

    pub fn cooling_down_amount(&self) -> u64 {
        self.cooling_down_amount.into()
    }

    pub fn subtract(&mut self, other: &Self) -> Result<(), VaultError> {
        let mut staked_amount: u64 = self.staked_amount.into();
        staked_amount = staked_amount
            .checked_sub(other.staked_amount.into())
            .ok_or(VaultError::VaultSecurityUnderflow)?;

        let mut enqueued_for_cooldown_amount: u64 = self.enqueued_for_cooldown_amount.into();
        enqueued_for_cooldown_amount = enqueued_for_cooldown_amount
            .checked_sub(other.enqueued_for_cooldown_amount.into())
            .ok_or(VaultError::VaultSecurityUnderflow)?;

        let mut cooling_down_amount: u64 = self.cooling_down_amount.into();
        cooling_down_amount = cooling_down_amount
            .checked_sub(other.cooling_down_amount.into())
            .ok_or(VaultError::VaultSecurityUnderflow)?;

        self.staked_amount = PodU64::from(staked_amount);
        self.enqueued_for_cooldown_amount = PodU64::from(enqueued_for_cooldown_amount);
        self.cooling_down_amount = PodU64::from(cooling_down_amount);
        Ok(())
    }

    /// Used to accumulate the state of other into the state of self
    pub fn accumulate(&mut self, other: &Self) -> Result<(), VaultError> {
        let mut staked_amount: u64 = self.staked_amount.into();
        staked_amount = staked_amount
            .checked_add(other.staked_amount.into())
            .ok_or(VaultError::VaultSecurityOverflow)?;

        let mut enqueued_for_cooldown_amount: u64 = self.enqueued_for_cooldown_amount.into();
        enqueued_for_cooldown_amount = enqueued_for_cooldown_amount
            .checked_add(other.enqueued_for_cooldown_amount.into())
            .ok_or(VaultError::VaultSecurityOverflow)?;

        let mut cooling_down_amount: u64 = self.cooling_down_amount.into();
        cooling_down_amount = cooling_down_amount
            .checked_add(other.cooling_down_amount.into())
            .ok_or(VaultError::VaultSecurityOverflow)?;

        self.staked_amount = PodU64::from(staked_amount);
        self.enqueued_for_cooldown_amount = PodU64::from(enqueued_for_cooldown_amount);
        self.cooling_down_amount = PodU64::from(cooling_down_amount);
        Ok(())
    }

    /// # Returns
    /// The total amount of stake on the operator that can be applied for security, which includes
    /// the active and any cooling down stake for re-delegation or withdrawal
    pub fn total_security(&self) -> Result<u64, VaultError> {
        let staked_amount: u64 = self.staked_amount.into();
        let enqueued_for_cooldown_amount: u64 = self.enqueued_for_cooldown_amount.into();
        let cooling_down_amount: u64 = self.cooling_down_amount.into();

        staked_amount
            .checked_add(enqueued_for_cooldown_amount)
            .and_then(|x| x.checked_add(cooling_down_amount))
            .ok_or(VaultError::VaultSecurityOverflow)
    }

    #[inline(always)]
    pub fn update(&mut self) {
        self.cooling_down_amount = self.enqueued_for_cooldown_amount;
        self.enqueued_for_cooldown_amount = PodU64::from(0);
    }

    /// Slashes the operator delegation by the given amount.
    ///
    /// Slashes are applied in the following order:
    /// 1. Staked amount
    /// 2. Enqueued for cooldown amount
    /// 3. Cooling down amount
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
        let mut staked_amount: u64 = self.staked_amount.into();
        apply_slash(&mut staked_amount)?;
        self.staked_amount = PodU64::from(staked_amount);

        let mut enqueued_for_cooldown_amount: u64 = self.enqueued_for_cooldown_amount.into();
        apply_slash(&mut enqueued_for_cooldown_amount)?;
        self.enqueued_for_cooldown_amount = PodU64::from(enqueued_for_cooldown_amount);

        let mut cooling_down_amount: u64 = self.cooling_down_amount.into();
        apply_slash(&mut cooling_down_amount)?;
        self.cooling_down_amount = PodU64::from(cooling_down_amount);

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
        if amount == 0 {
            msg!("Cooldown amount is zero");
            return Err(VaultError::VaultCooldownZero);
        }

        let mut staked_amount: u64 = self.staked_amount.into();
        staked_amount = staked_amount
            .checked_sub(amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        let mut enqueued_for_cooldown_amount: u64 = self.enqueued_for_cooldown_amount.into();
        enqueued_for_cooldown_amount = enqueued_for_cooldown_amount
            .checked_add(amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;

        self.staked_amount = PodU64::from(staked_amount);
        self.enqueued_for_cooldown_amount = PodU64::from(enqueued_for_cooldown_amount);

        Ok(())
    }

    /// Delegates assets to the operator
    pub fn delegate(&mut self, amount: u64) -> Result<(), VaultError> {
        if amount == 0 {
            msg!("Delegation amount is zero");
            return Err(VaultError::VaultDelegationZero);
        }

        let mut staked_amount: u64 = self.staked_amount.into();
        staked_amount = staked_amount
            .checked_add(amount)
            .ok_or(VaultError::VaultSecurityOverflow)?;
        self.staked_amount = PodU64::from(staked_amount);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use jito_bytemuck::types::PodU64;
    use jito_vault_sdk::error::VaultError;

    use super::{DelegationState, RESERVED_SPACE_LEN};

    #[test]
    fn test_delegation_state_no_padding() {
        let delegation_state_size = std::mem::size_of::<DelegationState>();
        let sum_of_fields = size_of::<PodU64>() // staked_amount
         + size_of::<PodU64>() // enqueued_for_cooldown_amount
         + size_of::<PodU64>() // cooling_down_amount
         + RESERVED_SPACE_LEN; // reserved
        assert_eq!(delegation_state_size, sum_of_fields);
    }

    #[test]
    fn test_undo_self_zeroes() {
        let mut delegation_state = DelegationState::new(1, 2, 3);
        let copy = delegation_state;
        delegation_state.subtract(&copy).unwrap();
        assert_eq!(delegation_state, DelegationState::default());
    }

    #[test]
    fn test_undo_complex() {
        let mut delegation_state_1 = DelegationState::new(10, 20, 30);
        let delegation_state_2 = DelegationState::new(5, 10, 15);
        delegation_state_1.subtract(&delegation_state_2).unwrap();
        assert_eq!(delegation_state_1.staked_amount(), 5);
        assert_eq!(delegation_state_1.enqueued_for_cooldown_amount(), 10);
        assert_eq!(delegation_state_1.cooling_down_amount(), 15);
    }

    #[test]
    fn test_delegate() {
        let mut delegation_state = DelegationState::default();
        delegation_state.delegate(100).unwrap();
        assert_eq!(delegation_state.staked_amount(), 100);
        assert_eq!(delegation_state.total_security().unwrap(), 100);
    }

    #[test]
    fn test_delegate_cooling_down() {
        let mut delegation_state = DelegationState::default();

        delegation_state.delegate(100).unwrap();

        delegation_state.cooldown(50).unwrap();
        assert_eq!(delegation_state.staked_amount(), 50);
        assert_eq!(delegation_state.enqueued_for_cooldown_amount(), 50);
        assert_eq!(delegation_state.total_security().unwrap(), 100);

        delegation_state.update();
        assert_eq!(delegation_state.staked_amount(), 50);
        assert_eq!(delegation_state.enqueued_for_cooldown_amount(), 0);
        assert_eq!(delegation_state.cooling_down_amount(), 50);
        assert_eq!(delegation_state.total_security().unwrap(), 100);

        delegation_state.update();
        assert_eq!(delegation_state.staked_amount(), 50);
        assert_eq!(delegation_state.enqueued_for_cooldown_amount(), 0);
        assert_eq!(delegation_state.cooling_down_amount(), 0);
        assert_eq!(delegation_state.total_security().unwrap(), 50);
    }

    #[test]
    fn test_delegate_zero() {
        let mut delegation_state = DelegationState::default();
        assert_eq!(
            delegation_state.delegate(0),
            Err(VaultError::VaultDelegationZero)
        );
    }

    #[test]
    fn test_cooldown_zero() {
        let mut delegation_state = DelegationState::new(100, 0, 0);
        assert_eq!(
            delegation_state.cooldown(0),
            Err(VaultError::VaultCooldownZero)
        );
    }
}
