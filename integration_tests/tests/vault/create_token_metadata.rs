use jito_vault_core::{config::Config, vault::Vault, vault_delegation_list::VaultDelegationList};
use mpl_token_metadata::accounts::Metadata;
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_create_token_metadata_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut vault_program_client = fixture.vault_program_client();

    let backing_token_mint = Keypair::new();
    fixture
        .create_token_mint(&backing_token_mint)
        .await
        .unwrap();

    let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
    let config_admin = Keypair::new();

    fixture.transfer(&config_admin.pubkey(), 1.0).await.unwrap();

    vault_program_client
        .initialize_config(&config_pubkey, &config_admin)
        .await
        .unwrap();

    // Initialize Vault
    let vault_base = Keypair::new();
    let vault_pubkey =
        Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;
    let vault_delegation_list =
        VaultDelegationList::find_program_address(&jito_vault_program::id(), &vault_pubkey).0;
    let lrt_mint = Keypair::new();
    let vault_admin = Keypair::new();

    fixture.transfer(&vault_admin.pubkey(), 1.0).await.unwrap();

    vault_program_client
        .initialize_vault(
            &config_pubkey,
            &vault_pubkey,
            &vault_delegation_list,
            &lrt_mint,
            &backing_token_mint,
            &vault_admin,
            &vault_base,
            99,
            100,
        )
        .await
        .unwrap();

    // Create token metadata
    let name = "restaking JTO";
    let symbol = "rJTO";
    let uri = "https://www.jito.network/restaking/";

    let metadata_pubkey = Metadata::find_pda(&lrt_mint.pubkey()).0;
    vault_program_client
        .create_token_metadata(
            &vault_pubkey,
            &lrt_mint.pubkey(),
            &metadata_pubkey,
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
    assert_eq!(token_metadata.update_authority, vault_pubkey);
    assert_eq!(token_metadata.mint, lrt_mint.pubkey());
    assert!(token_metadata.name.contains(name));
    assert!(token_metadata.symbol.contains(symbol));
}
