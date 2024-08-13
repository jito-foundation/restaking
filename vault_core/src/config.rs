use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{epoch_schedule::DEFAULT_SLOTS_PER_EPOCH, pubkey::Pubkey};

impl Discriminator for Config {
    const DISCRIMINATOR: u8 = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Config {
    /// The configuration admin
    pub admin: Pubkey,

    /// The approved restaking program for this vault
    pub restaking_program: Pubkey,

    /// The length of an epoch in slots
    pub epoch_length: u64,

    /// The number of vaults managed by the program
    pub num_vaults: u64,

    /// The fee cap in basis points ( withdraw and deposit )
    pub fee_cap_bps: u16,

    /// The maximum amount a fee can increase per epoch in basis points
    pub fee_rate_of_change_bps: u16,

    /// The amount a fee can increase above the rate of change in basis points
    pub fee_bump_bps: u16,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 17],
}

impl Config {
    /// Maximum fee cap in basis points
    pub const DEFAULT_FEES_CAP_BPS: u16 = 2_000; // 2%
    /// Maximum rate of change in fee baisis pointer per epoch
    pub const DEFAULT_FEE_RATE_OF_CHANGE_BPS: u16 = 2_500; // 25%
    /// Maximum bump in fee change above the rate of change
    pub const DEFAULT_FEE_BUMP_BPS: u16 = 10; // 0.1%

    pub const fn new(admin: Pubkey, restaking_program: Pubkey, bump: u8) -> Self {
        Self {
            admin,
            restaking_program,
            epoch_length: DEFAULT_SLOTS_PER_EPOCH,
            num_vaults: 0,
            fee_cap_bps: Self::DEFAULT_FEES_CAP_BPS,
            fee_rate_of_change_bps: Self::DEFAULT_FEE_RATE_OF_CHANGE_BPS,
            fee_bump_bps: Self::DEFAULT_FEE_BUMP_BPS,
            bump,
            reserved: [0; 17],
        }
    }

    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"config".to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds();
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
