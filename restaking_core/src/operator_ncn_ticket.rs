use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for OperatorNcnTicket {
    const DISCRIMINATOR: u8 = 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct OperatorNcnTicket {
    /// The operator account
    pub operator: Pubkey,

    /// The NCN account
    pub ncn: Pubkey,

    pub index: u64,

    pub state: SlotToggle,

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl OperatorNcnTicket {
    pub const fn new(operator: Pubkey, ncn: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            operator,
            ncn,
            index,
            state: SlotToggle::new(slot_added),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(operator: &Pubkey, ncn: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"operator_ncn_ticket".to_vec(),
            operator.to_bytes().to_vec(),
            ncn.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator, ncn);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
