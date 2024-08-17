use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use solana_program_test::BanksClient;
use spl_token_2022::extension::StateWithExtensionsOwned;

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

pub async fn get_token_account(
    banks_client: &mut BanksClient,
    token_account: &Pubkey,
) -> StateWithExtensionsOwned<spl_token_2022::state::Account> {
    let token_account = banks_client
        .get_account(*token_account)
        .await
        .unwrap()
        .unwrap();
    let account_info =
        StateWithExtensionsOwned::<spl_token_2022::state::Account>::unpack(token_account.data)
            .unwrap();

    account_info
}
