use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for NcnVaultSlasherTicket {
    const DISCRIMINATOR: u8 = 8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct NcnVaultSlasherTicket {
    /// The NCN
    pub ncn: Pubkey,

    /// The vault account this slasher can slash
    pub vault: Pubkey,

    /// The slasher signer
    pub slasher: Pubkey,

    /// The max slashable funds per epoch
    pub max_slashable_per_epoch: u64,

    /// The index
    pub index: u64,

    /// State of the NCN slasher
    pub state: SlotToggle,

    /// The bump seed for the PDA
    bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl NcnVaultSlasherTicket {
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

    pub fn seeds(ncn: &Pubkey, vault: &Pubkey, slasher: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_slasher_ticket".to_vec(),
            ncn.as_ref().to_vec(),
            vault.as_ref().to_vec(),
            slasher.as_ref().to_vec(),
        ])
    }

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
