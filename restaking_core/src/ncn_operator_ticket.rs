//! The NCN operator ticket account tracks the state of an NCN opting-in to an operator.
//! The NCN operator ticket account can be activated and deactivated over time by the NCN operator admin.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use solana_program::pubkey::Pubkey;

impl Discriminator for NcnOperatorTicket {
    const DISCRIMINATOR: u8 = 5;
}

/// The NcnOperatorTicket is created by the NCN and it tracks the state of a node consensus network
/// opting-in to an operator. The NcnOperatorTicket can be activated and deactivated over time.
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
    /// Create a new NcnOperatorTicket and immediately activates it at the given slot, transitioning
    /// it to warming up.
    ///
    /// # Arguments
    /// * `ncn` - The node consensus network
    /// * `operator` - The operator
    /// * `index` - The index
    /// * `slot_added` - The slot at which the ticket was created
    /// * `bump` - The bump seed for the PDA
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

    /// Returns the seeds for the PDA
    pub fn seeds(ncn: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_operator_ticket".to_vec(),
            ncn.as_ref().to_vec(),
            operator.as_ref().to_vec(),
        ])
    }

    /// Find the program address for the NcnOperatorTicket
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `ncn` - The node consensus network
    /// * `operator` - The operator
    ///
    /// # Returns
    /// * `Pubkey` - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
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
