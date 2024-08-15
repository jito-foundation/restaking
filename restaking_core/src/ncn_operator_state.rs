use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for NcnOperatorState {
    const DISCRIMINATOR: u8 = 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct NcnOperatorState {
    /// The NCN account
    pub ncn: Pubkey,

    /// The operator account
    pub operator: Pubkey,

    /// Index
    pub index: u64,

    /// State of the ncn opt-ing in to the operator
    pub ncn_opt_in_state: SlotToggle,

    /// State of the operator opt-ing in to the ncn
    pub operator_opt_in_state: SlotToggle,

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl NcnOperatorState {
    pub const fn new(ncn: Pubkey, operator: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            ncn,
            operator,
            index,
            ncn_opt_in_state: SlotToggle::new(0),
            operator_opt_in_state: SlotToggle::new(0),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(ncn: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_operator_state".to_vec(),
            ncn.to_bytes().to_vec(),
            operator.to_bytes().to_vec(),
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
