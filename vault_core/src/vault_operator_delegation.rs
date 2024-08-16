//! The [`VaultOperatorDelegation`] account tracks a vault's delegation to an operator

use std::cmp::min;

use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_vault_sdk::error::VaultError;
use solana_program::{msg, pubkey::Pubkey};

impl Discriminator for VaultOperatorDelegation {
    const DISCRIMINATOR: u8 = 4;
}

/// The [`VaultOperatorDelegation`] account tracks a vault's delegation to an operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultOperatorDelegation {
    /// The vault account
    pub vault: Pubkey,

    /// The operator account
    pub operator: Pubkey,

    /// The last slot the [`VaultOperatorDelegation::update`] method was updated
    pub last_update_slot: u64,

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

    /// The index
    pub index: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultOperatorDelegation {
    pub const fn new(vault: Pubkey, operator: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            vault,
            operator,
            last_update_slot: 0,
            staked_amount: 0,
            enqueued_for_cooldown_amount: 0,
            cooling_down_amount: 0,
            enqueued_for_withdraw_amount: 0,
            cooling_down_for_withdraw_amount: 0,
            index,
            bump,
            reserved: [0; 7],
        }
    }

    pub fn is_update_needed(&self, slot: u64, epoch_length: u64) -> bool {
        let last_updated_epoch = self.last_update_slot.checked_div(epoch_length).unwrap();
        let current_epoch = slot.checked_div(epoch_length).unwrap();
        last_updated_epoch < current_epoch
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

    /// Updates the state of the delegation
    /// The cooling_down_amount becomes the enqueued_for_cooldown_amount
    /// The enqueued_for_cooldown_amount is zeroed out
    /// The cooling_down_for_withdraw_amount becomes the enqueued_for_withdraw_amount
    /// The enqueued_for_withdraw_amount is zeroed out
    #[inline(always)]
    pub fn update(&mut self, slot: u64) {
        self.cooling_down_amount = self.enqueued_for_cooldown_amount;
        self.enqueued_for_cooldown_amount = 0;
        self.cooling_down_for_withdraw_amount = self.enqueued_for_withdraw_amount;
        self.enqueued_for_withdraw_amount = 0;

        self.last_update_slot = slot;
    }

    /// Slashes the operator delegation by the given amount
    /// All buckets are slashed pro-rata based on the total security amount
    ///
    /// # Arguments
    /// * `slash_amount` - The amount to slash
    ///
    /// # Returns
    /// * `Ok(())` if the slash was successful
    /// * `Err(VaultError)` if the slash failed
    pub fn slash(&mut self, slash_amount: u64) -> Result<(), VaultError> {
        // ensure the there's no underflow when slashing
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

        // Slash from each bucket
        // Ordering for the withdrawal buckets is important to ensure that those funds are available
        // for withdrawal. The `enqueued_for_withdraw_amount` is slashed first so that the delegation
        // admin can set aside more in the future if needed. Also, the vault update process has a
        // last look at moving funds around if needed.
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

    /// Undelegates assets from the operator, pulling from the staked assets.
    pub fn undelegate(&mut self, amount: u64) -> Result<(), VaultError> {
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

    /// Un-delegates assets for withdraw from the operator. If the total amount to withdraw is
    /// greater than the staked amount, it pulls from the enqueued_for_cooldown_amount.
    /// If there is still excess, it pulls from the cooling_down_amount.
    ///
    /// Funds that are cooling down are likely meant to be re-delegated by the delegation manager.
    /// The function first withdraws from staked assets, falling back to cooling down assets
    /// to avoid blocking the delegation manager from redelegating.
    pub fn undelegate_for_withdraw(&mut self, amount: u64) -> Result<(), VaultError> {
        if amount > self.withdrawable_security()? {
            msg!("Attempting to withdraw too much from the vault");
            return Err(VaultError::VaultSecurityUnderflow);
        }

        let mut amount_left = amount;

        let staked_amount_withdraw = min(self.staked_amount, amount_left);
        self.staked_amount = self
            .staked_amount
            .checked_sub(staked_amount_withdraw)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        amount_left = amount_left
            .checked_sub(staked_amount_withdraw)
            .ok_or(VaultError::VaultSecurityUnderflow)?;

        let enqueued_for_cooldown_amount_withdraw =
            min(self.enqueued_for_cooldown_amount, amount_left);
        self.enqueued_for_cooldown_amount = self
            .enqueued_for_cooldown_amount
            .checked_sub(enqueued_for_cooldown_amount_withdraw)
            .ok_or(VaultError::VaultSecurityUnderflow)?;
        amount_left = amount_left
            .checked_sub(enqueued_for_cooldown_amount_withdraw)
            .ok_or(VaultError::VaultSecurityUnderflow)?;

        let cooldown_amount_withdraw = min(self.cooling_down_amount, amount_left);
        self.cooling_down_amount = self
            .cooling_down_amount
            .checked_sub(cooldown_amount_withdraw)
            .ok_or(VaultError::VaultSecurityUnderflow)?;

        self.enqueued_for_withdraw_amount = self
            .enqueued_for_withdraw_amount
            .checked_add(amount)
            .ok_or(VaultError::VaultSecurityUnderflow)?;

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

    /// The seeds for the PDA
    pub fn seeds(vault: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_operator_delegation".to_vec(),
            vault.as_ref().to_vec(),
            operator.as_ref().to_vec(),
        ])
    }

    /// Find the program address for the PDA
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault` - The vault account
    /// * `operator` - The operator account
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds
    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use crate::vault_operator_delegation::VaultOperatorDelegation;

    fn new_vault_operator_delegation() -> VaultOperatorDelegation {
        VaultOperatorDelegation::new(Pubkey::new_unique(), Pubkey::new_unique(), 0, 0)
    }

    #[test]
    fn test_delegate() {
        let mut vault_operator_delegation = new_vault_operator_delegation();
        vault_operator_delegation.delegate(100).unwrap();
        assert_eq!(vault_operator_delegation.staked_amount, 100);
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100);
    }

    #[test]
    fn test_undelegate_with_updates() {
        let mut vault_operator_delegation = new_vault_operator_delegation();
        vault_operator_delegation.delegate(100).unwrap();
        vault_operator_delegation.undelegate(50).unwrap();

        assert_eq!(vault_operator_delegation.staked_amount, 50);
        assert_eq!(vault_operator_delegation.enqueued_for_cooldown_amount, 50);
        assert_eq!(vault_operator_delegation.cooling_down_amount, 0);
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100);

        vault_operator_delegation.update(0);

        assert_eq!(vault_operator_delegation.staked_amount, 50);
        assert_eq!(vault_operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(vault_operator_delegation.cooling_down_amount, 50);
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100);

        vault_operator_delegation.update(0);

        assert_eq!(vault_operator_delegation.staked_amount, 50);
        assert_eq!(vault_operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(vault_operator_delegation.cooling_down_amount, 0);
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 50);
    }

    #[test]
    fn test_undelegate_for_withdraw_with_updates() {
        let mut vault_operator_delegation = new_vault_operator_delegation();
        vault_operator_delegation.delegate(100).unwrap();
        vault_operator_delegation
            .undelegate_for_withdraw(50)
            .unwrap();

        assert_eq!(vault_operator_delegation.staked_amount, 50);
        assert_eq!(vault_operator_delegation.enqueued_for_withdraw_amount, 50);
        assert_eq!(
            vault_operator_delegation.cooling_down_for_withdraw_amount,
            0
        );
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100);

        vault_operator_delegation.update(0);

        assert_eq!(vault_operator_delegation.staked_amount, 50);
        assert_eq!(vault_operator_delegation.enqueued_for_withdraw_amount, 0);
        assert_eq!(
            vault_operator_delegation.cooling_down_for_withdraw_amount,
            50
        );
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100);

        vault_operator_delegation.update(0);

        assert_eq!(vault_operator_delegation.staked_amount, 50);
        assert_eq!(vault_operator_delegation.enqueued_for_withdraw_amount, 0);
        assert_eq!(
            vault_operator_delegation.cooling_down_for_withdraw_amount,
            0
        );
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 50);
    }

    #[test]
    fn test_slashing_simple() {
        let mut vault_operator_delegation = new_vault_operator_delegation();
        vault_operator_delegation.delegate(100_000).unwrap();
        vault_operator_delegation.undelegate(10_000).unwrap();
        vault_operator_delegation.slash(5_000).unwrap();

        assert_eq!(vault_operator_delegation.total_security().unwrap(), 95_000);
        assert_eq!(vault_operator_delegation.staked_amount, 85_000);
    }

    #[test]
    fn test_undelegate_for_withdraw_with_cooling_down() {
        let mut vault_operator_delegation = new_vault_operator_delegation();
        vault_operator_delegation.delegate(100_000).unwrap();
        assert_eq!(vault_operator_delegation.staked_amount, 100_000);

        vault_operator_delegation.undelegate(10_000).unwrap();
        assert_eq!(vault_operator_delegation.staked_amount, 90_000);
        assert_eq!(
            vault_operator_delegation.enqueued_for_cooldown_amount,
            10_000
        );

        vault_operator_delegation
            .undelegate_for_withdraw(95_000)
            .unwrap();
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(
            vault_operator_delegation.enqueued_for_cooldown_amount,
            5_000
        );
        assert_eq!(
            vault_operator_delegation.enqueued_for_withdraw_amount,
            95_000
        );
    }

    #[test]
    fn test_undelegate_for_withdraw_not_enough_security() {
        let mut vault_operator_delegation = new_vault_operator_delegation();
        vault_operator_delegation.delegate(100_000).unwrap();

        vault_operator_delegation
            .undelegate_for_withdraw(100_001)
            .unwrap_err();

        let mut vault_operator_delegation = new_vault_operator_delegation();
        vault_operator_delegation.delegate(100_000).unwrap();
        vault_operator_delegation
            .undelegate_for_withdraw(50_000)
            .unwrap();
        vault_operator_delegation
            .undelegate_for_withdraw(50_001)
            .unwrap_err();
    }

    /// Test pulling assets from enqueued for cooling down after staked assets are exhausted
    #[test]
    fn test_undelegate_for_withdraw_pull_from_enqueued_for_cooling_down() {
        let mut vault_operator_delegation = new_vault_operator_delegation();

        vault_operator_delegation.delegate(100_000).unwrap();
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);

        vault_operator_delegation.undelegate(50_000).unwrap();
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(vault_operator_delegation.staked_amount, 50_000);
        assert_eq!(
            vault_operator_delegation.enqueued_for_cooldown_amount,
            50_000
        );

        // shall pull 50,000 from the staked and 10,000 from the undelegated
        vault_operator_delegation
            .undelegate_for_withdraw(60_000)
            .unwrap();

        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(
            vault_operator_delegation.enqueued_for_withdraw_amount,
            60_000
        );
        assert_eq!(
            vault_operator_delegation.enqueued_for_cooldown_amount,
            40_000
        );
    }

    /// Test pulling assets from cooling down after staked assets are exhausted
    #[test]
    fn test_undelegate_for_withdraw_pull_from_cooling_down() {
        let mut vault_operator_delegation = new_vault_operator_delegation();

        vault_operator_delegation.delegate(100_000).unwrap();
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);

        vault_operator_delegation.undelegate(50_000).unwrap();
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(vault_operator_delegation.staked_amount, 50_000);
        assert_eq!(
            vault_operator_delegation.enqueued_for_cooldown_amount,
            50_000
        );

        vault_operator_delegation.update(0);

        // shall pull 50,000 from the staked and 10,000 from the undelegated
        vault_operator_delegation
            .undelegate_for_withdraw(60_000)
            .unwrap();

        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(
            vault_operator_delegation.enqueued_for_withdraw_amount,
            60_000
        );
        assert_eq!(vault_operator_delegation.cooling_down_amount, 40_000);
    }

    #[test]
    fn test_undelegate_for_withdraw_pull_from_enqueued_for_cooling_down_and_cooling_down() {
        let mut vault_operator_delegation = new_vault_operator_delegation();

        vault_operator_delegation.delegate(100_000).unwrap();
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);

        vault_operator_delegation.undelegate(50_000).unwrap();
        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(vault_operator_delegation.staked_amount, 50_000);
        assert_eq!(
            vault_operator_delegation.enqueued_for_cooldown_amount,
            50_000
        );

        vault_operator_delegation.update(0);

        vault_operator_delegation.undelegate(10_000).unwrap();

        // 100k total security, 40k staked, 10k in enqueued for cooling down, 50k in cooling down

        vault_operator_delegation
            .undelegate_for_withdraw(60_000)
            .unwrap();
        // shall pull 40,000 from the staked, 10k from the enqueued for cooling down, and 10k from cooling down

        assert_eq!(vault_operator_delegation.total_security().unwrap(), 100_000);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(vault_operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(vault_operator_delegation.cooling_down_amount, 40_000);
        assert_eq!(
            vault_operator_delegation.enqueued_for_withdraw_amount,
            60_000
        );
    }
}
