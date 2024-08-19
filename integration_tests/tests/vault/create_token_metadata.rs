use jito_vault_sdk::{error::VaultError, inline_mpl_token_metadata};
use solana_sdk::signature::Keypair;

use crate::fixtures::{
    fixture::TestBuilder,
    vault_client::{assert_vault_error, VaultRoot},
};

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
        .setup_config_and_vault(99, 100, 0)
        .await
        .unwrap();

    let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

    // Create token metadata
    let name = "restaking JTO";
    let symbol = "rJTO";
    let uri = "https://www.jito.network/restaking/";

    let metadata_pubkey = inline_mpl_token_metadata::pda::find_metadata_account(&vault.vrt_mint).0;

    vault_program_client
        .create_token_metadata(
            &vault_pubkey,
            &vault_admin,
            &vault.vrt_mint,
            &vault_admin,
            &metadata_pubkey,
            name.to_string(),
            symbol.to_string(),
            uri.to_string(),
        )
        .await
        .unwrap();

    let token_metadata = vault_program_client
        .get_token_metadata(&vault.vrt_mint)
        .await
        .unwrap();

    assert!(token_metadata.name.starts_with(name));
    assert!(token_metadata.symbol.starts_with(symbol));
    assert!(token_metadata.uri.starts_with(uri));
}

#[tokio::test]
async fn test_wrong_admin_signed() {
    let fixture = TestBuilder::new().await;

    let mut vault_program_client = fixture.vault_program_client();

    let (
        _config_admin,
        VaultRoot {
            vault_pubkey,
            vault_admin,
        },
    ) = vault_program_client
        .setup_config_and_vault(99, 100, 0)
        .await
        .unwrap();

    let vault = vault_program_client.get_vault(&vault_pubkey).await.unwrap();

    // Create token metadata
    let name = "restaking JTO";
    let symbol = "rJTO";
    let uri = "https://www.jito.network/restaking/";

    let metadata_pubkey = inline_mpl_token_metadata::pda::find_metadata_account(&vault.vrt_mint).0;

    let bad_admin = Keypair::new();
    let response = vault_program_client
        .create_token_metadata(
            &vault_pubkey,
            &bad_admin,
            &vault.vrt_mint,
            &vault_admin,
            &metadata_pubkey,
            name.to_string(),
            symbol.to_string(),
            uri.to_string(),
        )
        .await;

    assert_vault_error(response, VaultError::VaultAdminInvalid);
}
