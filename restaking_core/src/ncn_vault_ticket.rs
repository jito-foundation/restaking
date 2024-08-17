//! The NcnVaultTicket tracks the state of a node consensus network opting-in to a vault.
//! The NcnVaultTicket can be activated and deactivated over time by the NCN vault admin.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for NcnVaultTicket {
    const DISCRIMINATOR: u8 = 6;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct NcnVaultTicket {
    /// The NCN
    pub ncn: Pubkey,

    /// The vault account
    pub vault: Pubkey,

    pub index: u64,

    pub state: SlotToggle,

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl NcnVaultTicket {
    pub const fn new(ncn: Pubkey, vault: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            ncn,
            vault,
            index,
            state: SlotToggle::new(0),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(ncn: &Pubkey, vault: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_vault_ticket".to_vec(),
            ncn.to_bytes().to_vec(),
            vault.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn, vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
