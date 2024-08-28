//! The NcnVaultSlasherTicket tracks the opting-in of a slasher to a particular vault.
//! The NcnVaultSlasherTicket can be activated and deactivated over time by the NCN slasher admin.

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for NcnVaultSlasherTicket {
    const DISCRIMINATOR: u8 = 7;
}

/// The NcnVaultSlasherTicket is created by the NCN and it tracks the state of a node consensus network
/// opting-in to a vault slasher. The NcnVaultSlasherTicket can be activated and deactivated over time.
/// The NcnVaultSlasherTicket can slash a specific operator that's receiving delegation from a
/// vault for a maximum amount per epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct NcnVaultSlasherTicket {
    /// The NCN
    pub ncn: Pubkey,

    /// The vault account this slasher can slash
    pub vault: Pubkey,

    /// The slasher signer
    pub slasher: Pubkey,

    /// The max slashable funds per epoch per operator
    max_slashable_per_epoch: PodU64,

    /// The index
    index: PodU64,

    /// State of the NCN slasher
    pub state: SlotToggle,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 263],
}

impl NcnVaultSlasherTicket {
    /// Create a new NcnVaultSlasherTicket and immediately activates it at the given slot, transitioning
    /// it to warming up.
    ///
    /// # Arguments
    /// * `ncn` - The node consensus network
    /// * `vault` - The vault account this slasher can slash
    /// * `slasher` - The slasher signer
    /// * `max_slashable_per_epoch` - The max slashable funds per epoch per operator
    /// * `index` - The index
    /// * `slot_added` - The slot at which the ticket was created
    /// * `bump` - The bump seed for the PDA
    pub fn new(
        ncn: Pubkey,
        vault: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        index: u64,
        bump: u8,
        slot: u64,
    ) -> Self {
        Self {
            ncn,
            vault,
            slasher,
            max_slashable_per_epoch: PodU64::from(max_slashable_per_epoch),
            index: PodU64::from(index),
            state: SlotToggle::new(slot),
            bump,
            reserved: [0; 263],
        }
    }

    pub fn index(&self) -> u64 {
        self.index.into()
    }

    pub fn max_slashable_per_epoch(&self) -> u64 {
        self.max_slashable_per_epoch.into()
    }

    /// Returns the seeds for the PDA
    ///
    /// # Arguments
    /// * `ncn` - The node consensus network
    /// * `vault` - The vault account this slasher can slash
    /// * `slasher` - The slasher signer
    ///
    /// # Returns
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn seeds(ncn: &Pubkey, vault: &Pubkey, slasher: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_slasher_ticket".to_vec(),
            ncn.as_ref().to_vec(),
            vault.as_ref().to_vec(),
            slasher.as_ref().to_vec(),
        ])
    }

    /// Find the program address for the NcnVaultSlasherTicket
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `ncn` - The node consensus network
    /// * `vault` - The vault account this slasher can slash
    /// * `slasher` - The slasher signer
    ///
    /// # Returns
    /// * `Pubkey` - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn find_program_address(
        program_id: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn, vault, slasher);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the account as an [`NcnVaultSlasherTicket`] account, returning an error if it is not.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `ncn_vault_slasher_ticket` - The account to load the NCN vault slasher ticket from
    /// * `ncn` - The NCN account
    /// * `vault` - The vault account
    /// * `slasher` - The slasher account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        ncn_vault_slasher_ticket: &AccountInfo,
        ncn: &AccountInfo,
        vault: &AccountInfo,
        slasher: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if ncn_vault_slasher_ticket.owner.ne(program_id) {
            msg!("NCN vault slasher ticket account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if ncn_vault_slasher_ticket.data_is_empty() {
            msg!("NCN vault slasher ticket account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !ncn_vault_slasher_ticket.is_writable {
            msg!("NCN vault slasher ticket account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if ncn_vault_slasher_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("NCN vault slasher ticket account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey =
            Self::find_program_address(program_id, ncn.key, vault.key, slasher.key).0;
        if ncn_vault_slasher_ticket.key.ne(&expected_pubkey) {
            msg!("NCN vault slasher ticket account is not at the correct PDA");
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
    fn test_ncn_vault_slasher_ticket_no_padding() {
        let ncn_vault_slasher_ticket_size = std::mem::size_of::<NcnVaultSlasherTicket>();
        let sum_of_fields = size_of::<Pubkey>() + // ncn
            size_of::<Pubkey>() + // vault
            size_of::<Pubkey>() + // slasher
            size_of::<PodU64>() + // max_slashable_per_epoch
            size_of::<PodU64>() + // index
            size_of::<SlotToggle>() + // state
            size_of::<u8>() + // bump
            263; // reserved
        assert_eq!(ncn_vault_slasher_ticket_size, sum_of_fields);
    }

    #[test]
    fn test_ncn_vault_slasher_ticket_inactive_on_creation() {
        let slot = 1;
        let ncn_vault_slasher_ticket = NcnVaultSlasherTicket::new(
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            0,
            0,
            0,
            slot,
        );
        assert_eq!(
            ncn_vault_slasher_ticket.state.state(slot + 1, 100),
            SlotToggleState::Inactive
        );
    }
}
