use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for VaultNcnTicket {
    const DISCRIMINATOR: u8 = 3;
}

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

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultNcnTicket {
    pub const fn new(vault: Pubkey, ncn: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            vault,
            ncn,
            index,
            state: SlotToggle::new(slot_added),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(vault: &Pubkey, ncn: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_ncn_ticket".to_vec(),
            vault.as_ref().to_vec(),
            ncn.as_ref().to_vec(),
        ])
    }

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
}
