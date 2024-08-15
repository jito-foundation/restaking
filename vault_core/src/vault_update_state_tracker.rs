use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::pubkey::Pubkey;

impl Discriminator for VaultUpdateStateTracker {
    const DISCRIMINATOR: u8 = 9;
}

/// The [`crate::vault_operator_ticket::VaultUpdateDelegationsTicket`]
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
    pub amount_delegated: u64,
}

impl VaultUpdateStateTracker {
    pub fn new(vault: Pubkey, ncn_epoch: u64) -> Self {
        Self {
            vault,
            ncn_epoch,
            last_updated_index: u64::MAX,
            amount_delegated: 0,
        }
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
}
