use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for NcnOperatorTicket {
    const DISCRIMINATOR: u8 = 5;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct NcnOperatorTicket {
    /// The NCN
    pub ncn: Pubkey,

    /// The operator
    pub operator: Pubkey,

    /// The index
    pub index: u64,

    /// The state
    pub state: SlotToggle,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl NcnOperatorTicket {
    pub const fn new(ncn: Pubkey, operator: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            ncn,
            operator,
            index,
            state: SlotToggle::new(slot_added),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(ncn: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_operator_ticket".to_vec(),
            ncn.as_ref().to_vec(),
            operator.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        ncn: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn, operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
