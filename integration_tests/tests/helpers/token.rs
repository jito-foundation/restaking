use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;

#[derive(Clone, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub key: u8,
    pub update_authority: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<u8>>,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
}
