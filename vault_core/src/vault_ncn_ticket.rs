//! The [`VaultNcnTicket`] account tracks a vault supporting a node consensus network. It can be
//! enabled and disabled over time by the vault NCN admin.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for VaultNcnTicket {
    const DISCRIMINATOR: u8 = 3;
}

/// The [`VaultNcnTicket`] account tracks a vault supporting a node consensus network. It can be
/// enabled and disabled over time by the vault NCN admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultNcnTicket {
    /// The vault account
    pub vault: Pubkey,

    /// The ncn account
    pub ncn: Pubkey,

    /// The index
    pub index: u64,

    /// The slot toggle
    pub state: SlotToggle,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultNcnTicket {
    pub const fn new(vault: Pubkey, ncn: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            vault,
            ncn,
            index,
            state: SlotToggle::new(0),
            bump,
            reserved: [0; 7],
        }
    }

    /// The seeds for the PDA
    ///
    /// # Arguments
    /// * `vault` - The vault account
    /// * `ncn` - The ncn account
    pub fn seeds(vault: &Pubkey, ncn: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_ncn_ticket".to_vec(),
            vault.as_ref().to_vec(),
            ncn.as_ref().to_vec(),
        ])
    }

    /// Find the program address for the PDA
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault` - The vault account
    /// * `ncn` - The ncn account
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds
    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, ncn);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the [`VaultNcnTicket`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault_ncn_ticket` - The account to load
    /// * `vault` - The vault account
    /// * `ncn` - The ncn account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        vault_ncn_ticket: &AccountInfo,
        vault: &AccountInfo,
        ncn: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if vault_ncn_ticket.owner.ne(program_id) {
            msg!("Vault NCN ticket account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if vault_ncn_ticket.data_is_empty() {
            msg!("Vault NCN ticket account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !vault_ncn_ticket.is_writable {
            msg!("Vault NCN ticket account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_ncn_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Vault NCN ticket account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(program_id, vault.key, ncn.key).0;
        if vault_ncn_ticket.key.ne(&expected_pubkey) {
            msg!("Vault NCN ticket account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
