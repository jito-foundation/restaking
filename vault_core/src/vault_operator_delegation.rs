//! The [`VaultOperatorDelegation`] account tracks a vault's delegation to an operator

use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::delegation_state::DelegationState;

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

    pub delegation_state: DelegationState,

    /// The last slot the [`VaultOperatorDelegation::update`] method was updated
    pub last_update_slot: u64,

    /// The index
    pub index: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultOperatorDelegation {
    pub fn new(vault: Pubkey, operator: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            vault,
            operator,
            last_update_slot: 0,
            delegation_state: DelegationState::default(),
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

    /// Updates the state of the delegation
    /// The cooling_down_amount becomes the enqueued_for_cooldown_amount
    /// The enqueued_for_cooldown_amount is zeroed out
    /// The cooling_down_for_withdraw_amount becomes the enqueued_for_withdraw_amount
    /// The enqueued_for_withdraw_amount is zeroed out
    #[inline(always)]
    pub fn update(&mut self, slot: u64, epoch_length: u64) {
        let last_update_epoch = self.last_update_slot.checked_div(epoch_length).unwrap();
        let current_epoch = slot.checked_div(epoch_length).unwrap();

        let epoch_diff = current_epoch.checked_sub(last_update_epoch).unwrap();
        match epoch_diff {
            0 => {
                // do nothing
            }
            1 => {
                self.delegation_state.update();
            }
            _ => {
                // max 2 transitions needed (enqueued -> cooling down and cooling down -> not allocated)
                self.delegation_state.update();
                self.delegation_state.update();
            }
        }
        self.last_update_slot = slot;
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

    /// Loads the [`VaultOperatorDelegation`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault_operator_delegation` - The [`VaultOperatorDelegation`] account
    /// * `vault` - The vault account
    /// * `operator` - The operator account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        vault_operator_delegation: &AccountInfo,
        vault: &AccountInfo,
        operator: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if vault_operator_delegation.owner.ne(program_id) {
            msg!("Vault operator ticket account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if vault_operator_delegation.data_is_empty() {
            msg!("Vault operator ticket account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !vault_operator_delegation.is_writable {
            msg!("Vault operator ticket account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_operator_delegation.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Vault operator ticket account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(program_id, vault.key, operator.key).0;
        if vault_operator_delegation.key.ne(&expected_pubkey) {
            msg!("Vault operator ticket account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
