use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::pubkey::Pubkey;

impl Discriminator for VaultNcnSlasherOperatorTicket {
    const DISCRIMINATOR: u8 = 6;
}

/// Represents a vault node consensus network (NCN) slasher operator ticket, which tracks how much an operator
/// has been slashed by a slasher for a given NCN and vault for a given epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultNcnSlasherOperatorTicket {
    /// The vault slashed
    pub vault: Pubkey,

    /// The node consensus network slashed
    pub ncn: Pubkey,

    /// The slasher
    pub slasher: Pubkey,

    /// The operator
    pub operator: Pubkey,

    /// The epoch
    pub epoch: u64,

    /// The amount slashed for the given epoch
    pub slashed: u64,

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultNcnSlasherOperatorTicket {
    pub const fn new(
        vault: Pubkey,
        ncn: Pubkey,
        slasher: Pubkey,
        operator: Pubkey,
        epoch: u64,
        slashed: u64,
        bump: u8,
    ) -> Self {
        Self {
            vault,
            ncn,
            slasher,
            operator,
            epoch,
            slashed,
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_ncn_slasher_operator".to_vec(),
            vault.to_bytes().to_vec(),
            ncn.to_bytes().to_vec(),
            slasher.to_bytes().to_vec(),
            operator.to_bytes().to_vec(),
            epoch.to_le_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, ncn, slasher, operator, epoch);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
