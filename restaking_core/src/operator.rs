//! The Operator account stores global information for a particular operator
//! including the admin, voter, and the number of NCN and vault accounts.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::pubkey::Pubkey;

impl Discriminator for Operator {
    const DISCRIMINATOR: u8 = 3;
}

/// The Operator account stores global information for a particular operator
/// including the admin, voter, and the number of NCN and vault accounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Operator {
    /// The base pubkey used as a seed for the PDA
    pub base: Pubkey,

    /// The admin pubkey
    pub admin: Pubkey,

    /// The NCN admin can add and remove support for NCNs in the restaking protocol
    pub ncn_admin: Pubkey,

    /// The vault admin can add and remove support for vaults in the restaking protocol
    pub vault_admin: Pubkey,

    /// The withdrawal admin can withdraw assets from the operator
    pub withdrawal_admin: Pubkey,

    /// The withdrawal fee wallet where withdrawn funds are sent
    pub withdrawal_fee_wallet: Pubkey,

    /// The voter pubkey can be used as the voter for signing transactions for interacting
    /// with various NCN programs. NCNs can also opt for their own signing infrastructure.
    pub voter: Pubkey,

    /// The operator index
    pub index: u64,

    /// The number of NcnOperatorTickets associated with the operator.
    /// Helpful for indexing all available OperatorNcnTickets.
    pub ncn_count: u64,

    /// The number of NcnVaultTickets associated with the operator.
    /// Helpful for indexing all available OperatorVaultTickets.
    pub vault_count: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    pub reserved_space: [u8; 7],
}

impl Operator {
    /// Create a new Operator account
    /// # Arguments
    /// * `base` - The base account used as a PDA seed
    /// * `admin` - The admin of the Operator
    /// * `index` - The index of the Operator
    /// * `bump` - The bump seed for the PDA
    pub const fn new(base: Pubkey, admin: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            base,
            admin,
            ncn_admin: admin,
            vault_admin: admin,
            withdrawal_admin: admin,
            withdrawal_fee_wallet: admin,
            voter: admin,
            index,
            ncn_count: 0,
            vault_count: 0,
            bump,
            reserved_space: [0; 7],
        }
    }

    /// Returns the seeds for the PDA
    ///
    /// # Arguments
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"operator".to_vec(), base.as_ref().to_vec()])
    }

    /// Find the program address for the Operator account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * `Pubkey` - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
