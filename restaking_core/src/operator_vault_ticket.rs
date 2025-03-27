use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

const RESERVED_LEN: usize = 263;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct OperatorVaultTicket {
    /// The operator account
    pub operator: Pubkey,

    /// The vault account
    pub vault: Pubkey,

    /// The index
    index: PodU64,

    /// The slot toggle
    pub state: SlotToggle,

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 263],
}

impl OperatorVaultTicket {
    pub fn new(operator: Pubkey, vault: Pubkey, index: u64, bump: u8, slot: u64) -> Self {
        Self {
            operator,
            vault,
            index: PodU64::from(index),
            state: SlotToggle::new(slot),
            bump,
            reserved: [0; RESERVED_LEN],
        }
    }

    pub fn index(&self) -> u64 {
        self.index.into()
    }

    pub fn seeds(operator: &Pubkey, vault: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"operator_vault_ticket".to_vec(),
            operator.to_bytes().to_vec(),
            vault.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
        vault: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator, vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the account as an [`OperatorVaultTicket`] account, returning an error if it is not.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `operator_vault_ticket` - The account to load the operator vault ticket from
    /// * `operator` - The operator account
    /// * `vault` - The vault account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        operator_vault_ticket: &AccountInfo,
        operator: &AccountInfo,
        vault: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if operator_vault_ticket.owner.ne(program_id) {
            msg!("Operator vault ticket account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if operator_vault_ticket.data_is_empty() {
            msg!("Operator vault ticket account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !operator_vault_ticket.is_writable {
            msg!("Operator vault ticket account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if operator_vault_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Operator vault ticket account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(program_id, operator.key, vault.key).0;
        if operator_vault_ticket.key.ne(&expected_pubkey) {
            msg!("Operator vault ticket account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggle::SlotToggleState;

    use super::*;

    #[test]
    fn test_operator_vault_ticket_no_padding() {
        let operator_vault_ticket_size = std::mem::size_of::<OperatorVaultTicket>();
        let sum_of_fields = size_of::<Pubkey>() + // operator
            size_of::<Pubkey>() + // vault
            size_of::<PodU64>() + // index
            size_of::<SlotToggle>() + // state
            size_of::<u8>() + // bump
            RESERVED_LEN; // reserved
        assert_eq!(operator_vault_ticket_size, sum_of_fields);
    }

    #[test]
    fn test_operator_vault_ticket_inactive_on_creation() {
        let slot = 1;
        let operator_vault_ticket =
            OperatorVaultTicket::new(Pubkey::default(), Pubkey::default(), 0, 0, slot);
        assert_eq!(
            operator_vault_ticket.state.state(slot + 1, 100).unwrap(),
            SlotToggleState::Inactive
        );
    }
}
