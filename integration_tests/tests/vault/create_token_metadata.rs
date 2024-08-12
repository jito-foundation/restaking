use solana_program::pubkey::Pubkey;

use crate::fixtures::{fixture::TestBuilder, vault_client::VaultRoot};

#[tokio::test]
async fn test_create_token_metadata_ok() {
    let fixture = TestBuilder::new().await;

    let mut vault_program_client = fixture.vault_program_client();

    let (
        _config_admin,
        VaultRoot {
            vault_pubkey,
            vault_admin,
        },
    ) = vault_program_client
        .setup_config_and_vault(99, 100)
        .await
        .unwrap();

    let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

    // Create token metadata
    let name = "restaking JTO";
    let symbol = "rJTO";
    let uri = "https://www.jito.network/restaking/";

    let seeds = vec![
        b"metadata".as_ref().to_vec(),
        vault.lrt_mint.to_bytes().to_vec(),
    ];
    let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
    let (metadata_pubkey, _bump) =
        Pubkey::find_program_address(&seeds_iter, &jito_vault_program::id());

    vault_program_client
        .create_token_metadata(
            &metadata_pubkey,
            &vault_pubkey,
            &vault_admin,
            name.to_string(),
            symbol.to_string(),
            uri.to_string(),
        )
        .await
        .unwrap();

    let token_metadata = vault_program_client
        .get_token_metadata(&metadata_pubkey)
        .await
        .unwrap();

    assert_eq!(token_metadata.update_authority.0, vault_pubkey);
    assert_eq!(token_metadata.mint, vault.lrt_mint);
    assert_eq!(token_metadata.name, name);
    assert_eq!(token_metadata.symbol, symbol);
    assert_eq!(token_metadata.uri, uri);
    assert!(token_metadata.additional_metadata.is_empty());
}
