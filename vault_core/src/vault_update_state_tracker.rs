use crate::delegation_state::DelegationState;
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_vault_sdk::error::VaultError;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for VaultUpdateStateTracker {
    const DISCRIMINATOR: u8 = 9;
}

/// The [`crate::vault_operator_delegation::VaultUpdateDelegationsTicket`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultUpdateStateTracker {
    /// The vault associated with this update ticket
    pub vault: Pubkey,

    /// The NCN epoch for which the delegations are to be updated
    pub ncn_epoch: u64,

    /// The update index of the vault
    pub last_updated_index: u64,

    /// The total amount delegated across all the operators in the vault
    pub delegation_state: DelegationState,
}

impl VaultUpdateStateTracker {
    pub fn new(vault: Pubkey, ncn_epoch: u64) -> Self {
        Self {
            vault,
            ncn_epoch,
            last_updated_index: u64::MAX,
            delegation_state: DelegationState::default(),
        }
    }

    pub fn check_and_update_index(&mut self, index: u64) -> Result<(), VaultError> {
        if self.last_updated_index == u64::MAX {
            if index != 0 {
                msg!("VaultUpdateStateTracker incorrect index");
                return Err(VaultError::VaultUpdateIncorrectIndex);
            }
        } else if index != self.last_updated_index.checked_add(1).unwrap() {
            msg!("VaultUpdateStateTracker incorrect index");
            return Err(VaultError::VaultUpdateIncorrectIndex.into());
        }
        self.last_updated_index = index;
        Ok(())
    }

    /// Returns the seeds for the PDA
    ///
    /// # Arguments
    /// * `vault` - The vault
    /// * `ncn_epoch` - The NCN epoch
    pub fn seeds(vault: &Pubkey, ncn_epoch: u64) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_update_state_tracker".to_vec(),
            vault.to_bytes().to_vec(),
            ncn_epoch.to_le_bytes().to_vec(),
        ])
    }

    /// Find the program address for the PDA
    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        ncn_epoch: u64,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, ncn_epoch);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn load(
        program_id: &Pubkey,
        vault_update_delegation_ticket: &AccountInfo,
        vault: &AccountInfo,
        ncn_epoch: u64,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if vault_update_delegation_ticket.owner.ne(program_id) {
            msg!("Vault update delegations ticket has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if vault_update_delegation_ticket.data_is_empty() {
            msg!("Vault update delegations ticket data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !vault_update_delegation_ticket.is_writable {
            msg!("Vault update delegations ticket is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_update_delegation_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Vault update delegations ticket discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(program_id, vault.key, ncn_epoch).0;
        if vault_update_delegation_ticket.key.ne(&expected_pubkey) {
            msg!("Vault update delegations ticket is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::delegation_state::DelegationState;
    use crate::vault_update_state_tracker::VaultUpdateStateTracker;
    use jito_vault_sdk::error::VaultError;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_update_index_zero_ok() {
        let mut vault_update_state_tracker = VaultUpdateStateTracker {
            vault: Pubkey::new_unique(),
            ncn_epoch: 0,
            last_updated_index: u64::MAX,
            delegation_state: DelegationState::default(),
        };
        assert!(vault_update_state_tracker.check_and_update_index(0).is_ok());
    }

    #[test]
    fn test_update_index_skip_zero_fails() {
        let mut vault_update_state_tracker = VaultUpdateStateTracker {
            vault: Pubkey::new_unique(),
            ncn_epoch: 0,
            last_updated_index: u64::MAX,
            delegation_state: DelegationState::default(),
        };
        assert_eq!(
            vault_update_state_tracker.check_and_update_index(1),
            Err(VaultError::VaultUpdateIncorrectIndex)
        );
    }

    #[test]
    fn test_update_index_skip_index_fails() {
        let mut vault_update_state_tracker = VaultUpdateStateTracker {
            vault: Pubkey::new_unique(),
            ncn_epoch: 0,
            last_updated_index: u64::MAX,
            delegation_state: DelegationState::default(),
        };
        vault_update_state_tracker
            .check_and_update_index(0)
            .unwrap();
        vault_update_state_tracker
            .check_and_update_index(1)
            .unwrap();
        assert_eq!(
            vault_update_state_tracker.check_and_update_index(3),
            Err(VaultError::VaultUpdateIncorrectIndex)
        );
    }
}
