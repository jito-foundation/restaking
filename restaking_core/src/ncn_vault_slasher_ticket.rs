//! The NcnVaultSlasherTicket tracks the opting-in of a slasher to a particular vault.
//! The NcnVaultSlasherTicket can be activated and deactivated over time by the NCN slasher admin.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for NcnVaultSlasherTicket {
    const DISCRIMINATOR: u8 = 8;
}

/// The NcnVaultSlasherTicket is created by the NCN and it tracks the state of a node consensus network
/// opting-in to a vault slasher. The NcnVaultSlasherTicket can be activated and deactivated over time.
/// The NcnVaultSlasherTicket can slash a specific operator that's receiving delegation from a
/// vault for a maximum amount per epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct NcnVaultSlasherTicket {
    /// The NCN
    pub ncn: Pubkey,

    /// The vault account this slasher can slash
    pub vault: Pubkey,

    /// The slasher signer
    pub slasher: Pubkey,

    /// The max slashable funds per epoch per operator
    pub max_slashable_per_epoch: u64,

    /// The index
    pub index: u64,

    /// State of the NCN slasher
    pub state: SlotToggle,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
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
    pub const fn new(
        ncn: Pubkey,
        vault: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            ncn,
            vault,
            slasher,
            max_slashable_per_epoch,
            index,
            state: SlotToggle::new(slot_added),
            bump,
            reserved: [0; 7],
        }
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
}
