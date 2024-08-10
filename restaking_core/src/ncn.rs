use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Ncn {
    /// The base account used as a PDA seed
    pub base: Pubkey,

    /// The admin of the NCN
    pub admin: Pubkey,

    /// The operator admin of the NCN
    pub operator_admin: Pubkey,

    /// The vault admin of the NCN
    pub vault_admin: Pubkey,

    /// The slasher admin of the NCN
    pub slasher_admin: Pubkey,

    /// The withdraw admin of the NCN
    pub withdraw_admin: Pubkey,

    /// The withdraw fee wallet of the NCN
    pub withdraw_fee_wallet: Pubkey,

    /// The index of the NCN
    pub index: u64,

    /// Number of operator accounts associated with the NCN
    pub operator_count: u64,

    /// Number of vault accounts associated with the NCN
    pub vault_count: u64,

    /// Number of slasher accounts associated with the NCN
    pub slasher_count: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl Discriminator for Ncn {
    const DISCRIMINATOR: u8 = 2;
}

impl Ncn {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        base: Pubkey,
        admin: Pubkey,
        operator_admin: Pubkey,
        vault_admin: Pubkey,
        slasher_admin: Pubkey,
        withdraw_admin: Pubkey,
        withdraw_fee_wallet: Pubkey,
        ncn_index: u64,
        bump: u8,
    ) -> Self {
        Self {
            base,
            admin,
            operator_admin,
            vault_admin,
            slasher_admin,
            withdraw_admin,
            withdraw_fee_wallet,
            index: ncn_index,
            operator_count: 0,
            vault_count: 0,
            slasher_count: 0,
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"ncn".to_vec(), base.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
