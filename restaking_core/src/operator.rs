use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::pubkey::Pubkey;

impl Discriminator for Operator {
    const DISCRIMINATOR: u8 = 3;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Operator {
    /// The base pubkey used as a seed for the PDA
    pub base: Pubkey,

    /// The admin pubkey
    pub admin: Pubkey,

    pub ncn_admin: Pubkey,

    pub vault_admin: Pubkey,

    pub withdraw_admin: Pubkey,

    pub withdraw_fee_wallet: Pubkey,

    /// The voter pubkey
    pub voter: Pubkey,

    /// The operator index
    pub index: u64,

    pub ncn_count: u64,

    pub vault_count: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    pub reserved_space: [u8; 7],
}

impl Operator {
    pub const fn new(base: Pubkey, admin: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            base,
            admin,
            ncn_admin: admin,
            vault_admin: admin,
            withdraw_admin: admin,
            withdraw_fee_wallet: admin,
            voter: admin,
            index,
            ncn_count: 0,
            vault_count: 0,
            bump,
            reserved_space: [0; 7],
        }
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"operator".to_vec(), base.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
