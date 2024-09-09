use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use shank::{ShankAccount, ShankType};
use solana_program::pubkey::Pubkey;

/// The PriceTable holds the price for each mint in the NCN normalized to some asset.
/// It is the responsibility of the NCN to determine the voting weight for each asset it
/// uses for monetary security.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct PriceTable {
    /// The admin of the NCN
    pub ncn: Pubkey,

    /// Signer allowed to update prices
    pub admin: Pubkey,

    /// The number of vaults in the NCN
    pub table: [PriceTableEntry; 32],
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable, ShankType)]
#[repr(C)]
pub struct PriceTableEntry {
    /// The vault admin of the NCN
    pub mint: Pubkey,

    /// Usually price normalized to SOL
    pub normalization_price: f64,
}

impl Discriminator for PriceTable {
    const DISCRIMINATOR: u8 = 2;
}

impl PriceTable {
    pub fn new(ncn: Pubkey, admin: Pubkey) -> Self {
        Self {
            ncn,
            admin,
            table: [PriceTableEntry::default(); 32],
        }
    }

    pub fn seeds(ncn: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"ncn_price_table".to_vec(), ncn.to_bytes().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, ncn: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
