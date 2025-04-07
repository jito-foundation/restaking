use anchor_lang::prelude::Pubkey;
use borsh::BorshDeserialize;
use jito_restaking_client_common::log::{account_header, field, section_header, PrettyDisplay};

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

impl PrettyDisplay for Metadata {
    fn pretty_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&account_header("Token Metadata Account"));

        output.push_str(&section_header("Basic Information"));
        output.push_str(&field("Update Authority", self.update_authority));
        output.push_str(&field("Mint", self.mint));
        output.push_str(&field("Name", &self.name));
        output.push_str(&field("Symbol", &self.symbol));
        output.push_str(&field("URI", &self.uri));
        output.push_str(&field(
            "Seller Fee Basis Points",
            self.seller_fee_basis_points,
        ));

        output
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use jito_restaking_client_common::log::PrettyDisplay;

    use crate::log::metadata::Metadata;

    #[test]
    fn test_config_pretty_display_structure() {
        let metadata = Metadata {
            key: 0,
            update_authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            name: String::from("Jito Staked SOL"),
            symbol: String::from("JitoSOL"),
            uri: String::from("https://example.com"),
            seller_fee_basis_points: 0,
            creators: None,
            primary_sale_happened: false,
            is_mutable: false,
        };

        let output = metadata.pretty_display();

        assert!(output.contains(&metadata.update_authority.to_string()));
        assert!(output.contains(&metadata.mint.to_string()));
        assert!(output.contains(&metadata.name.to_string()));
        assert!(output.contains(&metadata.symbol.to_string()));
        assert!(output.contains(&metadata.uri.to_string()));
        assert!(output.contains(&metadata.seller_fee_basis_points.to_string()));
    }
}
