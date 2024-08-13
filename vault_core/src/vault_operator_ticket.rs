//! The [`VaultOperatorTicket`] account tracks a vault's support for an operator. It can be enabled
//! and disabled over time by the vault operator admin.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for VaultOperatorTicket {
    const DISCRIMINATOR: u8 = 4;
}

/// The [`VaultOperatorTicket`] account tracks a vault's support for an operator. It can be enabled
/// and disabled over time by the vault operator admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultOperatorTicket {
    /// The vault account
    pub vault: Pubkey,

    /// The operator account
    pub operator: Pubkey,

    /// The index
    pub index: u64,

    /// The slot toggle
    pub state: SlotToggle,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultOperatorTicket {
    pub const fn new(
        vault: Pubkey,
        operator: Pubkey,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            vault,
            operator,
            index,
            state: SlotToggle::new(slot_added),
            bump,
            reserved: [0; 7],
        }
    }

    /// The seeds for the PDA
    ///
    pub fn seeds(vault: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_operator_ticket".to_vec(),
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
