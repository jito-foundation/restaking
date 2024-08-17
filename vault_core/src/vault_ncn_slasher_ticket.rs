//! The [`VaultNcnSlasherTicket`] account tracks a vault's support for a node consensus network
//! slasher. It can be enabled and disabled over time by the vault slasher admin.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

impl Discriminator for VaultNcnSlasherTicket {
    const DISCRIMINATOR: u8 = 5;
}

/// The [`VaultNcnSlasherTicket`] account tracks a vault's support for a node consensus network
/// slasher. It can be enabled and disabled over time by the vault slasher admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultNcnSlasherTicket {
    /// The vault
    pub vault: Pubkey,

    /// The NCN
    pub ncn: Pubkey,

    /// The slasher
    pub slasher: Pubkey,

    /// The maximum slashable per epoch per operator
    pub max_slashable_per_epoch: u64,

    /// The index
    pub index: u64,

    /// The slot toggle
    pub state: SlotToggle,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultNcnSlasherTicket {
    pub const fn new(
        vault: Pubkey,
        ncn: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        index: u64,
        bump: u8,
    ) -> Self {
        Self {
            vault,
            ncn,
            slasher,
            max_slashable_per_epoch,
            index,
            state: SlotToggle::new(0),
            bump,
            reserved: [0; 7],
        }
    }

    /// Returns the seeds for the PDA
    /// # Arguments
    /// * `vault` - The vault
    /// * `ncn` - The node consensus network
    /// * `slasher` - The slasher
    pub fn seeds(vault: &Pubkey, ncn: &Pubkey, slasher: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_slasher_ticket".to_vec(),
            vault.as_ref().to_vec(),
            ncn.as_ref().to_vec(),
            slasher.as_ref().to_vec(),
        ])
    }

    /// Find the program address for the [`VaultNcnSlasherTicket`]
    /// account.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault` - The vault
    /// * `ncn` - The node consensus network
    /// * `slasher` - The slasher
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds
    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, ncn, slasher);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the [`VaultNcnSlasherTicket`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault_ncn_slasher_ticket` - The [`VaultNcnSlasherTicket`] account
    /// * `vault` - The [`Vault`] account
    /// * `ncn` - The ncn account
    /// * `slasher` - The slasher account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        vault_ncn_slasher_ticket: &AccountInfo,
        vault: &AccountInfo,
        ncn: &AccountInfo,
        slasher: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if vault_ncn_slasher_ticket.owner.ne(program_id) {
            msg!("Vault NCN slasher ticket account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if vault_ncn_slasher_ticket.data_is_empty() {
            msg!("Vault NCN slasher ticket account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !vault_ncn_slasher_ticket.is_writable {
            msg!("Vault NCN slasher ticket account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_ncn_slasher_ticket.data.borrow()[0].ne(&VaultNcnSlasherTicket::DISCRIMINATOR) {
            msg!("Vault NCN slasher ticket account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = VaultNcnSlasherTicket::find_program_address(
            program_id,
            vault.key,
            ncn.key,
            slasher.key,
        )
        .0;
        if vault_ncn_slasher_ticket.key.ne(&expected_pubkey) {
            msg!("Vault NCN slasher ticket account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
